use anyhow::anyhow;
use polars::prelude::*;
use sqlparser::ast::{
    BinaryOperator, Expr as SqlExpr, Offset as SqlOffset, OrderByExpr, Select, SelectItem, SetExpr,
    Statement, TableFactor, TableWithJoins, Value as SqlValue,
};

pub struct Sql<'a> {
    pub(crate) selection: Vec<Expr>,
    pub(crate) condition: Option<Expr>,
    pub(crate) source: &'a str,
    pub(crate) order_by: Vec<(String, bool)>,
    pub(crate) limit: Option<usize>,
    pub(crate) offset: Option<i64>,
}

pub struct Expression(pub(crate) Box<SqlExpr>);

pub struct Operation(pub(crate) BinaryOperator);

pub struct Projection<'a>(pub(crate) &'a SelectItem);

pub struct Source<'a>(pub(crate) &'a [TableWithJoins]);

pub struct Order<'a>(pub(crate) &'a OrderByExpr);

pub struct Limit<'a>(pub(crate) &'a SqlExpr);

pub struct Offset<'a>(pub(crate) &'a SqlOffset);

pub struct Value(pub(crate) SqlValue);

impl<'a> TryFrom<&'a Statement> for Sql<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a Statement) -> Result<Self, Self::Error> {
        match value {
            Statement::Query(q) => {
                let Select {
                    from: table_with_joins,
                    selection: where_clause,
                    projection,
                    group_by: _,
                    ..
                } = match &q.body.as_ref() {
                    SetExpr::Select(stat) => stat.as_ref(),
                    _ => return Err(anyhow!("We only support Select Query at the moment")),
                };

                let mut selection = Vec::with_capacity(8);
                for p in projection {
                    let expr = Projection(p).try_into()?;
                    selection.push(expr);
                }

                let condition = match where_clause {
                    Some(expr) => Some(Expression(Box::new(expr.to_owned())).try_into()?),
                    None => None,
                };

                let source = Source(table_with_joins).try_into()?;

                let limit = q.limit.as_ref();
                let limit = limit.map(|v| Limit(v).into());

                let offset = q.offset.as_ref();
                let offset = offset.map(|v| Offset(v).into());


                let orders = &q.order_by;
                let mut order_by = Vec::new();
                for expr in orders {
                    order_by.push(Order(expr).try_into()?);
                }

                Ok(Sql {
                    selection,
                    condition,
                    source,
                    order_by,
                    limit,
                    offset,
                })
            }
            _ => Err(anyhow!("We only support Query at the moment")),
        }
    }
}

impl TryFrom<Expression> for Expr {
    type Error = anyhow::Error;

    fn try_from(value: Expression) -> Result<Self, Self::Error> {
        match *value.0 {
            SqlExpr::BinaryOp { left, op, right } => Ok(Expr::BinaryExpr {
                left: Box::new(Expression(left).try_into()?),
                op: Operation(op).try_into()?,
                right: Box::new(Expression(right).try_into()?),
            }),
            SqlExpr::Wildcard => Ok(Self::Wildcard),
            SqlExpr::IsNull(expr) => Ok(Self::is_null(Expression(expr).try_into()?)),
            SqlExpr::IsNotNull(expr) => Ok(Self::is_not_null(Expression(expr).try_into()?)),
            SqlExpr::Identifier(id) => Ok(Self::Column(Arc::from(id.value))),
            SqlExpr::Value(v) => Ok(Self::Literal(Value(v).try_into()?)),
            v => Err(anyhow!("expr {:#?} is not supported", v)),
        }
    }
}

impl TryFrom<Operation> for Operator {
    type Error = anyhow::Error;

    fn try_from(value: Operation) -> Result<Self, Self::Error> {
        match value.0 {
            BinaryOperator::Plus => Ok(Self::Plus),
            BinaryOperator::Minus => Ok(Self::Minus),
            BinaryOperator::Multiply => Ok(Self::Multiply),
            BinaryOperator::Divide => Ok(Self::Divide),
            BinaryOperator::Modulo => Ok(Self::Modulus),
            BinaryOperator::Gt => Ok(Self::Gt),
            BinaryOperator::Lt => Ok(Self::Lt),
            BinaryOperator::GtEq => Ok(Self::GtEq),
            BinaryOperator::LtEq => Ok(Self::LtEq),
            BinaryOperator::Eq => Ok(Self::Eq),
            BinaryOperator::NotEq => Ok(Self::NotEq),
            BinaryOperator::And => Ok(Self::And),
            BinaryOperator::Or => Ok(Self::Or),
            v => Err(anyhow!("Operator {} is not supported", v))
        }
    }
}

impl<'a> TryFrom<Projection<'a>> for Expr {
    type Error = anyhow::Error;

    fn try_from(value: Projection<'a>) -> Result<Self, Self::Error> {
        match value.0 {
            SelectItem::UnnamedExpr(SqlExpr::Identifier(id)) => Ok(col(&id.to_string())),
            SelectItem::ExprWithAlias {
                expr: SqlExpr::Identifier(id),
                alias,
            } => Ok(Expr::Alias(
                Box::new(Expr::Column(Arc::from(id.to_string()))),
                Arc::from(alias.to_string()),
            )),
            SelectItem::QualifiedWildcard(name, _) => Ok(col(&name.to_string())),
            SelectItem::Wildcard(_) => Ok(col("*")),
            item => Err(anyhow!("projection {} not supported", item)),
        }
    }
}

impl<'a> TryFrom<Source<'a>> for &'a str {
    type Error = anyhow::Error;

    fn try_from(value: Source<'a>) -> Result<Self, Self::Error> {
        if value.0.len() != 1 {
            return Err(anyhow!("We only support single data source at the moment"));
        }

        let table = &value.0[0];
        if !table.joins.is_empty() {
            return Err(anyhow!("We do not support join data source at the moment"));
        }

        match &table.relation {
            TableFactor::Table { name, .. } => Ok(&name.0.first().unwrap().value),
            _ => Err(anyhow!("We only support table")),
        }
    }
}

impl<'a> TryFrom<Order<'a>> for (String, bool) {
    type Error = anyhow::Error;

    fn try_from(value: Order<'a>) -> Result<Self, Self::Error> {
        let name = match &value.0.expr {
            SqlExpr::Identifier(id) => id.to_string(),
            expr => {
                return Err(anyhow!("We only support identifier fro order by, got {}", expr));
            }
        };

        Ok((name, !value.0.asc.unwrap_or(true)))
    }
}

impl<'a> From<Limit<'a>> for usize {
    fn from(l: Limit<'a>) -> Self {
        match l.0 {
            SqlExpr::Value(SqlValue::Number(v, _b)) => v.parse().unwrap_or(usize::MAX),
            _ => usize::MAX,
        }
    }
}

impl<'a> From<Offset<'a>> for i64 {
    fn from(o: Offset<'a>) -> Self {
        match o.0 {
            SqlOffset {
                value: SqlExpr::Value(SqlValue::Number(v, _b)),
                ..
            } => v.parse().unwrap_or(0),
            _ => 0,
        }
    }
}

impl TryFrom<Value> for LiteralValue {
    type Error = anyhow::Error;

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        match v.0 {
            SqlValue::Number(v, _) => Ok(LiteralValue::Float64(v.parse().unwrap())),
            SqlValue::Boolean(v) => Ok(LiteralValue::Boolean(v)),
            SqlValue::Null => Ok(LiteralValue::Null),
            v => Err(anyhow!("Value {} is not supported", v)),
        }
    }
}

#[cfg(test)]
mod tests {
    use polars::prelude::col;
    use sqlparser::parser::Parser;
    use crate::covert::Sql;
    use crate::dialect::JrDialect;

    #[test]
    fn parse_sql_works() {
        let url = "https://abc.xyz/abc?a=1&b=2";
        let sql = format!("select a, b, c from {} where a=1 order by c desc limit 5 offset 10", url);

        let stat = &Parser::parse_sql(&JrDialect::default(), sql.as_ref()).unwrap()[0];

        let sql: Sql = stat.try_into().unwrap();

        assert_eq!(sql.source, url);
        assert_eq!(sql.limit, Some(5));
        assert_eq!(sql.offset, Some(10));
        assert_eq!(sql.order_by, vec![("c".into(), true)]);
        assert_eq!(sql.selection, vec![col("a"), col("b"), col("c")]);
    }
}

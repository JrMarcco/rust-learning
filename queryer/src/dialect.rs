use sqlparser::dialect::Dialect;

#[derive(Debug, Default)]
pub struct JrDialect;

impl Dialect for JrDialect {
    fn is_identifier_start(&self, ch: char) -> bool {
        ('a'..='z').contains(&ch) || ('A'..='Z').contains(&ch) || ch == '_'
    }

    fn is_identifier_part(&self, ch: char) -> bool {
        ('a'..='z').contains(&ch) || ('A'..='Z').contains(&ch) || ('0'..='9').contains(&ch) || [':', '/', '?', '&', '=', '-', '_', '.'].contains(&ch)
    }
}

#[cfg(test)]
mod tests {
    use sqlparser::parser::Parser;

    use super::*;

    #[test]
    fn it_works() {
        let url = "https://raw.githubusercontent.com/owid/covid-19-data/master/public/data/latest/owid-covid-latest.csv";

        let sql = format!(
            "SELECT location name, total_cases, new_cases, total_deaths, new_deaths \
            FROM {} where new_deaths >= 500 \
            ORDER BY new_cases DESC \
            LIMIT 6 OFFSET 5",
            url
        );
        assert!(Parser::parse_sql(&JrDialect::default(), &sql).is_ok());
    }
}

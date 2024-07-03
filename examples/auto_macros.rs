use rust_learning::{AutoDebug, AutoDeref};

#[allow(unused)]
#[derive(AutoDebug, AutoDeref)]
#[deref(field = "outer", mutable = true)]
pub struct JrString {
    inner: String,
    outer: String,
    #[debug(skip = true)]
    skip_field: String,
}

fn main() {
    let s = JrString {
        inner: "jrmarcco".to_string(),
        outer: "hello world".to_string(),
        skip_field: "nothing".to_string(),
    };

    println!("{:?}", &s);
    println!("inner: {}", s.inner);
    println!("outer: {}", s.outer);
}

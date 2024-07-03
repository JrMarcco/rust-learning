use rust_learning::AutoDeref;

#[derive(Debug, AutoDeref)]
#[deref(field = "outer", mutable = true)]
pub struct JrString {
    inner: String,
    outer: String,
}

fn main() {
    let s = JrString {
        inner: "jrmarcco".to_string(),
        outer: "hello world".to_string(),
    };

    println!("{:?}", &s);
    println!("inner: {}", s.inner);
    println!("outer: {}", s.outer);
}

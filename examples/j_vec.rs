use anyhow::Result;

#[macro_export]
macro_rules! j_vec {
    () => {
        Vec::new()
    };
    ($elem:expr; $n:expr) => {
        std::vec::from_elem($elem, $n)
    };
    ($($x:expr),+ $(,)?) => {{
        <[_]>::into_vec(Box::new([$($x),*]))
    }}
}

fn main() -> Result<()> {
    let val: Vec<i32> = j_vec![
        "1".parse()?,
        "2".parse()?,
        "3".parse()?,
        "4".parse()?,
        "5".parse()?,
        "6".parse()?,
    ];

    println!("{:?}", val);
    Ok(())
}

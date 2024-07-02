use anyhow::{anyhow, Result};

#[macro_export]
macro_rules! j_try {
    ($expr:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => return Err(err.into()),
        }
    };
}

fn first(s: impl AsRef<str>) -> Result<String> {
    Ok(format!("first: {}", s.as_ref()))
}

fn second(s: impl AsRef<str>) -> Result<String> {
    Ok(format!("second: {}", s.as_ref()))
}

fn third(s: impl AsRef<str>) -> Result<String> {
    Err(anyhow!("third: {}", s.as_ref()))
}

fn main() -> Result<()> {
    let res = j_try!(third(j_try!(second(j_try!(first("jrmarcco"))))));
    println!("Final result: {res}");
    Ok(())
}

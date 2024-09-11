use anyhow::Result;
use derive_builder::Builder;

#[allow(dead_code)]
#[derive(Debug, Builder)]
struct Lorem {
    ipsum: u32,
}

#[derive(Debug, Builder)]
#[builder(try_setter, setter(into))]
struct LoremN {
    name: String,
    ipsum: u8,
}

#[derive(Debug, Builder)]
struct Ipsum {
    #[builder(default = "self.default_name()")]
    name: String,
    #[builder(try_setter, setter(into, name = "foo"))]
    dolor: u8,
    #[builder(default = "vec![]", setter(each(name = "tag", into)))]
    tags: Vec<String>,
}

impl IpsumBuilder {
    fn default_name(&self) -> String {
        String::from("jrmarcco")
    }
}

fn main() -> Result<()> {
    let lorem = LoremBuilder::default().ipsum(32).build()?;
    println!("{:?}", lorem);

    let mut lorem_builder = LoremBuilder::default();
    lorem_builder.ipsum(42);

    let lorem = lorem_builder.build()?;
    println!("{:?}", lorem);

    let lorem_n = LoremNBuilder::default()
        .try_ipsum(1u16)?
        .name("hello")
        .build()
        .expect("1 fits into a u8");
    println!("{:?}, {} {}", lorem_n, lorem_n.name, lorem_n.ipsum);

    let ipsum = IpsumBuilder::default()
        .try_foo(1u16)?
        .tag("first")
        .tag("second")
        .build()
        .expect("1 fits into a u8");
    println!(
        "{:?}, {} {} {:?}",
        ipsum, ipsum.name, ipsum.dolor, ipsum.tags
    );

    Ok(())
}

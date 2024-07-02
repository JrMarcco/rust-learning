use rust_learning::EnumFrom;

fn main() {
    let up: Directions<i32> = 10.into();
    println!("{:?}", up);

    let down: Directions<i32> = DirectDown::new(20).into();
    println!("{:?}", down);
}

#[allow(unused)]
#[derive(Debug, EnumFrom)]
enum Directions<T> {
    Up(T),
    Down(DirectDown<T>),
}

#[allow(unused)]
#[derive(Debug)]
struct DirectDown<T> {
    speed: T,
}

impl<T> DirectDown<T> {
    fn new(speed: T) -> Self {
        Self { speed }
    }
}

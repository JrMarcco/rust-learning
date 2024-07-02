use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

macro_rules! j_ready {
    ($expr:expr) => {
        match $expr {
            std::task::Poll::Ready(v) => std::task::Poll::Ready(v),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    };
}

struct MyFuture {
    pooled: bool,
    val: usize,
}

impl MyFuture {
    fn new(val: usize) -> Self {
        Self { pooled: false, val }
    }
}

impl Future for MyFuture {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.pooled {
            Poll::Ready(self.val)
        } else {
            self.pooled = true;
            // wake up the waker
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

fn poll_future(cx: &mut Context<'_>) -> Poll<usize> {
    let mut future = MyFuture::new(42);
    let future = Pin::new(&mut future);

    j_ready!(future.poll(cx))
}

#[tokio::main]
async fn main() {
    let mut cx = Context::from_waker(futures::task::noop_waker_ref());
    let ret = poll_future(&mut cx);
    println!("Final result: {:?}", ret);

    let future = MyFuture::new(42);
    println!("Final result: {}", future.await)
}

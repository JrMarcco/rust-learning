use anyhow::Result;
use std::{thread, time::Duration};
use tokio::sync::mpsc;

// tokio async task send message to worker for expensive blocking operation
#[tokio::main]
async fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel(8);
    let handler = worker(rx);

    tokio::spawn(async move {
        let mut i = 0;
        loop {
            i += 1;
            println!("sending message {}", i);
            tx.send("hello world".to_string()).await?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });

    handler.join().unwrap();
    Ok(())
}

fn worker(mut rx: mpsc::Receiver<String>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while let Some(msg) = rx.blocking_recv() {
            let hash = expensive_blocking_operation(msg);
            println!("hash: {}", hash);
        }
    })
}

fn expensive_blocking_operation(s: String) -> String {
    thread::sleep(Duration::from_secs(1));
    blake3::hash(s.as_bytes()).to_string()
}

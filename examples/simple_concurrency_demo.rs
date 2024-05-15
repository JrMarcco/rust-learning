use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};

const PRODUCER_NUM: usize = 4;

#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    idx: usize,
    val: usize,
}

impl Msg {
    fn new(idx: usize, val: usize) -> Self {
        Self { idx, val }
    }
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let val = rand::random::<usize>();
        tx.send(Msg::new(idx, val))?;

        let sleep_time = rand::random::<u8>();
        if sleep_time % 5 == 0 {
            println!("producer {} exit.", idx);
            break;
        }

        thread::sleep(Duration::from_millis(sleep_time as u64 * 10));
    }
    Ok(())
}

fn main() -> Result<()> {
    let (tx, rx) = mpsc::channel();

    for i in 0..PRODUCER_NUM {
        let sender = tx.clone();
        thread::spawn(move || producer(i, sender));
    }
    // 释放 tx 否则 tx 结束
    drop(tx);

    let consumer = thread::spawn(move || {
        for msg in rx {
            println!("consume: {:?}", msg);
        }

        println!("consumer exit.")
    });

    consumer
        .join()
        .map_err(|e| anyhow!("Thread join error: {:?}", e))?;

    Ok(())
}

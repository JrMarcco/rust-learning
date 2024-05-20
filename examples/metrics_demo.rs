use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use rand::Rng;

#[derive(Debug, Clone)]
struct Metrics {
    data: Arc<Mutex<HashMap<String, u64>>>,
}

impl Metrics {
    fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn incr(&self, key: String, delta: u64) -> Result<()> {
        let mut data = self.data.lock().map_err(|e| anyhow!(e.to_string()))?;
        let counter = data.entry(key).or_insert(0);
        *counter += delta;

        Ok(())
    }

    fn snapshot(&self) -> Result<HashMap<String, u64>> {
        Ok(self
            .data
            .lock()
            .map_err(|e| anyhow!(e.to_string()))
            .unwrap()
            .clone())
    }
}

fn main() {
    let metrics = Metrics::new();

    for _i in 0..4 {
        let cloned_metrics = metrics.clone();
        thread::spawn(move || loop {
            let mut rng = rand::thread_rng();

            thread::sleep(Duration::from_secs(rng.gen_range(1..10)));
            cloned_metrics.incr("first".into(), 1).unwrap();
        });
    }

    for _i in 0..4 {
        let cloned_metrics = metrics.clone();
        thread::spawn(move || loop {
            let mut rng = rand::thread_rng();

            thread::sleep(Duration::from_millis(rng.gen_range(500..1000)));
            cloned_metrics.incr("second".into(), 1).unwrap();
        });
    }

    loop {
        thread::sleep(Duration::from_secs(2));
        println!("{:?}", metrics.snapshot());
    }
}
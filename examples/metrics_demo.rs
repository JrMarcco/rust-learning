use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use anyhow::{anyhow, Result};
use rand::Rng;

#[derive(Debug, Clone)]
struct Metrics {
    data: Arc<RwLock<HashMap<String, u64>>>,
}

impl Metrics {
    fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn incr(&self, key: impl Into<String>, delta: u64) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow!(e.to_string()))?;
        let counter = data.entry(key.into()).or_insert(0);
        *counter += delta;

        Ok(())
    }

    fn snapshot(&self) -> Result<HashMap<String, u64>> {
        Ok(self
            .data
            .read()
            .map_err(|e| anyhow!(e.to_string()))
            .unwrap()
            .clone())
    }
}

#[allow(unreachable_code)]
fn main() {
    let metrics = Metrics::new();

    for _i in 0..4 {
        let cloned_metrics = metrics.clone();
        thread::spawn(move || {
            loop {
                let mut rng = rand::thread_rng();

                thread::sleep(Duration::from_secs(rng.gen_range(1..10)));
                cloned_metrics.incr("first", 1)?;
            }

            Ok::<_, anyhow::Error>(())
        });
    }

    for _i in 0..4 {
        let cloned_metrics = metrics.clone();
        thread::spawn(move || {
            loop {
                let mut rng = rand::thread_rng();

                thread::sleep(Duration::from_millis(rng.gen_range(500..1000)));
                cloned_metrics.incr("second", 1)?;
            }

            Ok::<_, anyhow::Error>(())
        });
    }

    loop {
        thread::sleep(Duration::from_secs(2));
        println!("{:?}", metrics.snapshot());
    }
}

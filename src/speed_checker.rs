use std::time::{Duration, Instant};

pub struct SpeedChecker {
    counter: u64,
    interval: Duration,
    started_at: Instant,
}

impl SpeedChecker {
    pub fn new() -> SpeedChecker {
        SpeedChecker {
            counter: 0,
            interval: Duration::from_secs(5),
            started_at: Instant::now(),
        }
    }

    pub fn update(&mut self) {
        if self.started_at.elapsed().gt(&self.interval) {
            println!("Speed: {} H/s", self.counter / self.interval.as_secs());
            self.counter = 0;
            self.started_at = Instant::now();
        } else {
            self.counter += 1;
        }
    }
}
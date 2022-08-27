use std::thread;
use std::time::{Duration, Instant};

pub struct Timer {
    /// MS per loop
    pub time: u32,

    /// Update Start
    pub start: Instant,
}

impl Timer {
    pub fn new(time: u32) -> Self {
        Self {
            time,
            start: Instant::now(),
        }
    }

    pub fn start(&mut self, fun: impl Fn()) {
        loop {
            self.start = Instant::now();
            fun();

            thread::sleep(Duration::from_millis(
                self.time
                    .saturating_sub(self.start.elapsed().as_millis() as u32) as u64,
            ))
        }
    }
}

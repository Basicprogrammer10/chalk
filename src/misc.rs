use std::thread;
use std::time::{Duration, Instant};

use afire::{Content, Response};
use serde_json::json;

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

// Misc Functions
pub fn error_res<T: AsRef<str>>(err: T) -> Response {
    Response::new()
        .status(400)
        .text(json!({"error": err.as_ref()}))
        .content(Content::JSON)
}

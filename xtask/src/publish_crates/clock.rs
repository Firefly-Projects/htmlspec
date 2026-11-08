//! The clock the publisher runs on, passed in so a test runs it on a virtual one.

use std::time::{Duration, Instant, SystemTime};

/// The time, two ways, and a wait.
pub trait Clock {
    /// Time on a monotonic clock since an origin of the clock's choosing. The budget is measured
    /// on it, so a change of the wall clock during a run neither spends nor refunds any of it.
    fn monotonic(&self) -> Duration;

    /// The wall clock's time, which a time to try again, named as an HTTP date, is measured
    /// against.
    fn wall(&self) -> SystemTime;

    /// Waits for `length`.
    fn sleep(&self, length: Duration);
}

/// The process's own clocks.
pub struct SystemClock {
    origin: Instant,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Clock for SystemClock {
    fn monotonic(&self) -> Duration {
        self.origin.elapsed()
    }

    fn wall(&self) -> SystemTime {
        SystemTime::now()
    }

    fn sleep(&self, length: Duration) {
        std::thread::sleep(length);
    }
}

//! Injectable time seam for pacing: production sleeps a real thread; tests
//! manipulate deterministic time. Monotonic reads only (no wall clock) so a
//! system clock jump can never distort pacing.

use std::thread;
use std::time::{Duration, Instant};

/// Injectable clock the paced loop reads and sleeps through.
pub(crate) trait FrameClock {
    /// Monotonic nanoseconds since the clock's own anchor.
    fn now_nanos(&self) -> u64;
    /// Sleeps `nanos`; a value of 0 must return promptly.
    fn sleep_nanos(&self, nanos: u64);
}

/// Real implementation: one process-local monotonic anchor.
pub(crate) struct SystemFrameClock {
    start: Instant,
}

impl SystemFrameClock {
    /// Anchors at construction time; `Instant` is monotonic, so no
    /// wall-clock dependency exists anywhere in pacing.
    pub(crate) fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl FrameClock for SystemFrameClock {
    fn now_nanos(&self) -> u64 {
        u64::try_from(self.start.elapsed().as_nanos()).unwrap_or(u64::MAX)
    }

    fn sleep_nanos(&self, nanos: u64) {
        thread::sleep(Duration::from_nanos(nanos));
    }
}

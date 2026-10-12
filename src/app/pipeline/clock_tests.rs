use crate::app::pipeline::clock::{FrameClock, SystemFrameClock};

#[test]
fn system_frame_clock_reads_are_monotonic_and_non_negative() {
    let clock = SystemFrameClock::new();
    let first = clock.now_nanos();
    let second = clock.now_nanos();
    assert!(second >= first);
}

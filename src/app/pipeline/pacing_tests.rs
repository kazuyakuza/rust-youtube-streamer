use crate::app::pipeline::pacing::{interval_nanos, PacingSchedule, NANOS_PER_SECOND};

#[test]
fn interval_nanos_divides_one_second_across_fps_values() {
    assert_eq!(interval_nanos(25), 40_000_000);
    assert_eq!(interval_nanos(30), 33_333_333);
    assert_eq!(interval_nanos(60), 16_666_666);
}

#[test]
fn interval_nanos_survives_zero_fps_without_panic() {
    assert_eq!(interval_nanos(0), NANOS_PER_SECOND);
}

#[test]
fn pacing_schedule_sleeps_full_interval_when_punctual() {
    let mut schedule = PacingSchedule::started(40, 0);
    assert_eq!(schedule.next_gap(0), 40);
    assert_eq!(schedule.next_gap(40), 40);
    assert_eq!(schedule.next_gap(80), 40);
}

#[test]
fn pacing_schedule_drops_backlog_to_one_immediate_frame() {
    let mut schedule = PacingSchedule::started(40, 0);
    assert_eq!(schedule.next_gap(200), 0);
    assert_eq!(schedule.next_gap(200), 40);
    assert_eq!(schedule.next_gap(1_000), 0);
}

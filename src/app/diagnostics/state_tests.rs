//! Bounded aggregate pins: correlation matching, oldest-eviction at the cap,
//! retained-vs-enqueued counting, high-water tracking, oldest age, and the
//! queue-depth estimate.

use super::state::*;

fn entry(correlation: &str, received_nanos: u64) -> InFlightEntry {
    InFlightEntry {
        correlation: correlation.to_string(),
        received_nanos,
        enqueued_nanos: received_nanos,
        backpressured: false,
    }
}

#[test]
fn inflight_pop_matches_correlation() {
    let mut state = DiagnosticsState::new();
    state.add_enqueued(Some(entry("first", 1)), false);
    state.add_enqueued(Some(entry("second", 2)), false);
    state.take_dequeued(Some("second"), 10);
    let snapshot = state.snapshot(10);
    assert_eq!(snapshot.dequeued_total, 1);
    assert_eq!(snapshot.uncorrelated_total, 0);
    assert_eq!(snapshot.message_channel_ns.count, 1);
    assert_eq!(snapshot.queue_depth_estimate, 1);
}

#[test]
fn inflight_evicts_oldest_at_cap() {
    let mut state = DiagnosticsState::new();
    for index in 0..=IN_FLIGHT_CAP {
        state.add_enqueued(Some(entry(&format!("m-{index}"), index as u64)), false);
    }
    let snapshot = state.snapshot(1_000);
    assert_eq!(snapshot.enqueued_total, IN_FLIGHT_CAP as u64 + 1);
    assert_eq!(snapshot.in_flight_high_water, IN_FLIGHT_CAP as u64);
    assert_eq!(
        snapshot.oldest_in_flight_age_ns,
        Some(1_000 - 1),
        "the oldest entry was evicted, so the front is m-1"
    );
}

#[test]
fn backpressured_message_counts_as_retained_and_enqueued() {
    let mut state = DiagnosticsState::new();
    let mut backpressured = entry("m-1", 0);
    backpressured.backpressured = true;
    state.add_enqueued(Some(backpressured), true);
    let snapshot = state.snapshot(1);
    assert_eq!(snapshot.enqueued_total, 1);
    assert_eq!(snapshot.worker_retained_total, 1);
}

#[test]
fn high_water_tracks_peak_depth() {
    let mut state = DiagnosticsState::new();
    state.add_enqueued(Some(entry("m-1", 0)), false);
    state.add_enqueued(Some(entry("m-2", 0)), false);
    state.add_enqueued(Some(entry("m-3", 0)), false);
    state.take_dequeued(Some("m-1"), 1);
    let snapshot = state.snapshot(1);
    assert_eq!(snapshot.in_flight_high_water, 3);
    assert_eq!(snapshot.queue_depth_estimate, 2);
}

#[test]
fn oldest_age_reads_front_stamp() {
    let mut state = DiagnosticsState::new();
    state.add_enqueued(Some(entry("m-1", 100)), false);
    state.add_enqueued(Some(entry("m-2", 200)), false);
    let snapshot = state.snapshot(500);
    assert_eq!(snapshot.oldest_in_flight_age_ns, Some(400));
}

#[test]
fn queue_depth_is_enqueued_minus_dequeued() {
    let mut state = DiagnosticsState::new();
    state.add_enqueued(Some(entry("m-1", 0)), false);
    state.add_enqueued(Some(entry("m-2", 0)), false);
    state.take_dequeued(Some("m-1"), 1);
    assert_eq!(state.snapshot(1).queue_depth_estimate, 1);
    state.take_dequeued(None, 2);
    let snapshot = state.snapshot(2);
    assert_eq!(snapshot.queue_depth_estimate, 0);
    assert_eq!(snapshot.uncorrelated_total, 1);
}

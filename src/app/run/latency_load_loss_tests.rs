//! Loss, shutdown, and pure-pin coverage for the latency harness: the
//! accounting equality proves no silent mid-session loss, the stop-with-queue
//! scenario documents the queued shutdown loss, and the rational-rate and FPS
//! derivations are pinned without running a scenario.

use crate::config::valid_config;

use super::{
    assert_full_round_trip_accounting, assert_scenario_pacing, configured_interval_nanos, phase,
    rate_profile, run_scenario, scenario, RateProfile, CHANNEL_CAPACITY,
};

const MIXED_DUE_TOTAL: u64 = 40 + 16 * 2 + 64 / 2;

#[test]
fn round_trip_counts_prove_no_silent_loss_and_bounded_memory() {
    let scenario = scenario(
        "latency-mixed-profile",
        vec![
            phase(1, rate_profile(40, 1)),
            phase(16, rate_profile(2, 1)),
            phase(64, rate_profile(1, 2)),
            phase(4 * CHANNEL_CAPACITY, rate_profile(0, 1)),
        ],
    );
    assert_eq!(scenario.due_total(), MIXED_DUE_TOTAL);
    let run = run_scenario(scenario);
    assert_scenario_pacing(&run);
    assert_full_round_trip_accounting(MIXED_DUE_TOTAL, &run);
}

#[test]
fn stop_with_messages_queued_documents_shutdown_loss() {
    let interval = configured_interval_nanos();
    let run = run_scenario(scenario(
        "latency-queued-shutdown",
        vec![
            phase(1, rate_profile(CHANNEL_CAPACITY, 1)),
            phase(23, rate_profile(0, 1)),
        ],
    ));
    assert_eq!(run.total_frames, 24);
    let snapshot = &run.snapshot;
    assert_eq!(snapshot.received_total, CHANNEL_CAPACITY);
    assert_eq!(snapshot.dequeued_total, 23);
    assert_eq!(run.leftover.len() as u64, CHANNEL_CAPACITY - 23);
    assert_eq!(snapshot.queue_depth_estimate, CHANNEL_CAPACITY - 23);
    assert_eq!(snapshot.oldest_in_flight_age_ns, Some(23 * interval));
    assert!(run
        .leftover
        .iter()
        .all(|message| message.to_string() == "author-fixture: text-fixture"));
}

#[test]
fn rate_profile_due_counts_follow_rational_schedule() {
    let profile = RateProfile {
        numerator: 2,
        denominator: 3,
    };
    let mut credit = 0;
    let released: Vec<u64> = (0..6).map(|_| profile.advance(&mut credit)).collect();
    assert_eq!(released, vec![0, 1, 1, 0, 1, 1]);
    assert_eq!(credit, 0, "a full cycle leaves no remainder");
    let burst = RateProfile {
        numerator: 64,
        denominator: 1,
    };
    let mut credit = 0;
    assert_eq!(burst.advance(&mut credit), 64);
    assert_eq!(burst.advance(&mut credit), 64);
    assert_eq!(burst.advance(&mut credit), 64);
    assert_eq!(credit, 0);
}

#[test]
fn scenario_pacing_interval_derives_from_configured_fps() {
    let fps = u64::from(valid_config().video.fps);
    let interval = configured_interval_nanos();
    assert_eq!(interval, 1_000_000_000 / fps);
    assert!(interval * fps + fps > 1_000_000_000);
}

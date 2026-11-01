//! Rate, burst, and sustained-backpressure scenarios over the REAL bounded
//! channel and frame loop: every scenario runs on fake clocks with rational
//! rates, so counts, queue verdicts, and channel latencies are deterministic
//! and no scenario sleeps.

use crate::app::pipeline::PipelinePacing;
use crate::app::run::chat_worker::CHAT_CHANNEL_CAPACITY;
use crate::config::valid_config;

use super::{run_scenario, LatencyScenario, RatePhase, RateProfile, ScenarioOutcome};

const CHANNEL_CAPACITY: u64 = CHAT_CHANNEL_CAPACITY as u64;
const LOW_RATE_FRAMES: u64 = 161;

fn phase(frames: u64, numerator: u64, denominator: u64) -> RatePhase {
    RatePhase {
        frames,
        rate: RateProfile {
            numerator,
            denominator,
        },
    }
}

fn scenario(label: &'static str, phases: Vec<RatePhase>) -> LatencyScenario {
    LatencyScenario { label, phases }
}

fn configured_interval_nanos() -> u64 {
    PipelinePacing::from_config(&valid_config().video)
        .expect("validated FPS derives pacing")
        .interval_nanos()
}

fn assert_scenario_pacing(run: &ScenarioOutcome) {
    assert_eq!(
        run.pacing_interval_nanos,
        configured_interval_nanos(),
        "the scenario interval derives from the configured FPS"
    );
}

#[test]
fn low_rate_below_capacity_drains_per_frame_with_zero_backlog() {
    let interval = configured_interval_nanos();
    let run = run_scenario(scenario(
        "latency-low-rate",
        vec![phase(LOW_RATE_FRAMES, 1, 16)],
    ));
    assert_scenario_pacing(&run);
    let snapshot = &run.snapshot;
    assert_eq!(snapshot.received_total, 10);
    assert_eq!(snapshot.enqueued_total, 10);
    assert_eq!(snapshot.dequeued_total, 10);
    assert_eq!(snapshot.queue_depth_estimate, 0);
    assert_eq!(snapshot.in_flight_high_water, 1);
    assert_eq!(snapshot.oldest_in_flight_age_ns, None);
    assert_eq!(snapshot.message_channel_ns.count, 10);
    assert_eq!(snapshot.message_channel_ns.min_ns, Some(interval));
    assert_eq!(snapshot.message_channel_ns.max_ns, interval);
    assert_eq!(snapshot.render_ns.max_ns, 0);
    assert_eq!(snapshot.write_ns.max_ns, 0);
    assert_eq!(snapshot.write_ns.count, LOW_RATE_FRAMES);
    assert_eq!(
        snapshot.message_channel_ns.buckets[3],
        (10_000_000, Some(50_000_000), 10)
    );
}

#[test]
fn single_burst_temporarily_fills_channel_then_drains() {
    let interval = configured_interval_nanos();
    let run = run_scenario(scenario(
        "latency-single-burst",
        vec![
            phase(1, CHANNEL_CAPACITY, 1),
            phase(CHANNEL_CAPACITY + 2, 0, 1),
        ],
    ));
    assert_scenario_pacing(&run);
    let snapshot = &run.snapshot;
    assert_eq!(snapshot.received_total, CHANNEL_CAPACITY);
    assert_eq!(snapshot.enqueued_total, CHANNEL_CAPACITY);
    assert_eq!(snapshot.dequeued_total, CHANNEL_CAPACITY);
    assert_eq!(snapshot.in_flight_high_water, CHANNEL_CAPACITY);
    assert_eq!(snapshot.queue_depth_estimate, 0);
    assert_eq!(snapshot.full_retry_total, 0);
    assert!(run.leftover.is_empty(), "the burst drains completely");
    assert_eq!(snapshot.message_channel_ns.min_ns, Some(interval));
    assert_eq!(
        snapshot.message_channel_ns.max_ns,
        CHANNEL_CAPACITY * interval
    );
}

#[test]
fn sustained_below_capacity_keeps_backlog_stable() {
    let interval = configured_interval_nanos();
    let run = run_scenario(scenario(
        "latency-sustained-below",
        vec![phase(128, 1, 2), phase(4, 0, 1)],
    ));
    assert_scenario_pacing(&run);
    let snapshot = &run.snapshot;
    assert_eq!(snapshot.received_total, 64);
    assert_eq!(snapshot.dequeued_total, 64);
    assert_eq!(snapshot.in_flight_high_water, 1);
    assert!(run.leftover.is_empty(), "the backlog returns to rest");
    assert_eq!(snapshot.message_channel_ns.min_ns, Some(interval));
    assert_eq!(snapshot.message_channel_ns.max_ns, interval);
}

#[test]
fn sustained_at_capacity_holds_one_frame_backlog() {
    let interval = configured_interval_nanos();
    let run = run_scenario(scenario("latency-sustained-at", vec![phase(128, 1, 1)]));
    assert_scenario_pacing(&run);
    let snapshot = &run.snapshot;
    assert_eq!(
        snapshot.in_flight_high_water, 2,
        "one queued message plus the one the frame loop just dequeued"
    );
    assert_eq!(snapshot.dequeued_total, snapshot.received_total - 1);
    assert_eq!(run.leftover.len() as u64, 1);
    assert_eq!(
        snapshot.oldest_in_flight_age_ns,
        Some(0),
        "the single queued message was enqueued on the final frame tick"
    );
    assert_eq!(snapshot.message_channel_ns.min_ns, Some(interval));
    assert_eq!(snapshot.message_channel_ns.max_ns, interval);
}

#[test]
fn sustained_above_capacity_pins_queue_at_capacity_and_drains_after_rate_falls() {
    let interval = configured_interval_nanos();
    let run = run_scenario(scenario(
        "latency-sustained-above",
        vec![phase(128, 2, 1), phase(4 * CHANNEL_CAPACITY, 0, 1)],
    ));
    assert_scenario_pacing(&run);
    let snapshot = &run.snapshot;
    assert_eq!(snapshot.received_total, 2 * 128);
    assert_eq!(snapshot.enqueued_total, 2 * 128);
    assert_eq!(snapshot.dequeued_total, 2 * 128);
    assert_eq!(
        snapshot.in_flight_high_water,
        CHANNEL_CAPACITY + 1,
        "the full channel plus the message the frame loop just dequeued"
    );
    assert_eq!(snapshot.queue_depth_estimate, 0);
    assert!(
        run.leftover.is_empty(),
        "the recovery tail drains the queue"
    );
    assert_eq!(snapshot.message_channel_ns.min_ns, Some(interval));
    assert_eq!(
        snapshot.message_channel_ns.max_ns,
        CHANNEL_CAPACITY * interval
    );
}

#[test]
fn recovery_after_burst_drains_backlog_to_zero() {
    let run = run_scenario(scenario(
        "latency-recovery-after-burst",
        vec![
            phase(1, CHANNEL_CAPACITY + 8, 1),
            phase(4 * CHANNEL_CAPACITY, 0, 1),
        ],
    ));
    assert_scenario_pacing(&run);
    let snapshot = &run.snapshot;
    assert_eq!(snapshot.received_total, CHANNEL_CAPACITY + 8);
    assert_eq!(snapshot.enqueued_total, CHANNEL_CAPACITY + 8);
    assert_eq!(snapshot.dequeued_total, CHANNEL_CAPACITY + 8);
    assert_eq!(
        snapshot.in_flight_high_water,
        CHANNEL_CAPACITY + 1,
        "the pinned channel plus the message the frame loop just dequeued"
    );
    assert_eq!(snapshot.queue_depth_estimate, 0);
    assert!(run.leftover.is_empty(), "the surplus drains to zero");
}

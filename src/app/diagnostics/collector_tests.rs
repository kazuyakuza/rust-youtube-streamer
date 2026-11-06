//! Collector pins: the disabled-by-default guarantee, enabled construction,
//! deterministic stage propagation, uncorrelated counting, the summary timer,
//! and the secret-free row contract.

use crate::chat::ChatMessage;
use crate::config::diagnostics_config::DiagnosticsConfig;
use crate::config::valid_config;
use crate::config::AppConfig;

use super::diagnostics_test_fixtures::*;
use super::DiagnosticsCollector;

fn diagnostics_of(config: &mut AppConfig) -> &mut DiagnosticsConfig {
    config
        .diagnostics
        .as_mut()
        .expect("the fixture carries a diagnostics section")
}

fn diagnostics_config() -> AppConfig {
    let mut config = valid_config();
    diagnostics_of(&mut config).pipeline_timing_events = Some(false);
    config
}

fn secret_message() -> ChatMessage {
    ChatMessage::new("SECRET-AUTHOR", "SECRET-TEXT")
        .with_message_id("corr-1")
        .with_timestamp(Some(1_728_381_296))
}

#[test]
fn from_config_disabled_returns_none_for_absent_section() {
    let mut config = valid_config();
    config.diagnostics = None;
    assert!(DiagnosticsCollector::from_config(&config).is_none());
}

#[test]
fn from_config_disabled_returns_none_for_false_flags() {
    assert!(DiagnosticsCollector::from_config(&diagnostics_config()).is_none());
}

#[test]
fn from_config_disabled_ignores_output_file() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let mut config = diagnostics_config();
    diagnostics_of(&mut config).output_file = Some(path.to_string_lossy().into_owned());
    assert!(DiagnosticsCollector::from_config(&config).is_none());
    assert!(!path.exists(), "disabled diagnostics never creates a file");
}

#[test]
fn from_config_enabled_returns_some() {
    let mut config = diagnostics_config();
    diagnostics_of(&mut config).pipeline_timing_events = Some(true);
    assert!(DiagnosticsCollector::from_config(&config).is_some());
}

#[test]
fn stages_propagate_receive_enqueue_dequeue_render_write() {
    let harness = enabled_collector_for_tests();
    drive_stage_cycle(&harness, &secret_message());
    let snapshot = harness.snapshot();
    assert_eq!(snapshot.received_total, 1);
    assert_eq!(snapshot.enqueued_total, 1);
    assert_eq!(snapshot.dequeued_total, 1);
    assert_eq!(snapshot.message_channel_ns.count, 1);
    assert_eq!(snapshot.message_channel_ns.mean_ns, 20);
    assert_eq!(snapshot.message_render_ns.mean_ns, 5);
    assert_eq!(snapshot.render_ns.mean_ns, 5);
    assert_eq!(snapshot.write_ns.mean_ns, 7);
    assert_eq!(snapshot.queue_depth_estimate, 0);
}

#[test]
fn uncorrelated_message_is_counted_not_dropped() {
    let harness = enabled_collector_for_tests();
    let message = ChatMessage::new("alice", "Hello!");
    harness.collector.note_worker_receipt();
    harness
        .collector
        .report_enqueue_success(message.message_id(), message.timestamp());
    harness.collector.note_message_dequeued(&message);
    let snapshot = harness.snapshot();
    assert_eq!(snapshot.enqueued_total, 1);
    assert_eq!(snapshot.dequeued_total, 1);
    assert_eq!(snapshot.uncorrelated_total, 1);
}

#[test]
fn emits_summary_after_interval_and_not_earlier() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let harness = collector_with_output_file(&path);

    harness.collector.maybe_emit_summary();
    assert!(!path.exists(), "the first call only starts the interval");

    harness.advance(999_999_999);
    harness.collector.maybe_emit_summary();
    assert!(!path.exists(), "the summary is not early");

    harness.advance(1);
    harness.collector.maybe_emit_summary();
    let content = std::fs::read_to_string(&path).expect("summary row must be written");
    assert_eq!(content.lines().count(), 1);
    assert!(content.contains("\"event\":\"summary\""));
    assert!(content.contains("\"clock_note\""));
    assert!(
        !content.contains("\"provider_epoch_seconds\""),
        "summary rows never carry the per-message field"
    );
}

#[test]
fn rows_never_carry_message_text_or_author() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let harness = collector_with_output_file(&path);
    drive_stage_cycle(&harness, &secret_message());
    let content = std::fs::read_to_string(&path).expect("timing rows must be written");
    assert!(content.contains("corr-1"));
    assert!(!content.contains("SECRET-AUTHOR"));
    assert!(!content.contains("SECRET-TEXT"));
    assert!(!content.contains("\"author\""));
}

#[test]
fn queue_metrics_alone_does_not_emit_timing_rows_or_histograms() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let mut diagnostics = diagnostics_with_capabilities(false, true, false);
    diagnostics.output_file = Some(path.to_string_lossy().into_owned());
    let harness = collector_for_config(&diagnostics);
    drive_stage_cycle(&harness, &secret_message());

    let content = std::fs::read_to_string(&path).expect("queue rows must be written");
    assert!(
        content.contains("\"event\":\"queue\""),
        "queue metrics still emit queue rows"
    );
    assert!(!content.contains("\"stage\":\"dequeued\""));
    assert!(!content.contains("\"stage\":\"rendered\""));
    assert!(!content.contains("\"stage\":\"written\""));

    let snapshot = harness.snapshot();
    assert_eq!(snapshot.enqueued_total, 1);
    assert_eq!(snapshot.dequeued_total, 1);
    assert_eq!(snapshot.message_render_ns.count, 0);
    assert_eq!(snapshot.render_ns.count, 0);
    assert_eq!(snapshot.write_ns.count, 0);
}

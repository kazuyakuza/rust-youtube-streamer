//! Orchestrator tests (plan §4 roster B, happy-path and contract
//! subset; the per-step failure subset lives in the
//! `live_ingestion_failure_tests` sibling): the full happy script, the
//! exact command construction from the validated config, the operator
//! sequence, the pre-cancel contract (inserts are NOT cancellation-
//! gated) and the destination redaction — all against the shared
//! scripted transport fake (`live_test_fixtures`, Amendment 1). No
//! network, no Google.

use crate::config::model::AppConfig;
use crate::config::valid_config;
use crate::youtube::live_test_fixtures::*;

use super::*;

pub(super) const FIXED_NOW_EPOCH_SECS: u64 = 1_760_000_000;

pub(super) fn preparer() -> LiveBroadcastPreparer {
    LiveBroadcastPreparer::from_config(&valid_config(), FIXED_NOW_EPOCH_SECS)
}

fn variable_pairing_config() -> AppConfig {
    let mut config = valid_config();
    config.video.width = 123;
    config.video.height = 456;
    config.video.fps = 24;
    config
}

pub(super) fn queue_preparation(transport: &ScriptedLiveApiTransport) {
    queued(&transport.create_stream_results, Ok(inserted_stream()));
    queued(
        &transport.create_broadcast_results,
        Ok(inserted_broadcast()),
    );
    queued(&transport.bind_results, Ok(bound_broadcast()));
    queued(
        &transport.lookup_stream_results,
        Ok(Some(confirmed_ready_stream())),
    );
    queued(
        &transport.lookup_broadcast_results,
        Ok(Some(broadcast_with_lifecycle("ready"))),
    );
}

/// Failure extraction without `expect_err` (`PreparedBroadcast` is
/// Debug-free by the redaction contract).
pub(super) fn preparation_failure(
    outcome: Result<PreparedBroadcast, LiveApiError>,
) -> LiveApiError {
    let Err(error) = outcome else {
        panic!("expected a preparation failure");
    };
    error
}

#[test]
fn prepare_returns_broadcast_id_and_redacted_ingestion() {
    let transport = ScriptedLiveApiTransport::default();
    queue_preparation(&transport);
    let prepared = preparer()
        .prepare(&transport, &WaitCanceller::new())
        .expect("the happy script prepares");
    assert_eq!(prepared.broadcast_id(), BROADCAST_ID_FIXTURE);
    assert_eq!(
        prepared.into_ingestion().into_url(),
        format!("{INGEST_BASE_FIXTURE}/{STREAM_NAME_FIXTURE}")
    );
}

#[test]
fn prepare_records_exact_commands_from_validated_config() {
    let transport = ScriptedLiveApiTransport::default();
    queue_preparation(&transport);
    preparer()
        .prepare(&transport, &WaitCanceller::new())
        .expect("the happy script prepares");
    let captured_stream = transport.captured_create_stream.borrow();
    let stream = captured_stream
        .as_ref()
        .expect("the O1 command was captured");
    assert_eq!(stream.title, "Rust YouTube Streamer Prototype");
    assert_eq!(stream.resolution, "1080p");
    assert_eq!(stream.frame_rate, "30fps");
    let captured_broadcast = transport.captured_create_broadcast.borrow();
    let broadcast = captured_broadcast
        .as_ref()
        .expect("the O2 command was captured");
    assert_eq!(broadcast.title, "Rust YouTube Streamer Prototype");
    assert_eq!(broadcast.description, "YouTube Live streaming prototype");
    assert_eq!(broadcast.privacy_status, "unlisted");
    assert_eq!(
        broadcast.scheduled_start_time,
        scheduled_start_time_from_epoch(FIXED_NOW_EPOCH_SECS)
    );
}

#[test]
fn prepare_records_variable_pairing_for_off_table_video() {
    let transport = ScriptedLiveApiTransport::default();
    queue_preparation(&transport);
    let config = variable_pairing_config();
    LiveBroadcastPreparer::from_config(&config, FIXED_NOW_EPOCH_SECS)
        .prepare(&transport, &WaitCanceller::new())
        .expect("the happy script prepares");
    let captured_stream = transport.captured_create_stream.borrow();
    let stream = captured_stream
        .as_ref()
        .expect("the O1 command was captured");
    assert_eq!(stream.resolution, "variable");
    assert_eq!(stream.frame_rate, "variable");
}

#[test]
fn operator_sequence_is_insert_insert_bind_then_stream_and_broadcast_lookups() {
    let transport = ScriptedLiveApiTransport::default();
    queue_preparation(&transport);
    preparer()
        .prepare(&transport, &WaitCanceller::new())
        .expect("the happy script prepares");
    assert_eq!(
        *transport.operations.borrow(),
        [
            "create_stream",
            "create_broadcast",
            "bind",
            "lookup_stream",
            "lookup_broadcast"
        ]
    );
}

#[test]
fn pre_requested_cancel_executes_inserts_then_waits_cancelled() {
    let transport = ScriptedLiveApiTransport::default();
    queued(&transport.create_stream_results, Ok(inserted_stream()));
    queued(
        &transport.create_broadcast_results,
        Ok(inserted_broadcast()),
    );
    queued(&transport.bind_results, Ok(bound_broadcast()));
    let canceller = WaitCanceller::new();
    canceller.request_cancel();
    let error = preparation_failure(preparer().prepare(&transport, &canceller));
    assert!(matches!(error, LiveApiError::WaitCancelled));
    assert_eq!(
        *transport.operations.borrow(),
        ["create_stream", "create_broadcast", "bind"],
        "inserts are not cancellation-gated; the wait stops before any lookup"
    );
}

#[test]
fn prepared_broadcast_debug_reveals_no_ingestion_material() {
    let transport = ScriptedLiveApiTransport::default();
    queue_preparation(&transport);
    let prepared = preparer()
        .prepare(&transport, &WaitCanceller::new())
        .expect("the happy script prepares");
    let rendered = format!("{prepared:?}");
    assert!(!rendered.contains("rtmps"), "no RTMPS text in: {rendered}");
    assert!(
        !rendered.contains("/ingest"),
        "no destination path in: {rendered}"
    );
    assert!(
        !rendered.contains(STREAM_NAME_FIXTURE),
        "no stream name in: {rendered}"
    );
}

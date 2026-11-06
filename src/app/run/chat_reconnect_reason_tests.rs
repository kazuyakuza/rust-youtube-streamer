//! Reconnect-cause pins: the delivery classification labels every reconnect
//! row with the fixed non-secret cause (`clean_close` vs `transient_failure`)
//! while every other queue row kind stays free of the `reason` key. Offline
//! only, zero sleeps (the test slice is a no-op).

use std::path::Path;

use crate::app::diagnostics::diagnostics_test_fixtures::collector_with_output_file;
use crate::app::run::chat_worker::runtime_chat_fakes::{
    worker_plan, ChatFakes, LIVE_CHAT_ID_FIXTURE,
};
use crate::app::run::chat_worker::{chat_channel, spawn_chat_worker};
use crate::youtube::{response_plain, text_item, yielded, LiveApiError};

use super::{CHAT_CLEAN_CLOSE_REASON, CHAT_TRANSIENT_FAILURE_REASON};

fn script_lookup(fakes: &ChatFakes) {
    fakes
        .lookup
        .script_lookup(Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string())));
}

fn network_failure() -> LiveApiError {
    LiveApiError::Network {
        reason: "network-fixture".to_string(),
    }
}

fn run_worker_to_output_file(fakes: &ChatFakes, path: &Path) {
    let channel = chat_channel();
    let harness = collector_with_output_file(path);
    let mut plan = worker_plan(fakes, &channel);
    plan.diagnostics = Some(harness.collector.clone());
    spawn_chat_worker(plan)
        .join()
        .expect("the worker thread joins");
}

#[test]
fn clean_close_reconnect_rows_carry_the_clean_close_label() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let fakes = ChatFakes::new();
    script_lookup(&fakes);
    fakes.script_token_batch_session();
    fakes.script_token_batch_session();
    fakes.script_graceful_end_session();

    run_worker_to_output_file(&fakes, &path);

    let content = std::fs::read_to_string(&path).expect("reconnect rows must be written");
    let labelled_row = format!("\"backoff_ms\":1000,\"reason\":\"{CHAT_CLEAN_CLOSE_REASON}\"}}");
    assert_eq!(content.matches(labelled_row.as_str()).count(), 2);
    let batch_rows: Vec<&str> = content
        .lines()
        .filter(|line| line.contains("\"event\":\"queue\""))
        .filter(|line| !line.contains("\"batch_size\":null"))
        .collect();
    assert_eq!(batch_rows.len(), 2);
    assert!(batch_rows.iter().all(|line| !line.contains("\"reason\"")));
}

#[test]
fn transient_failure_reconnect_rows_carry_the_transient_failure_label() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let fakes = ChatFakes::new();
    script_lookup(&fakes);
    fakes.stream_ledger.script_open_failure(network_failure());
    fakes.script_graceful_end_session();

    run_worker_to_output_file(&fakes, &path);

    let content = std::fs::read_to_string(&path).expect("reconnect rows must be written");
    let labelled_row =
        format!("\"backoff_ms\":1000,\"reason\":\"{CHAT_TRANSIENT_FAILURE_REASON}\"}}");
    assert_eq!(content.matches(labelled_row.as_str()).count(), 1);
}

#[test]
fn other_queue_rows_carry_no_reason_key() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let fakes = ChatFakes::new();
    script_lookup(&fakes);
    fakes.script_token_batch_session();
    fakes
        .stream_ledger
        .script_session(vec![Some(Ok(yielded(&response_plain(vec![text_item(
            "m-1",
        )]))))]);
    fakes.script_graceful_end_session();

    run_worker_to_output_file(&fakes, &path);

    let content = std::fs::read_to_string(&path).expect("queue rows must be written");
    assert!(content.contains("\"event\":\"queue\""));
    let other_queue_rows: Vec<&str> = content
        .lines()
        .filter(|line| line.contains("\"event\":\"queue\""))
        .filter(|line| line.contains("\"backoff_ms\":null"))
        .collect();
    assert!(other_queue_rows.len() >= 2);
    assert!(other_queue_rows
        .iter()
        .all(|line| !line.contains("\"reason\"")));
}

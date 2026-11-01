//! Worker-side diagnostics pins: the delivery path reports batch receipt,
//! enqueue success, full-channel retention, and reconnect backoff into the
//! shared collector, and the terminal helpers stay reachable through the
//! delivery re-exports. Offline only, zero sleeps (the test slice is a
//! no-op).

use std::sync::Mutex;

use crate::app::diagnostics::diagnostics_test_fixtures::{
    enabled_collector_for_tests, TestCollector,
};
use crate::app::run::chat_worker::runtime_chat_fakes::{
    worker_plan, ChatFakes, LIVE_CHAT_ID_FIXTURE,
};
use crate::app::run::chat_worker::{chat_channel, spawn_chat_worker, CHAT_CHANNEL_CAPACITY};
use crate::chat::ChatMessage;
use crate::youtube::{response_offline, response_plain, text_item, yielded, LiveApiError};

use super::{record_terminal, transport_terminal, CHAT_WORKER_PANIC_DETAIL};

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

#[test]
fn worker_reports_batch_received_and_enqueue_success() {
    let fakes = ChatFakes::new();
    script_lookup(&fakes);
    fakes.script_token_batch_session();
    fakes.stream_ledger.script_session(vec![
        Some(Ok(yielded(&response_plain(vec![text_item("m-1")])))),
        Some(Ok(yielded(&response_offline(vec![])))),
    ]);
    let channel = chat_channel();
    let harness = enabled_collector_for_tests();
    let mut plan = worker_plan(&fakes, &channel);
    plan.diagnostics = Some(harness.collector.clone());

    spawn_chat_worker(plan)
        .join()
        .expect("the worker thread joins");

    let snapshot = harness
        .state
        .lock()
        .expect("diagnostics state")
        .snapshot(harness.now());
    assert_eq!(snapshot.received_total, 1);
    assert_eq!(snapshot.enqueued_total, 1);
    assert_eq!(snapshot.full_retry_total, 0);
    assert_eq!(snapshot.queue_depth_estimate, 1);
}

#[test]
fn worker_marks_backpressured_on_full_channel() {
    let fakes = ChatFakes::new();
    script_lookup(&fakes);
    fakes.script_token_batch_session();
    fakes.stream_ledger.script_session(vec![
        Some(Ok(yielded(&response_plain(vec![text_item("burst-1")])))),
        Some(Ok(yielded(&response_offline(vec![])))),
    ]);
    let channel = chat_channel();
    for index in 0..CHAT_CHANNEL_CAPACITY {
        let filler = ChatMessage::new(&format!("prefill-{index}"), "capacity");
        channel
            .sender
            .send(filler)
            .expect("the fresh channel holds the prefill");
    }
    let harness = enabled_collector_for_tests();
    let mut plan = worker_plan(&fakes, &channel);
    plan.diagnostics = Some(harness.collector.clone());
    let handle = spawn_chat_worker(plan);

    let full_retries = wait_for_full_retry(&harness);
    assert!(full_retries >= 1, "the full channel retries");
    let mut drained = 0;
    while drained < CHAT_CHANNEL_CAPACITY + 1 {
        channel
            .receiver
            .recv()
            .expect("the retained message arrives");
        drained += 1;
    }
    handle.join().expect("the worker thread joins");

    let snapshot = harness
        .state
        .lock()
        .expect("diagnostics state")
        .snapshot(harness.now());
    assert_eq!(snapshot.enqueued_total, 1);
    assert_eq!(snapshot.worker_retained_total, 1);
}

fn wait_for_full_retry(harness: &TestCollector) -> u64 {
    for _attempt in 0..1_000_000 {
        let retries = harness
            .state
            .lock()
            .expect("diagnostics state")
            .snapshot(harness.now())
            .full_retry_total;
        if retries > 0 {
            return retries;
        }
        std::thread::yield_now();
    }
    0
}

#[test]
fn worker_reports_reconnect_with_backoff() {
    let fakes = ChatFakes::new();
    script_lookup(&fakes);
    fakes.stream_ledger.script_open_failure(network_failure());
    fakes.script_graceful_end_session();
    let channel = chat_channel();
    let harness = enabled_collector_for_tests();
    let mut plan = worker_plan(&fakes, &channel);
    plan.diagnostics = Some(harness.collector.clone());

    spawn_chat_worker(plan)
        .join()
        .expect("the worker thread joins");

    let snapshot = harness
        .state
        .lock()
        .expect("diagnostics state")
        .snapshot(harness.now());
    assert_eq!(snapshot.reconnect_count, 1);
    assert_eq!(snapshot.reconnect_backoff_total_ms, 1_000);
}

#[test]
fn terminal_helpers_still_exported_from_delivery() {
    let cell = Mutex::new(None);
    record_terminal(&cell, transport_terminal("fixture-detail".to_string()));
    assert!(cell
        .lock()
        .expect("terminal cell")
        .as_ref()
        .is_some_and(|error| error.to_string().contains("fixture-detail")));
    assert!(CHAT_WORKER_PANIC_DETAIL.contains("panicked"));
}

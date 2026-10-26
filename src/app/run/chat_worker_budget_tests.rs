//! Roster B — the chat worker's boundedness/cancellation contract (plan
//! §5.2b): transient budget exhaustion and recovery, the pure backoff
//! pins, stop-before-transport, stop-during-delivery and the blocked
//! in-flight call. Offline only; tests never sleep except the deliberate
//! 250 ms stall fixture.

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::app::run::chat_worker::runtime_chat_fakes::{
    chat_page, text_message_item, worker_plan, ScriptedRunChatTransport, LIVE_CHAT_ID_FIXTURE,
};
use crate::youtube::LiveApiError;

use super::{chat_channel, run_chat_worker, spawn_chat_worker, CHAT_BACKOFF_MAX_MS};
use crate::app::run::chat_worker_delivery::backoff_delay_ms;

const STALLED_LIST_MILLIS: u64 = 250;
const JOIN_WALL_BOUND_SECONDS: u64 = 5;

fn lookup_found(fake: &ScriptedRunChatTransport) {
    fake.script_lookup(Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string())));
}

fn network_failure(attempt: u32) -> LiveApiError {
    LiveApiError::Network {
        reason: format!("network-fixture-{attempt}"),
    }
}

#[test]
fn worker_transient_failures_exhaust_the_budget_into_a_transport_terminal() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    for attempt in 1..=10 {
        fake.script_list(Err(network_failure(attempt)));
    }
    let channel = chat_channel();
    run_chat_worker(worker_plan(fake, &channel));

    let failure = channel
        .terminal
        .lock()
        .expect("cell mutex")
        .take()
        .expect("the budget exhausts into a terminal");
    assert!(matches!(
        failure,
        crate::app::pipeline::ChatRetrievalError::Transport { ref detail }
            if detail == "unable to reach the YouTube API: network-fixture-10"
    ));
}

#[test]
fn worker_transient_failures_recover_within_the_budget() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.script_list(Err(network_failure(1)));
    fake.script_list(Err(network_failure(2)));
    fake.script_list(Ok(chat_page(
        vec![text_message_item("m1", "recovered", "back online", None)],
        None,
        None,
        Some("2026-10-26T02:00:00Z"),
    )));
    let channel = chat_channel();
    run_chat_worker(worker_plan(fake, &channel));

    let terminal = channel.terminal.lock().expect("cell mutex").take();
    assert!(terminal.is_none(), "recovery inside the budget is clean");
    let received: Vec<String> = drain(&channel.receiver);
    assert!(
        received.is_empty(),
        "the recovery page is the baseline page: discarded by design"
    );
}

#[test]
fn worker_backoff_delay_doubles_and_caps_at_the_maximum() {
    assert_eq!(backoff_delay_ms(1, 10_000), 10_000);
    assert_eq!(backoff_delay_ms(2, 10_000), 20_000);
    assert_eq!(backoff_delay_ms(7, 10_000), CHAT_BACKOFF_MAX_MS);
    assert_eq!(backoff_delay_ms(1, 120_000), CHAT_BACKOFF_MAX_MS);
}

#[test]
fn worker_stop_flag_ends_the_loop_before_any_transport_call() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    let channel = chat_channel();
    let plan = worker_plan(fake.clone(), &channel);
    plan.stop_flag.request_stop();
    run_chat_worker(plan);

    assert!(
        fake.operations().is_empty(),
        "a pre-set stop flag ends the worker before any transport call"
    );
}

#[test]
fn worker_stop_during_delivery_drops_pending_messages_without_blocking() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.script_list(Ok(chat_page(vec![], Some("token-1"), None, None)));
    fake.script_list(Ok(chat_page(
        vec![text_message_item("m1", "alice", "too late", None)],
        None,
        None,
        None,
    )));
    let channel = chat_channel();
    let plan = worker_plan(fake.clone(), &channel);
    fake.flip_stop_on_list_from(2, plan.stop_flag.clone());
    run_chat_worker(plan);

    assert!(
        drain(&channel.receiver).is_empty(),
        "the stop flag drops the pending page without blocking"
    );
    let terminal = channel.terminal.lock().expect("cell mutex").take();
    assert!(terminal.is_none());
}

#[test]
fn worker_blocked_list_call_cannot_prevent_bounded_shutdown() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.stall_list_calls_for(STALLED_LIST_MILLIS);
    fake.script_list(Ok(chat_page(vec![], Some("token-1"), None, None)));
    let channel = chat_channel();
    let plan = worker_plan(fake, &channel);
    let stop_flag = plan.stop_flag.clone();
    let handle = spawn_chat_worker(plan);

    // Land the flip while the worker is inside the stalled list call.
    std::thread::sleep(Duration::from_millis(STALLED_LIST_MILLIS / 2));
    stop_flag.request_stop();
    let started = Instant::now();
    handle.join().expect("the worker thread joins");
    let elapsed = started.elapsed();
    assert!(
        elapsed <= Duration::from_secs(JOIN_WALL_BOUND_SECONDS),
        "a stalled in-flight call cannot hang shutdown: {elapsed:?}"
    );
}

fn drain(receiver: &std::sync::mpsc::Receiver<crate::chat::ChatMessage>) -> Vec<String> {
    let mut rendered = Vec::new();
    while let Ok(message) = receiver.try_recv() {
        rendered.push(message.to_string());
    }
    rendered
}

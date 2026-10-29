//! Roster B — the stream worker's boundedness/cancellation contract (plan §E.3/E.5), incl. both 250 ms stall pins. Offline only.

use std::time::{Duration, Instant};

use crate::app::pipeline::ChatRetrievalError;
use crate::app::run::chat_stream_delivery::backoff_delay_ms;
use crate::app::run::chat_worker::runtime_chat_fakes::{run_worker, worker_plan, ChatFakes};
use crate::youtube::{
    response_offline, response_plain, response_with_token, text_item, yielded, LiveApiError,
    RESUME_TOKEN_FIXTURE,
};

use super::chat_worker_tests::{drain, lookup_found, terminal_of};
use super::{
    chat_channel, run_chat_worker, spawn_chat_worker, CHAT_BACKOFF_MAX_MS, CHAT_SLEEP_SLICE_MS,
};

const STALLED_MILLIS: u64 = 250;
const JOIN_WALL_BOUND_SECONDS: u64 = 5;

fn network_failure(attempt: u32) -> LiveApiError {
    LiveApiError::Network {
        reason: format!("network-fixture-{attempt}"),
    }
}

#[test]
fn worker_transient_failures_exhaust_the_budget_into_a_transport_terminal() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    for attempt in 1..=10 {
        fakes
            .stream_ledger
            .script_open_failure(network_failure(attempt));
    }
    let channel = run_worker(&fakes);

    let failure = terminal_of(&channel).expect("the budget exhausts into a terminal");
    let ChatRetrievalError::Transport { detail } = failure else {
        panic!("the exhausted budget is the Transport class");
    };
    assert_eq!(
        detail,
        "unable to reach the YouTube API: network-fixture-10"
    );
    assert_eq!(
        fakes.stream_ledger.operations().len(),
        10,
        "the budget bounds the establishments; never periodic"
    );
}

#[test]
fn worker_transient_failures_recover_within_the_budget() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.stream_ledger.script_open_failure(network_failure(1));
    fakes.stream_ledger.script_open_failure(network_failure(2));
    fakes.script_graceful_end_session();
    let channel = run_worker(&fakes);

    assert!(
        terminal_of(&channel).is_none(),
        "recovery inside the budget is clean"
    );
    assert!(
        drain(&channel.receiver).is_empty(),
        "the recovery session is the baseline session: discarded by design"
    );
}

#[test]
fn worker_backoff_delay_doubles_and_caps_at_the_maximum() {
    assert_eq!(backoff_delay_ms(1, CHAT_SLEEP_SLICE_MS), 1_000);
    assert_eq!(backoff_delay_ms(2, CHAT_SLEEP_SLICE_MS), 2_000);
    assert_eq!(backoff_delay_ms(1, 10_000), 10_000);
    assert_eq!(backoff_delay_ms(2, 10_000), 20_000);
    assert_eq!(backoff_delay_ms(7, 10_000), CHAT_BACKOFF_MAX_MS);
    assert_eq!(backoff_delay_ms(1, 120_000), CHAT_BACKOFF_MAX_MS);
}

#[test]
fn worker_stop_flag_ends_the_loop_before_any_transport_open() {
    let fakes = ChatFakes::new();
    let channel = chat_channel();
    let plan = worker_plan(&fakes, &channel);
    plan.stop_flag.request_stop();
    run_chat_worker(plan);

    assert!(
        fakes.operations().is_empty(),
        "a pre-set stop flag ends the worker before any lookup"
    );
    assert!(
        fakes.stream_ledger.operations().is_empty(),
        "no establishment happens under a pre-set stop flag"
    );
}

#[test]
fn worker_stop_during_delivery_drops_pending_messages_without_blocking() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_token_batch_session();
    fakes
        .stream_ledger
        .script_session(vec![Some(Ok(yielded(&response_plain(vec![text_item(
            "m1",
        )]))))]);
    let channel = chat_channel();
    let plan = worker_plan(&fakes, &channel);
    let stop_flag = plan.stop_flag.clone();
    fakes
        .stream_ledger
        .flip_stop_on_open_from(2, move || stop_flag.request_stop());
    run_chat_worker(plan);

    assert!(
        drain(&channel.receiver).is_empty(),
        "the stop flag drops the pending message without blocking"
    );
    assert!(terminal_of(&channel).is_none());
}

#[test]
fn worker_stalled_stream_cannot_prevent_a_bounded_join() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.stream_ledger.stall_next_batch_for(STALLED_MILLIS);
    fakes.stream_ledger.script_session(vec![]);
    let channel = chat_channel();
    let plan = worker_plan(&fakes, &channel);
    let stop_flag = plan.stop_flag.clone();
    let handle = spawn_chat_worker(plan);

    // Land the flip while the worker is inside the stalled read.
    std::thread::sleep(Duration::from_millis(STALLED_MILLIS / 2));
    stop_flag.request_stop();
    let started = Instant::now();
    handle.join().expect("the worker thread joins");
    let elapsed = started.elapsed();
    assert!(
        elapsed <= Duration::from_secs(JOIN_WALL_BOUND_SECONDS),
        "a stalled read cannot hang shutdown: {elapsed:?}"
    );
}

#[test]
fn worker_never_reestablishes_after_a_graceful_end() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_graceful_end_session();
    let channel = run_worker(&fakes);

    assert!(terminal_of(&channel).is_none());
    assert_eq!(
        fakes.stream_ledger.operations(),
        ["open"],
        "a graceful end never re-establishes"
    );
}

#[test]
fn worker_delivery_after_a_stalled_read_adds_no_poll_interval() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.stream_ledger.stall_next_batch_for(STALLED_MILLIS);
    fakes.stream_ledger.script_session(vec![
        Some(Ok(yielded(&response_with_token(
            vec![],
            RESUME_TOKEN_FIXTURE,
        )))),
        Some(Ok(yielded(&response_plain(vec![text_item("m-1")])))),
        Some(Ok(yielded(&response_offline(vec![])))),
    ]);
    let started = Instant::now();
    let channel = run_worker(&fakes);
    let elapsed = started.elapsed();

    let received = drain(&channel.receiver);
    assert_eq!(received.len(), 1, "the live message arrives");
    assert!(format!("{:?}", received[0]).contains("m-1"));
    assert!(
        elapsed >= Duration::from_millis(STALLED_MILLIS / 2),
        "the stall really happened: {elapsed:?}"
    );
    assert!(
        elapsed < Duration::from_secs(1),
        "no poll interval is added after arrival: {elapsed:?}"
    );
}

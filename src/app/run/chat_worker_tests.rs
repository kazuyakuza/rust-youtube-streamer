//! Roster A (plan §E.1; items 1–4 of the retired flow roster live in the
//! `chat_worker_flow_tests` sibling): the stream worker's graceful ends,
//! terminal classes, the O6 resolution budget and the pinned info-event
//! constants. Offline only; the shared `runtime_chat_fakes`.

use crate::app::pipeline::ChatRetrievalError;
use crate::app::run::chat_worker::runtime_chat_fakes::{
    run_worker, ChatFakes, LIVE_CHAT_ID_FIXTURE,
};
use crate::chat::ChatMessage;
use crate::youtube::{response_offline, response_with_token, yielded, LiveApiError};

use super::{ChatChannel, CHAT_BASELINE_MESSAGE, CHAT_ENDED_MESSAGE, CHAT_ID_RESOLVED_MESSAGE};

pub(super) fn lookup_found(fakes: &ChatFakes) {
    fakes
        .lookup
        .script_lookup(Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string())));
}

pub(super) fn drain(receiver: &std::sync::mpsc::Receiver<ChatMessage>) -> Vec<ChatMessage> {
    let mut drained = Vec::new();
    while let Ok(message) = receiver.try_recv() {
        drained.push(message);
    }
    drained
}

pub(super) fn terminal_of(channel: &ChatChannel) -> Option<ChatRetrievalError> {
    channel.terminal.lock().expect("cell mutex").take()
}

#[test]
fn worker_offline_ended_batch_disconnects_without_terminal_error() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes
        .stream_ledger
        .script_session(vec![Some(Ok(yielded(&response_offline(vec![]))))]);
    let channel = run_worker(&fakes);

    assert!(terminal_of(&channel).is_none(), "offlineAt is graceful");
    drop(channel.sender);
    assert!(matches!(
        channel.receiver.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Disconnected)
    ));
}

#[test]
fn worker_mid_session_precondition_is_graceful() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.stream_ledger.script_session(vec![
        Some(Ok(yielded(&response_with_token(
            vec![],
            crate::youtube::RESUME_TOKEN_FIXTURE,
        )))),
        Some(Err(LiveApiError::ChatPrecondition {
            detail: "LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED".to_string(),
        })),
    ]);
    let channel = run_worker(&fakes);

    assert!(
        terminal_of(&channel).is_none(),
        "a mid-session precondition is the graceful end"
    );
    assert_eq!(
        fakes.stream_ledger.operations(),
        ["open"],
        "the failure landed inside the ONE established session"
    );
}

#[test]
fn worker_establishment_precondition_writes_the_chat_unavailable_terminal() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes
        .stream_ledger
        .script_open_failure(LiveApiError::ChatPrecondition {
            detail: "LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED".to_string(),
        });
    let channel = run_worker(&fakes);

    let failure = terminal_of(&channel).expect("an establishment precondition is terminal");
    assert_eq!(
        failure.to_string(),
        "live chat is unavailable: live chat precondition failed \
         (reason 'LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED')"
    );
}

#[test]
fn worker_resolution_absence_after_budget_maps_chat_unavailable() {
    let fakes = ChatFakes::new();
    for _attempt in 0..6 {
        fakes.lookup.script_lookup(Ok(None));
    }
    let channel = run_worker(&fakes);

    let failure = terminal_of(&channel).expect("absence after budget is terminal");
    let ChatRetrievalError::ChatUnavailable { detail } = failure else {
        panic!("absence after the O6 budget is the ChatUnavailable class");
    };
    assert_eq!(
        detail,
        "YouTube live chat not found; deleted or never created"
    );
    assert!(
        fakes
            .operations()
            .iter()
            .all(|op| *op == "lookup_broadcast_chat_id"),
        "no stream open happens without a chat id"
    );
    assert!(fakes.stream_ledger.operations().is_empty());
}

#[test]
fn worker_lookup_ending_failure_writes_the_chat_unavailable_terminal() {
    let fakes = ChatFakes::new();
    fakes.lookup.script_lookup(Err(LiveApiError::ChatDisabled));
    let channel = run_worker(&fakes);

    let failure = terminal_of(&channel).expect("ChatDisabled aborts the resolution");
    let ChatRetrievalError::ChatUnavailable { detail } = failure else {
        panic!("the ending failure maps into ChatUnavailable");
    };
    assert_eq!(
        detail,
        "live chat is disabled for this broadcast; enable chat and reuse its id"
    );
    assert!(fakes.stream_ledger.operations().is_empty());
}

#[test]
fn worker_event_constants_are_pinned_and_secret_free() {
    assert_eq!(CHAT_ID_RESOLVED_MESSAGE, "live chat id resolved");
    assert_eq!(
        CHAT_BASELINE_MESSAGE,
        "live chat baseline established; the history page was discarded"
    );
    assert_eq!(
        CHAT_ENDED_MESSAGE,
        "live chat ended; the stream continues without new chat messages"
    );
    for rendered in [
        CHAT_ID_RESOLVED_MESSAGE,
        CHAT_BASELINE_MESSAGE,
        CHAT_ENDED_MESSAGE,
    ] {
        assert!(!rendered.contains("rtmps"), "no RTMPS in: {rendered}");
        assert!(!rendered.contains("Bearer"), "no Bearer in: {rendered}");
        assert!(!rendered.contains("token"), "no token in: {rendered}");
    }
}

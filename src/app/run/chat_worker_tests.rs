//! Roster A (plan §5.2 items 5–10; items 1–4 live in the
//! `chat_worker_flow_tests` sibling per AMENDMENT 1 ruling 3): graceful
//! ends, terminal classes, the resolution budget and the pinned
//! info-event constants. Offline only; the shared Mutex-based
//! `runtime_chat_fakes`.

use std::sync::Arc;

use crate::app::pipeline::ChatRetrievalError;
use crate::app::run::chat_worker::runtime_chat_fakes::{
    chat_page, worker_plan, ScriptedRunChatTransport, LIVE_CHAT_ID_FIXTURE,
};
use crate::chat::ChatMessage;

use super::{chat_channel, run_chat_worker, ChatChannel};
use super::{CHAT_BASELINE_MESSAGE, CHAT_ENDED_MESSAGE, CHAT_ID_RESOLVED_MESSAGE};

pub(super) const PUBLISHED_AT: &str = "2026-10-26T01:02:03Z";
pub(super) const OFFLINE_AT: &str = "2026-10-26T02:00:00Z";

pub(super) fn lookup_found(fake: &ScriptedRunChatTransport) {
    fake.script_lookup(Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string())));
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
fn worker_graceful_ended_page_disconnects_without_terminal_error() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.script_list(Ok(chat_page(vec![], None, None, Some(OFFLINE_AT))));
    let channel = chat_channel();
    run_chat_worker(worker_plan(fake, &channel));

    assert!(terminal_of(&channel).is_none(), "offlineAt is graceful");
    drop(channel.sender);
    assert!(matches!(
        channel.receiver.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Disconnected)
    ));
}

#[test]
fn worker_chat_ended_error_class_is_graceful() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.script_list(Err(crate::youtube::LiveApiError::ChatEnded));
    let channel = chat_channel();
    run_chat_worker(worker_plan(fake, &channel));

    assert!(terminal_of(&channel).is_none(), "ChatEnded is graceful");
    drop(channel.sender);
    assert!(matches!(
        channel.receiver.try_recv(),
        Err(std::sync::mpsc::TryRecvError::Disconnected)
    ));
}

#[test]
fn worker_chat_disabled_failure_writes_the_chat_unavailable_terminal() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.script_list(Err(crate::youtube::LiveApiError::ChatDisabled));
    let channel = chat_channel();
    run_chat_worker(worker_plan(fake, &channel));

    let failure = terminal_of(&channel).expect("ChatDisabled is terminal");
    assert!(matches!(
        failure,
        ChatRetrievalError::ChatUnavailable { ref detail }
            if detail == "live chat is disabled for this broadcast; enable chat and reuse its id"
    ));
}

#[test]
fn worker_resolution_absence_after_budget_maps_chat_unavailable() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    for _attempt in 0..6 {
        fake.script_lookup(Ok(None));
    }
    let channel = chat_channel();
    run_chat_worker(worker_plan(Arc::clone(&fake), &channel));

    let failure = terminal_of(&channel).expect("absence after budget is terminal");
    assert!(matches!(
        failure,
        ChatRetrievalError::ChatUnavailable { ref detail }
            if detail == "YouTube live chat not found; deleted or never created"
    ));
    assert!(
        fake.operations()
            .iter()
            .all(|op| *op == "lookup_broadcast_chat_id"),
        "no list call happens without a chat id"
    );
}

#[test]
fn worker_rate_limit_exceeded_is_transient_while_quota_exhaustion_is_terminal() {
    let recovering = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&recovering);
    recovering.script_list(Err(crate::youtube::LiveApiError::QuotaLike {
        reason: "rateLimitExceeded".to_string(),
    }));
    recovering.script_list(Err(crate::youtube::LiveApiError::QuotaLike {
        reason: "rateLimitExceeded".to_string(),
    }));
    recovering.script_list(Ok(chat_page(vec![], None, None, Some(OFFLINE_AT))));
    let channel = chat_channel();
    run_chat_worker(worker_plan(recovering, &channel));
    assert!(
        terminal_of(&channel).is_none(),
        "rateLimitExceeded recovers within the budget"
    );

    let exhausted = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&exhausted);
    exhausted.script_list(Err(crate::youtube::LiveApiError::QuotaLike {
        reason: "quotaExceeded".to_string(),
    }));
    let channel = chat_channel();
    run_chat_worker(worker_plan(exhausted, &channel));
    let failure = terminal_of(&channel).expect("quotaExceeded is terminal");
    assert!(matches!(failure, ChatRetrievalError::Quota { .. }));
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

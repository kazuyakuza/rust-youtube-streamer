//! Roster A flow half (plan §E.2): the live delivery path — API order and
//! metadata, whole-batch history discard, cross-session dedup, verbatim
//! token progression, the pre-baseline failed-connect refinement and the
//! malformed-batch transient discard. Offline only.

use crate::app::run::chat_worker::runtime_chat_fakes::{run_worker, ChatFakes};
use crate::chat::ChatMessage;
use crate::youtube::{
    response_offline, response_plain, text_item, yielded, LiveApiError, RESUME_TOKEN_FIXTURE,
};

use super::chat_worker_tests::{drain, lookup_found, terminal_of};

const PINNED_PUBLISHED_EPOCH: u64 = 1_792_931_445;

#[test]
fn worker_delivers_live_batches_in_api_order_with_metadata() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_history_batch_session();
    fakes.stream_ledger.script_session(vec![
        Some(Ok(yielded(&response_plain(vec![
            text_item("m1"),
            text_item("m2"),
        ])))),
        Some(Ok(yielded(&response_offline(vec![])))),
    ]);
    let channel = run_worker(&fakes);

    let received = drain(&channel.receiver);
    assert_eq!(received.len(), 2, "both live messages arrive in order");
    assert_eq!(received[0].to_string(), "author-fixture: text-fixture");
    assert_eq!(received[1].to_string(), "author-fixture: text-fixture");
    assert!(format!("{:?}", received[0]).contains("m1"));
    assert!(format!("{:?}", received[1]).contains("m2"));
    assert_eq!(received[1].timestamp(), Some(PINNED_PUBLISHED_EPOCH));
}

#[test]
fn worker_discards_the_entire_history_batch_before_the_first_token() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_history_batch_session();
    fakes
        .stream_ledger
        .script_session(vec![Some(Ok(yielded(&response_offline(vec![text_item(
            "m-1",
        )]))))]);
    let channel = run_worker(&fakes);

    let received = drain(&channel.receiver);
    assert_eq!(received.len(), 1, "the history batch is discarded whole");
    assert!(format!("{:?}", received[0]).contains("m-1"));
    assert!(
        terminal_of(&channel).is_none(),
        "a discarded batch is not an error"
    );
}

#[test]
fn worker_dedup_drops_replayed_ids_across_sessions() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_token_batch_session();
    fakes
        .stream_ledger
        .script_session(vec![Some(Ok(yielded(&response_plain(vec![text_item(
            "m1",
        )]))))]);
    fakes
        .stream_ledger
        .script_session(vec![Some(Ok(yielded(&response_offline(vec![
            text_item("m1"),
            text_item("m2"),
        ]))))]);
    let channel = run_worker(&fakes);

    let received = drain(&channel.receiver);
    let rendered: Vec<String> = received.iter().map(ChatMessage::to_string).collect();
    assert_eq!(
        rendered,
        [
            "author-fixture: text-fixture",
            "author-fixture: text-fixture"
        ],
        "the replayed id is dropped; the window survived the reconnect"
    );
    assert!(format!("{:?}", received[0]).contains("m1"));
    assert!(format!("{:?}", received[1]).contains("m2"));
    assert!(terminal_of(&channel).is_none());
}

#[test]
fn worker_token_progresses_verbatim_without_inference() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_token_batch_session();
    fakes.script_graceful_end_session();
    let channel = run_worker(&fakes);

    let opened = fakes.stream_ledger.opened_requests();
    assert_eq!(opened.len(), 2, "both establishments happened");
    assert_eq!(opened[0].page_token, None);
    assert_eq!(opened[1].page_token.as_deref(), Some(RESUME_TOKEN_FIXTURE));
    assert!(
        terminal_of(&channel).is_none(),
        "an empty batch never infers the end"
    );
}

#[test]
fn worker_failed_connect_before_the_first_batch_reestablishes_tokenless_then_discards_history() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes
        .stream_ledger
        .script_open_failure(LiveApiError::Network {
            reason: "network-fixture-1".to_string(),
        });
    fakes.script_history_batch_session();
    fakes.script_graceful_end_session();
    let channel = run_worker(&fakes);

    let opened = fakes.stream_ledger.opened_requests();
    assert_eq!(opened.len(), 3, "one failed connect plus two sessions");
    assert_eq!(opened[0].page_token, None);
    assert_eq!(
        opened[1].page_token, None,
        "the pre-baseline re-open is tokenless"
    );
    assert_eq!(opened[2].page_token.as_deref(), Some(RESUME_TOKEN_FIXTURE));
    assert!(terminal_of(&channel).is_none());
    assert!(
        drain(&channel.receiver).is_empty(),
        "the first delivered session is history and is discarded whole"
    );
}

#[test]
fn worker_malformed_batch_is_discarded_and_never_ordinary_chat() {
    let fakes = ChatFakes::new();
    lookup_found(&fakes);
    fakes.script_token_batch_session();
    fakes
        .stream_ledger
        .script_session(vec![Some(Err(LiveApiError::MalformedResponse {
            field: "stream response".to_string(),
        }))]);
    fakes
        .stream_ledger
        .script_session(vec![Some(Ok(yielded(&response_offline(vec![text_item(
            "m-1",
        )]))))]);
    let channel = run_worker(&fakes);

    let received = drain(&channel.receiver);
    assert_eq!(received.len(), 1, "the malformed batch never becomes chat");
    assert!(format!("{:?}", received[0]).contains("m-1"));
    assert!(terminal_of(&channel).is_none());
    assert_eq!(
        fakes.stream_ledger.operations(),
        ["open", "open", "open"],
        "only the scripted establishments, never a periodic call"
    );
}

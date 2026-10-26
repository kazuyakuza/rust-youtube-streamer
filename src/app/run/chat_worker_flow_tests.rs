//! Roster A flow half (plan §5.2 items 1–4, AMENDMENT 1 ruling 3): the
//! mapped-message delivery path — order/metadata, baseline discard,
//! cross-page dedup and token progression. Offline only; the shared
//! Mutex-based `runtime_chat_fakes`.

use std::sync::Arc;

use crate::app::run::chat_worker::runtime_chat_fakes::{
    chat_page, run_worker, text_message_item, ScriptedRunChatTransport,
};
use crate::chat::ChatMessage;
use crate::youtube::parse_rfc3339_seconds;

use super::chat_worker_tests::{drain, lookup_found, terminal_of, OFFLINE_AT, PUBLISHED_AT};

#[test]
fn worker_delivers_mapped_messages_in_page_order_with_metadata() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.script_list(Ok(chat_page(vec![], None, None, None)));
    fake.script_list(Ok(chat_page(
        vec![
            text_message_item("m1", "alice", "Hello there", None),
            text_message_item("m2", "bob", "line one\nline two", Some(PUBLISHED_AT)),
        ],
        None,
        None,
        None,
    )));
    fake.script_list(Ok(chat_page(vec![], None, None, Some(OFFLINE_AT))));
    let channel = run_worker(fake);

    let received = drain(&channel.receiver);
    assert_eq!(received.len(), 2, "both live messages arrive in order");
    assert_eq!(received[0].to_string(), "alice: Hello there");
    assert_eq!(received[1].to_string(), "bob: line one line two");
    let rendered = format!("{:?}", received[1]);
    assert!(
        rendered.contains("m2"),
        "the provider id arrives verbatim: {rendered}"
    );
    let expected_epoch = parse_rfc3339_seconds(PUBLISHED_AT).expect("the fixture parses");
    assert_eq!(received[1].timestamp(), Some(expected_epoch));
    assert_eq!(received[0].timestamp(), None);
}

#[test]
fn worker_discards_the_entire_baseline_history_page() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.script_list(Ok(chat_page(
        vec![text_message_item("h1", "history", "old message", None)],
        Some("history-token"),
        None,
        None,
    )));
    fake.script_list(Ok(chat_page(
        vec![text_message_item("m1", "live", "new message", None)],
        None,
        None,
        Some(OFFLINE_AT),
    )));
    let channel = run_worker(fake);

    let received = drain(&channel.receiver);
    assert_eq!(received.len(), 1, "the baseline page is discarded whole");
    assert_eq!(received[0].to_string(), "live: new message");
    assert!(
        terminal_of(&channel).is_none(),
        "a discarded page is not an error"
    );
}

#[test]
fn worker_dedup_drops_replayed_ids_across_pages() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.script_list(Ok(chat_page(vec![], Some("baseline-token"), None, None)));
    fake.script_list(Ok(chat_page(
        vec![text_message_item("m1", "alice", "first", None)],
        Some("token-a"),
        None,
        None,
    )));
    fake.script_list(Ok(chat_page(
        vec![
            text_message_item("m1", "alice", "first replayed", None),
            text_message_item("m2", "bob", "second", None),
        ],
        None,
        None,
        Some(OFFLINE_AT),
    )));
    let channel = run_worker(fake);

    let received = drain(&channel.receiver);
    let rendered: Vec<String> = received.iter().map(ChatMessage::to_string).collect();
    assert_eq!(rendered, ["alice: first", "bob: second"]);
    assert!(terminal_of(&channel).is_none());
}

#[test]
fn worker_empty_page_advances_the_token_without_inference() {
    let fake = Arc::new(ScriptedRunChatTransport::default());
    lookup_found(&fake);
    fake.script_list(Ok(chat_page(vec![], Some("token-1-fixture"), None, None)));
    fake.script_list(Ok(chat_page(vec![], None, None, Some(OFFLINE_AT))));
    let channel = run_worker(Arc::clone(&fake));

    let commands = fake.commands();
    assert_eq!(commands.len(), 2, "both list calls happened");
    assert_eq!(commands[0].page_token, None);
    assert_eq!(
        commands[1].page_token.as_deref(),
        Some("token-1-fixture"),
        "emptiness never infers the end; the token advances verbatim"
    );
    assert!(terminal_of(&channel).is_none());
}

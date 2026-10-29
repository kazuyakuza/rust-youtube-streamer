//! Seam-contract pins for the mockable stream transport (plan §E.5): the
//! `Send` superbound, scripted session ordering with request capture, the
//! scripted batch → clean-close sequence, typed error pass-through and the
//! mapper-built carrier shape. Every async piece runs on the tiny
//! current-thread test runtime; no test waits on wall-clock time and nothing
//! dials out.

use super::*;
use crate::youtube::stream_test_fixtures::ScriptedChatStreamSession;
use crate::youtube::{
    response_plain, response_with_token, stream_test_block_on, text_item, yielded,
    ScriptedChatStreamTransport, ScriptedSessionSpec, LIVE_CHAT_ID_FIXTURE, RESUME_TOKEN_FIXTURE,
};

fn request(page_token: Option<&str>) -> ChatStreamRequest {
    ChatStreamRequest {
        live_chat_id: LIVE_CHAT_ID_FIXTURE.to_string(),
        page_token: page_token.map(str::to_string),
    }
}

#[test]
fn stream_seam_objects_stay_send_bounded() {
    let _transport: Box<dyn ChatStreamTransport> = Box::new(ScriptedChatStreamTransport::new());
    let _session: Box<dyn LiveChatStreamSession> = Box::new(ScriptedChatStreamSession::new_empty());
}

#[test]
fn fake_transport_opens_scripted_sessions_in_order() {
    let mut transport = ScriptedChatStreamTransport::new();
    transport.enqueue_session(Ok(ScriptedSessionSpec::new(vec![])));
    transport.enqueue_session(Ok(ScriptedSessionSpec::new(vec![])));
    let first = request(None);
    let second = request(Some(RESUME_TOKEN_FIXTURE));
    stream_test_block_on(async {
        transport.open(first).await.expect("first session opens");
        transport.open(second).await.expect("second session opens");
    });
    assert_eq!(transport.invokes(), vec!["open", "open"]);
    let captured = transport.captured();
    assert_eq!(captured.len(), 2);
    assert_eq!(captured[0].live_chat_id, LIVE_CHAT_ID_FIXTURE);
    assert!(captured[0].page_token.is_none());
    assert_eq!(
        captured[1].page_token.as_deref(),
        Some(RESUME_TOKEN_FIXTURE)
    );
}

#[test]
fn fake_transport_yields_scripted_batches_then_clean_close() {
    let mut transport = ScriptedChatStreamTransport::new();
    let batch = yielded(&response_plain(vec![
        text_item("msg-1"),
        text_item("msg-2"),
    ]));
    transport.enqueue_session(Ok(ScriptedSessionSpec::new(vec![Some(Ok(batch))])));
    let (first, second) = stream_test_block_on(async {
        let mut session = transport
            .open(request(None))
            .await
            .expect("the scripted session opens");
        let first = session.next_batch().await;
        let second = session.next_batch().await;
        (first, second)
    });
    let first = first.expect("one scripted batch").expect("the batch maps");
    assert_eq!(first.messages.len(), 2);
    assert!(second.is_none());
}

#[test]
fn fake_transport_reports_typed_errors_from_the_queue() {
    let mut open_transport = ScriptedChatStreamTransport::new();
    open_transport.enqueue_session(Err(LiveApiError::InsufficientPermissions));
    let open_result = stream_test_block_on(open_transport.open(request(None)));
    assert!(matches!(
        open_result,
        Err(LiveApiError::InsufficientPermissions)
    ));

    let mut delivery_transport = ScriptedChatStreamTransport::new();
    let failure = LiveApiError::ChatPrecondition {
        detail: "LIVE_CHAT_DISABLED-or-LIVE_CHAT_ENDED".to_string(),
    };
    delivery_transport.enqueue_session(Ok(ScriptedSessionSpec::new(vec![Some(Err(failure))])));
    let delivered = stream_test_block_on(async {
        let mut session = delivery_transport
            .open(request(None))
            .await
            .expect("the scripted session opens");
        session.next_batch().await.expect("one scripted delivery")
    });
    assert!(matches!(
        delivered,
        Err(LiveApiError::ChatPrecondition { .. })
    ));
}

#[test]
fn fake_session_carriers_match_the_real_mapper() {
    let batch = yielded(&response_with_token(
        vec![text_item("msg-1")],
        RESUME_TOKEN_FIXTURE,
    ));
    let mut transport = ScriptedChatStreamTransport::new();
    transport.enqueue_session(Ok(ScriptedSessionSpec::new(vec![Some(Ok(batch))])));
    let delivered = stream_test_block_on(async {
        let mut session = transport
            .open(request(None))
            .await
            .expect("the scripted session opens");
        session.next_batch().await.expect("one scripted batch")
    })
    .expect("the batch maps");
    assert_eq!(
        delivered.next_page_token.as_deref(),
        Some(RESUME_TOKEN_FIXTURE)
    );
    assert_eq!(delivered.messages.len(), 1);
    assert_eq!(delivered.messages[0].author, "author-fixture");
    assert!(!delivered.offline_ended);
}

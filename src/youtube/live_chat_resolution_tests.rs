//! O6 resolution pins (plan §5.3): scripted seam, ZERO network, the
//! caller-chosen test interval keeping every sleep at 0 ms — immediate
//! success, bounded absence, transient-consumes-attempt, and the terminal/
//! cancellation aborts that must never be retried. Offline only.

use std::cell::RefCell;
use std::collections::VecDeque;

use super::live_chat_test_fixtures::{
    ScriptedLiveChatTransport, BROADCAST_ID_FIXTURE, LIVE_CHAT_ID_FIXTURE,
};
use super::*;

const ZERO_INTERVAL_MS: u64 = 0;

fn scripted(lookups: Vec<Result<Option<String>, LiveApiError>>) -> ScriptedLiveChatTransport {
    ScriptedLiveChatTransport {
        lookup_results: RefCell::new(lookups.into()),
        list_results: RefCell::new(VecDeque::new()),
        operations: RefCell::new(Vec::new()),
    }
}

fn lookup_count(transport: &ScriptedLiveChatTransport) -> usize {
    transport.operations.borrow().len()
}

#[test]
fn resolution_returns_the_broadcast_live_chat_id() {
    let transport = scripted(vec![Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string()))]);
    let resolved = resolve_live_chat_id(&transport, BROADCAST_ID_FIXTURE)
        .expect("the scripted chat id resolves");
    assert_eq!(resolved, LIVE_CHAT_ID_FIXTURE);
    assert_eq!(lookup_count(&transport), 1);
}

#[test]
fn resolution_polls_until_the_chat_id_appears() {
    let transport = scripted(vec![
        Ok(None),
        Ok(None),
        Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string())),
    ]);
    let resolved =
        resolve_live_chat_id_with_interval(&transport, BROADCAST_ID_FIXTURE, ZERO_INTERVAL_MS)
            .expect("the chat id appears within the budget");
    assert_eq!(resolved, LIVE_CHAT_ID_FIXTURE);
    assert_eq!(
        lookup_count(&transport),
        3,
        "attempts stay below the budget"
    );
}

#[test]
fn resolution_absent_after_budget_is_typed_not_found() {
    let lookups: Vec<Result<Option<String>, LiveApiError>> = (0..6).map(|_| Ok(None)).collect();
    let transport = scripted(lookups);
    let error =
        resolve_live_chat_id_with_interval(&transport, BROADCAST_ID_FIXTURE, ZERO_INTERVAL_MS)
            .expect_err("the budget exhausts into a typed not-found");
    assert!(matches!(
        &error,
        LiveApiError::ApiNotFound { resource } if resource == "live chat"
    ));
    assert_eq!(lookup_count(&transport), 6, "exactly the bounded budget");
}

#[test]
fn resolution_transient_failures_consume_attempts_then_recover() {
    let transport = scripted(vec![
        Err(LiveApiError::Network {
            reason: "fixture outage".to_string(),
        }),
        Ok(Some(LIVE_CHAT_ID_FIXTURE.to_string())),
    ]);
    let resolved =
        resolve_live_chat_id_with_interval(&transport, BROADCAST_ID_FIXTURE, ZERO_INTERVAL_MS)
            .expect("the transient attempt recovers");
    assert_eq!(resolved, LIVE_CHAT_ID_FIXTURE);
    assert_eq!(lookup_count(&transport), 2);
}

#[test]
fn resolution_terminal_errors_abort_immediately() {
    let transport = scripted(vec![Err(LiveApiError::InsufficientPermissions)]);
    let error =
        resolve_live_chat_id_with_interval(&transport, BROADCAST_ID_FIXTURE, ZERO_INTERVAL_MS)
            .expect_err("a terminal class aborts");
    assert!(matches!(error, LiveApiError::InsufficientPermissions));
    assert_eq!(
        lookup_count(&transport),
        1,
        "terminal errors are not retried"
    );
}

#[test]
fn resolution_wait_cancelled_is_not_translucently_retried() {
    let transport = scripted(vec![Err(LiveApiError::WaitCancelled)]);
    let error =
        resolve_live_chat_id_with_interval(&transport, BROADCAST_ID_FIXTURE, ZERO_INTERVAL_MS)
            .expect_err("WaitCancelled propagates verbatim");
    assert!(matches!(error, LiveApiError::WaitCancelled));
    assert_eq!(lookup_count(&transport), 1, "cancellation is never retried");
}

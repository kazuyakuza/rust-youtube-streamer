//! Offline establishment pins (2026-10-30 readiness-reserve fix): the
//! incident anchor reproduces the PRE-FIX reserve-less call shape against a
//! dead plaintext loopback endpoint (the local panic is the pinned
//! contract), and the fix shape drives the production composition through
//! `open_with_endpoint` with the same endpoint (no unwind, typed Network
//! error). Offline only: no listener, no Google contact, no TLS.

use std::panic::{catch_unwind, AssertUnwindSafe};

use super::*;
use crate::youtube::stream_test_block_on;
use crate::youtube::ACCESS_TOKEN_FIXTURE;

const LOOPBACK_DEAD_URI: &str = "http://127.0.0.1:1";

const DEAD_ENDPOINT_STREAM_REASONS: [&str; 3] = [
    STREAM_CONNECT_TIMEOUT_REASON,
    STREAM_CONNECT_FAILURE_REASON,
    STREAM_READY_FAILURE_REASON,
];

/// The dead plaintext loopback endpoint: `http://` scheme (no TLS
/// configuration), the pinned 10 s connect bound for faithfulness, and a
/// dead port so no network I/O can complete anywhere in the roster.
fn dead_loopback_endpoint() -> tonic::transport::Endpoint {
    tonic::transport::Endpoint::from_shared(LOOPBACK_DEAD_URI.to_string())
        .expect("the plaintext dead loopback endpoint parses")
        .connect_timeout(Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS))
}

fn fixture_command() -> ChatStreamRequest {
    ChatStreamRequest {
        live_chat_id: crate::youtube::LIVE_CHAT_ID_FIXTURE.to_string(),
        page_token: None,
    }
}

/// The incident anchor (TODO Task-1 bullet (a), offline): the PRE-FIX call
/// shape — `Grpc::new(channel)` + `server_streaming(..)` WITHOUT the
/// readiness reserve — against a dead plaintext loopback endpoint. The
/// channel is built with the documented-lazy `connect_lazy` so the call
/// path (and its local panic) is reached with zero network I/O; the
/// reserve-less Buffer call panics locally exactly as the incident logged
/// (`send_item called without first calling poll_reserve`, tower-0.5.3
/// buffer/service.rs, operator-supplied `logs/run/20261030-211225.log`).
/// The literal panic text is deliberately NOT asserted (brittle across
/// tower patches); the anchor pins the unwinding itself, and the exact
/// text lives in the incident record / the plan's evidence block.
#[test]
fn establishment_anchor_unread_shape_panics_locally() {
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        stream_test_block_on(async {
            let endpoint = dead_loopback_endpoint();
            let channel = endpoint.connect_lazy();
            let mut grpc = tonic::client::Grpc::new(channel);
            let _ = grpc
                .server_streaming(
                    stream_request(
                        stream_list_request(&fixture_command()),
                        bearer_token_metadata(&AccessToken::new(ACCESS_TOKEN_FIXTURE))
                            .expect("the fixture token is header-safe"),
                    ),
                    PathAndQuery::from_static(STREAM_LIST_PATH),
                    tonic::codec::ProstCodec::default(),
                )
                .await
                .map(|response| TonicChatStreamSession::new(response.into_inner()));
        });
    }));
    assert!(
        outcome.is_err(),
        "the pre-fix shape must unwind locally without the readiness reserve"
    );
}

/// The fix shape: the EXACT production establishment composition
/// (bearer → bounded connect → `Grpc::new` → the readiness reserve →
/// `server_streaming`) driven through the private `open_with_endpoint`
/// helper with a dead plaintext loopback endpoint. Assert: NO unwinding
/// (the reserve honors the tower Buffer contract; the incident panic
/// cannot recur on this shape) and the outcome is a typed
/// `LiveApiError::Network` under the dead-endpoint stream reason set.
#[test]
fn establishment_fixed_shape_reserves_readiness_and_yields_typed_error() {
    let mut transport = GoogleLiveChatStreamTransport::new(AccessToken::new(ACCESS_TOKEN_FIXTURE));
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        stream_test_block_on(async {
            tokio::time::timeout(
                Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS),
                transport.open_with_endpoint(dead_loopback_endpoint(), fixture_command()),
            )
            .await
        })
    }));
    let attempt = outcome.expect("no panic occurs on the fixed shape");
    let attempt =
        attempt.expect("the dead loopback establishment completes within the pinned connect bound");
    let reason = match attempt {
        Err(LiveApiError::Network { reason }) => reason,
        Err(other) => panic!("a dead endpoint yields the Network class, got {other:?}"),
        Ok(_) => panic!("a dead endpoint cannot establish a stream session"),
    };
    assert!(
        DEAD_ENDPOINT_STREAM_REASONS.contains(&reason.as_str()),
        "the observed reason '{reason}' is a pinned dead-endpoint stream reason"
    );
}

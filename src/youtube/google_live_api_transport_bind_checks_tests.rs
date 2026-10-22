//! The three bodyless-POST wire regression tests (the HTTP 411 fix,
//! plan D7): (a) the bind POST arrives WITH `Content-Length: 0` and an
//! empty body and the Google-like enforcing server accepts it; (b) the
//! reproduced PRE-FIX lengthless bind POST is rejected 411 and
//! classifies as the observed incident shape; (c) the GET lookup stays
//! on the no-body `call()` form. Loopback `127.0.0.1` only; no test
//! reaches Google (D10).
//!
//! Plan: `.kilo/plans/20261022-fix-http-411-task1-implementation.md`;
//! TODO `.agent/todos/20261022/20261022-todo-1.md` Task 1.

use super::google_live_api_transport_bind_wire_tests::*;
use super::*;

fn fixture_bind_command() -> BindCommand {
    BindCommand {
        broadcast_id: BROADCAST_ID_FIXTURE.to_string(),
        stream_id: STREAM_ID_FIXTURE.to_string(),
    }
}

/// The D1 pin: the transport's bodyless bind POST transmits zero body
/// bytes WITH `Content-Length: 0`, which a Google-like enforcing server
/// accepts instead of answering 411 Length Required.
#[test]
fn bind_post_zero_content_length_is_accepted_by_enforcing_server() {
    let server = EnforcingBindServer::start(BOUND_BROADCAST_BODY);
    let broadcast = server
        .transport()
        .bind(&fixture_bind_command())
        .expect("the zero-length bind POST is accepted by the enforcing server");
    assert_eq!(broadcast.id.as_deref(), Some(BROADCAST_ID_FIXTURE));
    let served = server.served_request();
    assert!(served
        .request_line
        .starts_with("POST /youtube/v3/liveBroadcasts/bind?"));
    assert_eq!(served.content_length.as_deref(), Some("0"));
    assert!(served.body.is_empty(), "bind sends no body bytes");
    server.finish();
}

/// The incident anchor (TODO bullet (b)): after D1 no production code
/// path performs a lengthless POST, so the in-test reproduction is the
/// only honest wire-negative pairing (D4) — the pre-fix wire form
/// carries no `Content-Length`, the enforcing server answers 411, and
/// the transport's status arm classifies it into the exact observed
/// incident line.
#[test]
fn pre_fix_lengthless_bind_post_classifies_as_observed_411_shape() {
    let server = EnforcingBindServer::start(BOUND_BROADCAST_BODY);
    let url = format!(
        "http://127.0.0.1:{}/youtube/v3/liveBroadcasts/bind?id={}&streamId={}",
        server.port, BROADCAST_ID_FIXTURE, STREAM_ID_FIXTURE
    );
    let error = classify_through_status_arm(pre_fix_lengthless_call(&url));
    let LiveApiError::ApiRejected { status, reason } = &error else {
        panic!("expected ApiRejected, got {error}");
    };
    assert_eq!(*status, 411);
    assert_eq!(reason, "unspecified");
    assert_eq!(
        error.to_string(),
        "YouTube rejected request (HTTP 411, reason 'unspecified')"
    );
    let served = server.served_request();
    assert!(
        served.content_length.is_none(),
        "the reproduced pre-fix wire form carries no Content-Length"
    );
    server.finish();
}

/// The D2 pin: the GET form (verifier/O4/O5 class) still transmits NO
/// `Content-Length` header — no drift — and a Google-like server
/// accepts it, proving the enforcement rule is POST-specific.
#[test]
fn get_lookup_stays_lengthless_with_lengthless_body() {
    let server = EnforcingBindServer::start(r#"{"items":[]}"#);
    let lookup = server
        .transport()
        .lookup_stream(STREAM_ID_FIXTURE)
        .expect("a GET lookup needs no Content-Length");
    assert!(lookup.is_none());
    let served = server.served_request();
    assert!(served
        .request_line
        .starts_with("GET /youtube/v3/liveStreams?"));
    assert!(
        served.content_length.is_none(),
        "GET stays on the no-body call() form"
    );
    assert!(served.body.is_empty());
    server.finish();
}

//! Verifier pure-classifier tests (R1, R7-R9): the documented
//! endpoint-constant contract and the synthesized status→error mapping,
//! including the body-redaction guarantee. No sockets; fully offline.

use super::google_authorization_verifier_tests::{verification_failure, ACCESS_TOKEN_FIXTURE};
use super::*;

#[test]
fn verification_request_url_is_the_documented_channels_call() {
    assert_eq!(
        VERIFICATION_ENDPOINT,
        "https://www.googleapis.com/youtube/v3/channels?part=id&mine=true"
    );
}

#[test]
fn other_statuses_map_to_transport() {
    for status in [400_u16, 404, 500] {
        let error = verification_failure(classify_verification_reply(status, None));
        let VerificationError::Transport { reason } = &error else {
            panic!("expected the Transport class, got {error}");
        };
        assert!(reason.contains(&status.to_string()));
    }
}

#[test]
fn error_bodies_are_never_embedded_into_reasons() {
    let hostile = format!(
        r#"{{"error":{{"code":401,"message":"token {ACCESS_TOKEN_FIXTURE} was rejected","errors":[{{"reason":"authError"}}]}}}}"#
    );
    let error = verification_failure(classify_verification_reply(401, Some(&hostile)));
    let rendered = format!("{error}");
    assert!(rendered.contains("authError"));
    assert!(!rendered.contains(ACCESS_TOKEN_FIXTURE));
}

#[test]
fn unparsable_error_bodies_use_the_fixed_reason_text() {
    let error = verification_failure(classify_verification_reply(
        401,
        Some("Connection refused by the operator."),
    ));
    let VerificationError::NotAuthorized { reason } = &error else {
        panic!("expected the NotAuthorized class, got {error}");
    };
    assert_eq!(reason, REJECTED_TOKEN_TEXT);
}

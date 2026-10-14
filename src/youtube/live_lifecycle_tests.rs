//! State-machine tests for the staged lifecycle enums: every documented
//! label parses, unknown and mis-cased labels are typed contract
//! violations, and the readiness/pending predicates match the documented
//! states (plan §5 T2).

use super::*;

#[test]
fn every_documented_broadcast_label_parses() {
    assert_eq!(
        BroadcastLifecycle::from_api_label("created").expect("created"),
        BroadcastLifecycle::Created
    );
    assert_eq!(
        BroadcastLifecycle::from_api_label("ready").expect("ready"),
        BroadcastLifecycle::Ready
    );
    assert_eq!(
        BroadcastLifecycle::from_api_label("liveStarting").expect("liveStarting"),
        BroadcastLifecycle::LiveStarting
    );
    assert_eq!(
        BroadcastLifecycle::from_api_label("live").expect("live"),
        BroadcastLifecycle::Live
    );
    assert_eq!(
        BroadcastLifecycle::from_api_label("complete").expect("complete"),
        BroadcastLifecycle::Complete
    );
    assert_eq!(
        BroadcastLifecycle::from_api_label("revoked").expect("revoked"),
        BroadcastLifecycle::Revoked
    );
    assert_eq!(
        BroadcastLifecycle::from_api_label("testStarting").expect("testStarting"),
        BroadcastLifecycle::TestStarting
    );
    assert_eq!(
        BroadcastLifecycle::from_api_label("testing").expect("testing"),
        BroadcastLifecycle::Testing
    );
}

#[test]
fn every_documented_stream_label_parses() {
    assert_eq!(
        StreamStatusKind::from_api_label("active").expect("active"),
        StreamStatusKind::Active
    );
    assert_eq!(
        StreamStatusKind::from_api_label("created").expect("created"),
        StreamStatusKind::Created
    );
    assert_eq!(
        StreamStatusKind::from_api_label("error").expect("error"),
        StreamStatusKind::Error
    );
    assert_eq!(
        StreamStatusKind::from_api_label("inactive").expect("inactive"),
        StreamStatusKind::Inactive
    );
    assert_eq!(
        StreamStatusKind::from_api_label("ready").expect("ready"),
        StreamStatusKind::Ready
    );
}

#[test]
fn unknown_broadcast_label_is_malformed_response() {
    let error = BroadcastLifecycle::from_api_label("bogusLabel")
        .expect_err("an undocumented label must fail typed");
    let LiveApiError::MalformedResponse { field } = &error else {
        panic!("expected MalformedResponse, got {error}");
    };
    assert_eq!(field, "status.lifeCycleStatus");
}

#[test]
fn unknown_stream_label_is_malformed_response() {
    let error = StreamStatusKind::from_api_label("bogusLabel")
        .expect_err("an undocumented label must fail typed");
    let LiveApiError::MalformedResponse { field } = &error else {
        panic!("expected MalformedResponse, got {error}");
    };
    assert_eq!(field, "status.streamStatus");
}

#[test]
fn only_starting_states_are_transition_pending() {
    assert!(BroadcastLifecycle::LiveStarting.is_transition_pending());
    assert!(BroadcastLifecycle::TestStarting.is_transition_pending());
    for state in [
        BroadcastLifecycle::Created,
        BroadcastLifecycle::Ready,
        BroadcastLifecycle::Live,
        BroadcastLifecycle::Complete,
        BroadcastLifecycle::Revoked,
        BroadcastLifecycle::Testing,
    ] {
        assert!(!state.is_transition_pending());
    }
}

#[test]
fn only_ready_is_streamable_setup() {
    assert!(BroadcastLifecycle::Ready.is_streamable_setup());
    for state in [
        BroadcastLifecycle::Created,
        BroadcastLifecycle::LiveStarting,
        BroadcastLifecycle::Live,
        BroadcastLifecycle::Complete,
        BroadcastLifecycle::Revoked,
        BroadcastLifecycle::TestStarting,
        BroadcastLifecycle::Testing,
    ] {
        assert!(!state.is_streamable_setup());
    }
}

#[test]
fn labels_parse_case_sensitively() {
    assert!(BroadcastLifecycle::from_api_label("READY").is_err());
    assert!(BroadcastLifecycle::from_api_label("ready ").is_err());
    assert!(StreamStatusKind::from_api_label("ACTIVE").is_err());
    assert!(StreamStatusKind::from_api_label("ready ").is_err());
}

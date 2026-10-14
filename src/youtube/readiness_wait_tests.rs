//! Readiness-wait loop tests (plan §4 roster C, wait-loop subset; the
//! confirmation subset lives in `readiness_wait_confirmation_tests`,
//! Amendment 2): bounded O4+O5 polling, typed terminal failures, the
//! transient matrix and the cancellation seam. Fakes come from
//! `live_test_fixtures` (Amendment 1); no network, no Google.

use crate::youtube::live_models::LiveBroadcast;
use crate::youtube::live_test_fixtures::*;

use super::*;

type StreamLookupOutcome = Result<Option<LiveStream>, LiveApiError>;
type BroadcastLookupOutcome = Result<Option<LiveBroadcast>, LiveApiError>;

/// Failure extraction without `expect_err` (`LiveStream` is Debug-free).
fn wait_failure(outcome: Result<LiveStream, LiveApiError>) -> LiveApiError {
    let Err(error) = outcome else {
        panic!("expected a readiness wait failure");
    };
    error
}

fn queued_stream(transport: &ScriptedLiveApiTransport, outcome: StreamLookupOutcome) {
    let queue = &transport.lookup_stream_results;
    queue.borrow_mut().push_back(outcome);
}

fn queued_broadcast(transport: &ScriptedLiveApiTransport, outcome: BroadcastLookupOutcome) {
    let queue = &transport.lookup_broadcast_results;
    queue.borrow_mut().push_back(outcome);
}

fn scripted_wait<'a>(
    transport: &'a ScriptedLiveApiTransport,
    canceller: &'a WaitCanceller,
) -> ReadinessWait<'a> {
    ReadinessWait::with_polling(
        transport,
        STREAM_ID_FIXTURE,
        BROADCAST_ID_FIXTURE,
        canceller,
        3,
        0,
    )
}

#[test]
fn both_statuses_ready_returns_the_observed_stream() {
    let transport = ScriptedLiveApiTransport::default();
    queued_stream(&transport, Ok(Some(confirmed_ready_stream())));
    queued_broadcast(&transport, Ok(Some(broadcast_with_lifecycle("ready"))));
    let canceller = WaitCanceller::new();
    let confirmed = scripted_wait(&transport, &canceller)
        .run()
        .expect("both statuses observed ready");
    assert_eq!(confirmed.id.as_deref(), Some(STREAM_ID_FIXTURE));
    assert_eq!(
        *transport.operations.borrow(),
        ["lookup_stream", "lookup_broadcast"]
    );
}

#[test]
fn never_ready_times_out_as_readiness_with_stage() {
    let transport = ScriptedLiveApiTransport::default();
    for _attempt in 0..2 {
        queued_stream(&transport, Ok(Some(not_ready_stream())));
        queued_broadcast(&transport, Ok(Some(not_ready_broadcast())));
    }
    let canceller = WaitCanceller::new();
    let wait = ReadinessWait::with_polling(
        &transport,
        STREAM_ID_FIXTURE,
        BROADCAST_ID_FIXTURE,
        &canceller,
        2,
        0,
    );
    let error = wait_failure(wait.run());
    assert!(matches!(error, LiveApiError::ReadinessTimeout { stage } if stage == "readiness"));
    assert_eq!(transport.operations.borrow().len(), 4);
}

#[test]
fn cancel_during_wait_ends_with_wait_cancelled() {
    let transport = ScriptedLiveApiTransport::default();
    queued_stream(&transport, Ok(Some(not_ready_stream())));
    queued_broadcast(&transport, Ok(Some(not_ready_broadcast())));
    let canceller = WaitCanceller::new();
    *transport.canceller_flip.borrow_mut() = Some(canceller.clone());
    transport.cancel_after_broadcast_lookups.set(1);
    let wait = ReadinessWait::with_polling(
        &transport,
        STREAM_ID_FIXTURE,
        BROADCAST_ID_FIXTURE,
        &canceller,
        5,
        0,
    );
    let error = wait_failure(wait.run());
    assert!(matches!(error, LiveApiError::WaitCancelled));
    assert_eq!(
        *transport.operations.borrow(),
        ["lookup_stream", "lookup_broadcast"],
        "extra poll cycles must never run after the flip"
    );
}

#[test]
fn stream_error_terminates_immediately_as_broadcast_state() {
    let transport = ScriptedLiveApiTransport::default();
    queued_stream(&transport, Ok(Some(stream_with_status("error"))));
    let canceller = WaitCanceller::new();
    let error = wait_failure(scripted_wait(&transport, &canceller).run());
    assert!(
        matches!(error, LiveApiError::BroadcastState { detail } if detail == "streamStatus error")
    );
    assert_eq!(*transport.operations.borrow(), ["lookup_stream"]);
}

#[test]
fn broadcast_revoked_terminates_immediately_as_broadcast_state() {
    let transport = ScriptedLiveApiTransport::default();
    queued_stream(&transport, Ok(Some(not_ready_stream())));
    queued_broadcast(&transport, Ok(Some(broadcast_with_lifecycle("revoked"))));
    let canceller = WaitCanceller::new();
    let error = wait_failure(scripted_wait(&transport, &canceller).run());
    assert!(
        matches!(error, LiveApiError::BroadcastState { detail } if detail == "lifeCycleStatus revoked")
    );
}

#[test]
fn vanished_stream_lookup_none_is_api_not_found() {
    let transport = ScriptedLiveApiTransport::default();
    queued_stream(&transport, Ok(None));
    let canceller = WaitCanceller::new();
    let error = wait_failure(scripted_wait(&transport, &canceller).run());
    assert!(matches!(error, LiveApiError::ApiNotFound { resource } if resource == "liveStream"));
    assert_eq!(*transport.operations.borrow(), ["lookup_stream"]);
}

#[test]
fn vanished_broadcast_lookup_none_is_api_not_found() {
    let transport = ScriptedLiveApiTransport::default();
    queued_stream(&transport, Ok(Some(not_ready_stream())));
    queued_broadcast(&transport, Ok(None));
    let canceller = WaitCanceller::new();
    let error = wait_failure(scripted_wait(&transport, &canceller).run());
    assert!(matches!(error, LiveApiError::ApiNotFound { resource } if resource == "liveBroadcast"));
    assert_eq!(transport.operations.borrow().len(), 2);
}

#[test]
fn unknown_stream_label_is_malformed_response() {
    let transport = ScriptedLiveApiTransport::default();
    queued_stream(
        &transport,
        Ok(Some(stream_with_status("bogus-label-fixture"))),
    );
    let canceller = WaitCanceller::new();
    let error = wait_failure(scripted_wait(&transport, &canceller).run());
    assert!(
        matches!(error, LiveApiError::MalformedResponse { field } if field == "status.streamStatus")
    );
}

#[test]
fn pre_cancelled_wait_makes_zero_api_calls() {
    let transport = ScriptedLiveApiTransport::default();
    let canceller = WaitCanceller::new();
    canceller.request_cancel();
    let error = wait_failure(scripted_wait(&transport, &canceller).run());
    assert!(matches!(error, LiveApiError::WaitCancelled));
    assert!(transport.operations.borrow().is_empty());
}

#[test]
fn transient_backend_error_consumes_an_attempt_inside_the_budget() {
    let transport = ScriptedLiveApiTransport::default();
    queued_stream(
        &transport,
        Err(LiveApiError::BackendUnavailable {
            reason: "backendError".to_string(),
        }),
    );
    queued_stream(&transport, Ok(Some(confirmed_ready_stream())));
    queued_broadcast(&transport, Ok(Some(broadcast_with_lifecycle("ready"))));
    let canceller = WaitCanceller::new();
    scripted_wait(&transport, &canceller)
        .run()
        .expect("the transient failure is tolerated");
    assert_eq!(
        *transport.operations.borrow(),
        ["lookup_stream", "lookup_stream", "lookup_broadcast"],
        "the failed cycle consumed one attempt and the retry succeeded"
    );
}

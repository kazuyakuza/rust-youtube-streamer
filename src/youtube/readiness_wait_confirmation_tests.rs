//! Confirmation-wait tests (plan §4 roster C, confirmation subset; the
//! wait-loop subset lives in the `readiness_wait_tests` sibling per
//! Amendment 2): the O4-only `active` gate, the confirmation-stage
//! timeout, and the O4-only profile assertion — all against the shared
//! scripted transport fake (`live_test_fixtures`, Amendment 1). Pure
//! fakes; no network, no Google.

use crate::youtube::live_test_fixtures::*;

use super::*;

fn observation_pair(canceller: &WaitCanceller) -> StreamObservationRequest<'_> {
    StreamObservationRequest {
        stream_id: STREAM_ID_FIXTURE,
        cancellation: canceller,
    }
}

#[test]
fn confirmation_wait_returns_when_stream_active() {
    let transport = ScriptedLiveApiTransport::default();
    queued(
        &transport.lookup_stream_results,
        Ok(Some(stream_with_status("inactive"))),
    );
    queued(
        &transport.lookup_stream_results,
        Ok(Some(stream_with_status("active"))),
    );
    let canceller = WaitCanceller::new();
    wait_until_stream_active_with_polling(&transport, &observation_pair(&canceller), 3, 0)
        .expect("the stream becomes active");
    assert_eq!(
        *transport.operations.borrow(),
        ["lookup_stream", "lookup_stream"],
        "the confirmation wait is O4-only"
    );
}

#[test]
fn confirmation_timeout_carries_the_confirmation_stage() {
    let transport = ScriptedLiveApiTransport::default();
    for _attempt in 0..3 {
        queued(
            &transport.lookup_stream_results,
            Ok(Some(stream_with_status("inactive"))),
        );
    }
    let canceller = WaitCanceller::new();
    let error =
        wait_until_stream_active_with_polling(&transport, &observation_pair(&canceller), 3, 0)
            .expect_err("the confirmation budget exhausts");
    assert!(
        matches!(error, LiveApiError::ReadinessTimeout { stage } if stage == "ingestion-confirmation")
    );
}

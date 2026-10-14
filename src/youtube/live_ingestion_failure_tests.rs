//! Orchestrator failure tests (plan §4 roster B, per-step failure
//! subset; the happy-path subset lives in the `live_ingestion_tests`
//! sibling): the per-step missing-id/bind-policy failures, the typed
//! error propagation out of each stage (each its own named test, roster
//! item 8 counting as three) and the confirmed-stream destination-
//! missing mappings — all against the shared scripted transport fake
//! (`live_test_fixtures`, Amendment 1). No network, no Google.

use serde_json::json;

use crate::youtube::live_test_fixtures::*;

use super::live_ingestion_tests::*;
use super::*;

const OTHER_STREAM_ID_FIXTURE: &str = "other-stream-id-fixture";

#[test]
fn stream_resource_without_id_is_malformed() {
    let transport = ScriptedLiveApiTransport::default();
    queued(
        &transport.create_stream_results,
        Ok(stream_from_json(json!({}))),
    );
    let error = preparation_failure(preparer().prepare(&transport, &WaitCanceller::new()));
    assert!(matches!(
        error,
        LiveApiError::MalformedResponse { field } if field == "id"
    ));
    assert_eq!(*transport.operations.borrow(), ["create_stream"]);
}

#[test]
fn broadcast_resource_without_id_is_malformed() {
    let transport = ScriptedLiveApiTransport::default();
    queued(&transport.create_stream_results, Ok(inserted_stream()));
    queued(
        &transport.create_broadcast_results,
        Ok(broadcast_from_json(json!({}))),
    );
    let error = preparation_failure(preparer().prepare(&transport, &WaitCanceller::new()));
    assert!(matches!(
        error,
        LiveApiError::MalformedResponse { field } if field == "id"
    ));
    assert_eq!(
        *transport.operations.borrow(),
        ["create_stream", "create_broadcast"]
    );
}

#[test]
fn bind_without_bound_stream_id_is_malformed() {
    let transport = ScriptedLiveApiTransport::default();
    queued(&transport.create_stream_results, Ok(inserted_stream()));
    queued(
        &transport.create_broadcast_results,
        Ok(inserted_broadcast()),
    );
    queued(&transport.bind_results, Ok(inserted_broadcast()));
    let error = preparation_failure(preparer().prepare(&transport, &WaitCanceller::new()));
    assert!(matches!(
        error,
        LiveApiError::MalformedResponse { field } if field == "contentDetails.boundStreamId"
    ));
}

#[test]
fn bind_to_a_different_stream_is_broadcast_state() {
    let transport = ScriptedLiveApiTransport::default();
    queued(&transport.create_stream_results, Ok(inserted_stream()));
    queued(
        &transport.create_broadcast_results,
        Ok(inserted_broadcast()),
    );
    queued(
        &transport.bind_results,
        Ok(broadcast_from_json(json!({
            "id": BROADCAST_ID_FIXTURE,
            "contentDetails": {"boundStreamId": OTHER_STREAM_ID_FIXTURE}
        }))),
    );
    let error = preparation_failure(preparer().prepare(&transport, &WaitCanceller::new()));
    assert!(matches!(
        error,
        LiveApiError::BroadcastState { detail } if detail == "boundStreamId mismatch"
    ));
}

#[test]
fn invalid_metadata_propagates_from_stream_creation() {
    let transport = ScriptedLiveApiTransport::default();
    queued(
        &transport.create_stream_results,
        Err(LiveApiError::InvalidMetadata {
            field: "snippet.title".to_string(),
        }),
    );
    let error = preparation_failure(preparer().prepare(&transport, &WaitCanceller::new()));
    assert!(matches!(
        error,
        LiveApiError::InvalidMetadata { field } if field == "snippet.title"
    ));
    assert_eq!(*transport.operations.borrow(), ["create_stream"]);
}

#[test]
fn quota_propagates_from_broadcast_creation() {
    let transport = ScriptedLiveApiTransport::default();
    queued(&transport.create_stream_results, Ok(inserted_stream()));
    queued(
        &transport.create_broadcast_results,
        Err(LiveApiError::QuotaLike {
            reason: "userRequestsExceedRateLimit".to_string(),
        }),
    );
    let error = preparation_failure(preparer().prepare(&transport, &WaitCanceller::new()));
    assert!(matches!(
        error,
        LiveApiError::QuotaLike { reason } if reason == "userRequestsExceedRateLimit"
    ));
    assert_eq!(
        *transport.operations.borrow(),
        ["create_stream", "create_broadcast"],
        "bind is never called after a failed insert"
    );
}

#[test]
fn insufficient_permissions_propagate_from_bind() {
    let transport = ScriptedLiveApiTransport::default();
    queued(&transport.create_stream_results, Ok(inserted_stream()));
    queued(
        &transport.create_broadcast_results,
        Ok(inserted_broadcast()),
    );
    queued(
        &transport.bind_results,
        Err(LiveApiError::InsufficientPermissions),
    );
    let error = preparation_failure(preparer().prepare(&transport, &WaitCanceller::new()));
    assert!(matches!(error, LiveApiError::InsufficientPermissions));
}

#[test]
fn missing_rtmps_address_in_confirmed_stream_is_destination_missing() {
    let transport = ScriptedLiveApiTransport::default();
    queued(&transport.create_stream_results, Ok(inserted_stream()));
    queued(
        &transport.create_broadcast_results,
        Ok(inserted_broadcast()),
    );
    queued(&transport.bind_results, Ok(bound_broadcast()));
    queued(
        &transport.lookup_stream_results,
        Ok(Some(stream_from_json(json!({
            "id": STREAM_ID_FIXTURE,
            "status": {"streamStatus": "ready"},
            "cdn": {"ingestionInfo": {"streamName": STREAM_NAME_FIXTURE}}
        })))),
    );
    queued(
        &transport.lookup_broadcast_results,
        Ok(Some(broadcast_with_lifecycle("ready"))),
    );
    let error = preparation_failure(preparer().prepare(&transport, &WaitCanceller::new()));
    assert!(matches!(
        error,
        LiveApiError::DestinationMissing { field } if field == "cdn.ingestionInfo.rtmpsIngestionAddress"
    ));
}

#[test]
fn missing_stream_name_in_confirmed_stream_is_destination_missing() {
    let transport = ScriptedLiveApiTransport::default();
    queued(&transport.create_stream_results, Ok(inserted_stream()));
    queued(
        &transport.create_broadcast_results,
        Ok(inserted_broadcast()),
    );
    queued(&transport.bind_results, Ok(bound_broadcast()));
    queued(
        &transport.lookup_stream_results,
        Ok(Some(stream_from_json(json!({
            "id": STREAM_ID_FIXTURE,
            "status": {"streamStatus": "ready"},
            "cdn": {"ingestionInfo": {"rtmpsIngestionAddress": INGEST_BASE_FIXTURE}}
        })))),
    );
    queued(
        &transport.lookup_broadcast_results,
        Ok(Some(broadcast_with_lifecycle("ready"))),
    );
    let error = preparation_failure(preparer().prepare(&transport, &WaitCanceller::new()));
    assert!(matches!(
        error,
        LiveApiError::DestinationMissing { field } if field == "cdn.ingestionInfo.streamName"
    ));
}

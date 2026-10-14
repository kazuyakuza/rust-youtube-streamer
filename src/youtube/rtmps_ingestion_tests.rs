//! RTMPS-destination extraction tests (plan §4 roster D): the exact
//! documented concatenation, the no-fallback rule, the layered
//! missing-field failures and the redaction contract. Fixtures only; no
//! network, no Google.

use serde_json::json;

use crate::youtube::live_test_fixtures::*;

use super::*;

fn stream_with_cdn(cdn: serde_json::Value) -> LiveStream {
    stream_from_json(json!({"id": STREAM_ID_FIXTURE, "cdn": cdn}))
}

/// Failure extraction without `expect_err` (`RtmpsIngestion` is
/// Debug-free by the redaction contract).
fn extraction_failure(stream: &LiveStream) -> LiveApiError {
    let Err(error) = RtmpsIngestion::from_stream(stream) else {
        panic!("expected a destination extraction failure");
    };
    error
}

#[test]
fn assembles_exactly_address_slash_stream_name() {
    let stream = stream_with_cdn(json!({"ingestionInfo": {
        "streamName": STREAM_NAME_FIXTURE,
        "rtmpsIngestionAddress": INGEST_BASE_FIXTURE
    }}));
    let ingestion =
        RtmpsIngestion::from_stream(&stream).expect("the documented fields are present");
    assert_eq!(
        ingestion.into_url(),
        format!("{INGEST_BASE_FIXTURE}/{STREAM_NAME_FIXTURE}")
    );
}

#[test]
fn absent_rtmps_address_never_falls_back_to_plain_ingestion_address() {
    let stream = stream_with_cdn(json!({"ingestionInfo": {
        "streamName": STREAM_NAME_FIXTURE,
        "ingestionAddress": "rtmp://ingest.example.test/ingest"
    }}));
    let error = extraction_failure(&stream);
    assert!(
        matches!(error, LiveApiError::DestinationMissing { field } if field == "cdn.ingestionInfo.rtmpsIngestionAddress")
    );
}

#[test]
fn absent_stream_name_is_destination_missing() {
    let stream = stream_with_cdn(json!({"ingestionInfo": {
        "rtmpsIngestionAddress": INGEST_BASE_FIXTURE
    }}));
    let error = extraction_failure(&stream);
    assert!(
        matches!(error, LiveApiError::DestinationMissing { field } if field == "cdn.ingestionInfo.streamName")
    );
}

#[test]
fn absent_cdn_and_absent_ingestion_info_map_in_layers() {
    let missing_cdn = stream_from_json(json!({"id": STREAM_ID_FIXTURE}));
    let error = extraction_failure(&missing_cdn);
    assert!(matches!(error, LiveApiError::DestinationMissing { field } if field == "cdn"));

    let missing_info = stream_with_cdn(json!({"ingestionType": "rtmp"}));
    let error = extraction_failure(&missing_info);
    assert!(
        matches!(error, LiveApiError::DestinationMissing { field } if field == "cdn.ingestionInfo")
    );
}

#[test]
fn debug_is_exactly_the_redacted_marker() {
    let stream = stream_with_cdn(json!({"ingestionInfo": {
        "streamName": STREAM_NAME_FIXTURE,
        "rtmpsIngestionAddress": INGEST_BASE_FIXTURE
    }}));
    let ingestion =
        RtmpsIngestion::from_stream(&stream).expect("the documented fields are present");
    assert_eq!(format!("{ingestion:?}"), "RtmpsIngestion(<redacted>)");
}

#[test]
fn into_url_consumes_and_returns_the_exact_url() {
    let stream = stream_with_cdn(json!({"ingestionInfo": {
        "streamName": STREAM_NAME_FIXTURE,
        "rtmpsIngestionAddress": INGEST_BASE_FIXTURE
    }}));
    let ingestion =
        RtmpsIngestion::from_stream(&stream).expect("the documented fields are present");
    let revealed = ingestion.into_url();
    assert_eq!(
        revealed,
        format!("{INGEST_BASE_FIXTURE}/{STREAM_NAME_FIXTURE}")
    );
}

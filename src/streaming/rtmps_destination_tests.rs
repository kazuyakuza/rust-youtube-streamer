//! Destination-wrapper tests: the `Debug` output is exactly the redacted
//! marker and the URL is exposed only through the streaming-sibling
//! accessor (plan D-T3-11 surface 3).

use super::*;

const FIXTURE_URL: &str = "rtmps://ingest.example.test/ingest/stream-key-fixture";

#[test]
fn destination_debug_is_exactly_the_redacted_marker() {
    let destination = RtmpsDestination::new(FIXTURE_URL.to_string());

    assert_eq!(format!("{destination:?}"), "RtmpsDestination(<redacted>)");
}

#[test]
fn destination_as_url_exposes_the_assembled_url_to_streaming_siblings() {
    let destination = RtmpsDestination::new(FIXTURE_URL.to_string());

    assert_eq!(destination.as_url(), FIXTURE_URL);
}

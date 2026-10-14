//! Offline model tests for the staged Live API resource types: documented
//! field extraction, unknown-field tolerance, empty/missing-part pages,
//! the locked config→CDN mapping and the exact insert-body JSON shapes
//! (plan §5 T1). Fixtures are fake-shaped; no test contacts Google.

use serde_json::json;

use super::*;

fn video_config(width: u32, height: u32, fps: u32) -> VideoConfig {
    VideoConfig {
        width,
        height,
        fps,
        pixel_format: "rgb24".to_string(),
    }
}

fn broadcast_page_json() -> serde_json::Value {
    json!({
        "items": [
            {
                "id": "broadcast-id-fixture",
                "snippet": {
                    "title": "Fixture broadcast",
                    "description": "Fixture description",
                    "scheduledStartTime": "2026-10-14T12:00:00Z"
                },
                "status": {
                    "lifeCycleStatus": "ready",
                    "privacyStatus": "unlisted",
                    "recordingStatus": "notRecording"
                },
                "contentDetails": {
                    "boundStreamId": "stream-id-fixture",
                    "enableAutoStart": true
                }
            }
        ]
    })
}

#[test]
fn broadcast_page_item_deserializes_documented_fields() {
    let page: LiveBroadcastList =
        serde_json::from_value(broadcast_page_json()).expect("documented fields parse");
    let item = page.items.first().expect("one item");
    assert_eq!(item.id.as_deref(), Some("broadcast-id-fixture"));
    let status = item.status.as_ref().expect("status present");
    assert_eq!(status.life_cycle_status.as_deref(), Some("ready"));
    assert_eq!(status.privacy_status.as_deref(), Some("unlisted"));
    let details = item.content_details.as_ref().expect("contentDetails");
    assert_eq!(details.bound_stream_id.as_deref(), Some("stream-id-fixture"));
}

#[test]
fn broadcast_page_ignores_unknown_fields() {
    let mut page = broadcast_page_json();
    page["etag"] = json!("etag-fixture");
    page["items"][0]["monetizationDetails"] = json!({"enabled": false});
    let parsed: LiveBroadcastList =
        serde_json::from_value(page).expect("unknown fields must be ignored");
    assert_eq!(parsed.items.len(), 1);
}

#[test]
fn stream_page_item_deserializes_ingestion_info_all_five_fields() {
    let page: LiveStreamList = serde_json::from_value(json!({
        "items": [
            {
                "id": "stream-id-fixture",
                "status": {
                    "streamStatus": "ready",
                    "healthStatus": {"status": "good"}
                },
                "cdn": {
                    "ingestionType": "rtmp",
                    "ingestionInfo": {
                        "streamName": "stream-name-fixture",
                        "ingestionAddress": "rtmp://a.example.test/fixture",
                        "backupIngestionAddress": "rtmp://b.example.test/fixture",
                        "rtmpsIngestionAddress": "rtmps://a.example.test/fixture",
                        "rtmpsBackupIngestionAddress": "rtmps://b.example.test/fixture"
                    }
                }
            }
        ]
    }))
    .expect("all five ingestion fields parse");
    let stream = page.items.first().expect("one item");
    let ingestion = stream
        .cdn
        .as_ref()
        .expect("cdn present")
        .ingestion_info
        .as_ref()
        .expect("ingestionInfo present");
    assert_eq!(ingestion.stream_name.as_deref(), Some("stream-name-fixture"));
    assert_eq!(
        ingestion.ingestion_address.as_deref(),
        Some("rtmp://a.example.test/fixture")
    );
    assert_eq!(
        ingestion.backup_ingestion_address.as_deref(),
        Some("rtmp://b.example.test/fixture")
    );
    assert_eq!(
        ingestion.rtmps_ingestion_address.as_deref(),
        Some("rtmps://a.example.test/fixture")
    );
    assert_eq!(
        ingestion.rtmps_backup_ingestion_address.as_deref(),
        Some("rtmps://b.example.test/fixture")
    );
}

#[test]
fn empty_items_fall_back_to_default() {
    let broadcasts: LiveBroadcastList =
        serde_json::from_value(json!({})).expect("empty body parses");
    assert!(broadcasts.items.is_empty());
    let streams: LiveStreamList =
        serde_json::from_value(json!({})).expect("empty body parses");
    assert!(streams.items.is_empty());
}

#[test]
fn missing_parts_yield_none_fields() {
    let page: LiveBroadcastList = serde_json::from_value(json!({"items": [{"id": "x"}]}))
        .expect("a response missing status parses");
    let item = page.items.first().expect("one item");
    assert!(item.status.is_none());
    assert!(item.snippet.is_none());
    assert!(item.content_details.is_none());
}

#[test]
fn stream_cdn_maps_known_resolution_and_frame_rate() {
    let cdn = StreamCreationCdn::from_video_config(&video_config(1920, 1080, 30));
    assert_eq!(cdn.ingestion_type, StreamCreationCdn::RTMP_INGESTION_TYPE);
    assert_eq!(cdn.resolution, "1080p");
    assert_eq!(cdn.frame_rate, "30fps");
}

#[test]
fn stream_cdn_maps_unknown_size_to_variable_pair() {
    let cdn = StreamCreationCdn::from_video_config(&video_config(123, 456, 24));
    assert_eq!(cdn.resolution, "variable");
    assert_eq!(cdn.frame_rate, "variable");
}

#[test]
fn create_commands_serialize_documented_shape() {
    let broadcast_command = CreateBroadcastCommand {
        title: "Fixture broadcast".to_string(),
        description: "Fixture description".to_string(),
        privacy_status: "unlisted".to_string(),
        scheduled_start_time: "2026-10-14T12:01:00Z".to_string(),
    };
    let body = BroadcastRequestBody::from_command(&broadcast_command);
    assert_eq!(
        serde_json::to_value(&body).expect("body serializes"),
        json!({
            "snippet": {
                "title": "Fixture broadcast",
                "description": "Fixture description",
                "scheduledStartTime": "2026-10-14T12:01:00Z"
            },
            "contentDetails": {
                "enableAutoStart": true,
                "enableAutoStop": true,
                "monitorStream": {"enableMonitorStream": false}
            },
            "status": {"privacyStatus": "unlisted"}
        })
    );

    let stream_command = CreateStreamCommand {
        title: "Fixture stream".to_string(),
        resolution: "1080p".to_string(),
        frame_rate: "30fps".to_string(),
    };
    let stream_body = StreamRequestBody::from_command(&stream_command);
    assert_eq!(
        serde_json::to_value(&stream_body).expect("body serializes"),
        json!({
            "snippet": {"title": "Fixture stream"},
            "cdn": {
                "ingestionType": "rtmp",
                "resolution": "1080p",
                "frameRate": "30fps"
            }
        })
    );
}

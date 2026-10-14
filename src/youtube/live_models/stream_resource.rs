//! `liveStreams` resource views (O1/O4). See the parent `live_models`
//! module for the contract: documented fields only, unknown fields
//! ignored, missing parts `None`.
//!
//! Grounding: `id`, `cdn.{ingestionType,ingestionInfo}`, the five
//! `cdn.ingestionInfo` address/name fields and `status.{streamStatus,
//! healthStatus}` are the documented fields of
//! https://developers.google.com/youtube/v3/live/docs/liveStreams;
//! `rtmpsIngestionAddress` is the FFmpeg destination source per
//! https://developers.google.com/youtube/v3/live/guides/rtmps-ingestion
//! (the destination is exactly `rtmpsIngestionAddress`/`streamName`).
//!
//! Staging: dead until Task 2's concrete transport deserializes a
//! `liveStreams` response through these views.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.3, F1); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 1.

use serde::Deserialize;

/// A `liveStreams.list` page (O4); `items` defaults to empty.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveStreamList {
    #[serde(default)]
    pub(crate) items: Vec<LiveStream>,
}

/// One live-stream resource view (O1/O4); documented fields only.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveStream {
    pub(crate) id: Option<String>,
    pub(crate) status: Option<StreamStatus>,
    pub(crate) cdn: Option<StreamCdn>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamCdn {
    pub(super) ingestion_type: Option<String>,
    pub(crate) ingestion_info: Option<IngestionInfo>,
}

/// The documented ingestion-address family (never real keys in fixtures).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IngestionInfo {
    pub(crate) stream_name: Option<String>,
    pub(super) ingestion_address: Option<String>,
    pub(super) backup_ingestion_address: Option<String>,
    pub(crate) rtmps_ingestion_address: Option<String>,
    pub(super) rtmps_backup_ingestion_address: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamStatus {
    pub(crate) stream_status: Option<String>,
    pub(super) health_status: Option<StreamHealth>,
}

#[derive(Deserialize)]
pub(crate) struct StreamHealth {
    #[serde(rename = "status")]
    overall: Option<String>,
}

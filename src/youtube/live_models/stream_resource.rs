//! `liveStreams` resource views (O1/O4). See the parent `live_models`
//! module for the contract: documented fields only, unknown fields
//! ignored, missing parts `None`.

use serde::Deserialize;

/// A `liveStreams.list` page (O4); `items` defaults to empty.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveStreamList {
    #[serde(default)]
    pub(super) items: Vec<LiveStream>,
}

/// One live-stream resource view (O1/O4); documented fields only.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveStream {
    pub(super) id: Option<String>,
    pub(super) status: Option<StreamStatus>,
    pub(super) cdn: Option<StreamCdn>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamCdn {
    pub(super) ingestion_type: Option<String>,
    pub(super) ingestion_info: Option<IngestionInfo>,
}

/// The documented ingestion-address family (never real keys in fixtures).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IngestionInfo {
    pub(super) stream_name: Option<String>,
    pub(super) ingestion_address: Option<String>,
    pub(super) backup_ingestion_address: Option<String>,
    pub(super) rtmps_ingestion_address: Option<String>,
    pub(super) rtmps_backup_ingestion_address: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamStatus {
    pub(super) stream_status: Option<String>,
    pub(super) health_status: Option<StreamHealth>,
}

#[derive(Deserialize)]
pub(crate) struct StreamHealth {
    #[serde(rename = "status")]
    overall: Option<String>,
}

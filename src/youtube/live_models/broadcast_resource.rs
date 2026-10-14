//! `liveBroadcasts` resource views (O2/O3/O5) and the shared response
//! field types. See the parent `live_models` module for the contract:
//! documented fields only, unknown fields ignored, missing parts `None`.

use serde::Deserialize;

/// A `liveBroadcasts.list` page (O5); `items` defaults to empty.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveBroadcastList {
    #[serde(default)]
    pub(super) items: Vec<LiveBroadcast>,
}

/// One broadcast resource view (O2/O3/O5); documented fields only.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LiveBroadcast {
    pub(super) id: Option<String>,
    pub(super) snippet: Option<BroadcastSnippet>,
    pub(super) status: Option<BroadcastStatus>,
    pub(super) content_details: Option<BroadcastContentDetails>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BroadcastSnippet {
    pub(super) scheduled_start_time: Option<String>,
    pub(super) title: Option<String>,
    pub(super) description: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BroadcastStatus {
    pub(super) life_cycle_status: Option<String>,
    pub(super) privacy_status: Option<String>,
    pub(super) recording_status: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BroadcastContentDetails {
    pub(super) bound_stream_id: Option<String>,
}

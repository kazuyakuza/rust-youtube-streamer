//! `liveBroadcasts` resource views (O2/O3/O5) and the shared response
//! field types. See the parent `live_models` module for the contract:
//! documented fields only, unknown fields ignored, missing parts `None`.
//!
//! Grounding: `id`, `snippet.{title,description,scheduledStartTime}`,
//! `status.{lifeCycleStatus,privacyStatus,recordingStatus}` and
//! `contentDetails.boundStreamId` are the documented fields of
//! https://developers.google.com/youtube/v3/live/docs/liveBroadcasts.
//!
//! Staging: dead until Task 2's concrete transport deserializes a
//! `liveBroadcasts` response through these views.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.3, F1); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 1.

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
    pub(crate) id: Option<String>,
    pub(super) snippet: Option<BroadcastSnippet>,
    pub(crate) status: Option<BroadcastStatus>,
    pub(crate) content_details: Option<BroadcastContentDetails>,
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
    pub(crate) life_cycle_status: Option<String>,
    pub(super) privacy_status: Option<String>,
    pub(super) recording_status: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BroadcastContentDetails {
    pub(crate) bound_stream_id: Option<String>,
}

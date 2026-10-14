//! The mockable YouTube Live API transport seam (plan F5, D-T1-3): one
//! trait with exactly the five selected operations O1–O5. Task 2 owns the
//! concrete `ureq`-backed implementation (no Tokio, redirects disabled,
//! the Phase-04 verifier agent pattern) and Task 2/3 tests fake this trait
//! offline — no `cfg(test)` split, no real Google call ever.
//!
//! `lookup_*` returns `Ok(None)` only when the list answer is verifiably
//! empty for the requested id (an empty `items` array). A body that fails
//! to parse is a typed `MalformedResponse`, never a silent `None`.
//!
//! Grounding: O1/O4 are the insert/list operations of
//! https://developers.google.com/youtube/v3/live/docs/liveStreams and
//! O2/O3/O5 those of
//! https://developers.google.com/youtube/v3/live/docs/liveBroadcasts
//! (each request carries `Authorization: Bearer <access_token>`; the
//! force-ssl scope already pinned in `authorization` is documented-
//! sufficient for all five).
//!
//! Staging: the concrete ureq backend landed in Task 2
//! (`google_live_api_transport` implements this trait); this module keeps
//! no whole-module staging allow after the Task-2 retirement act. The
//! seam is consumed by Task 2's transport/orchestrator/waits and is truly
//! live when Task 3's `run` wiring roots the chain.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.1, D-T1-3, F5); TODO `.agent/todos/20261013/20261013-todo-1.md`
//! Task 1.

use super::live_error::LiveApiError;
use super::live_models::{
    BindCommand, CreateBroadcastCommand, CreateStreamCommand, LiveBroadcast, LiveStream,
};

/// The five Phase-05 runtime operations (plan §1.1). The concrete ureq
/// backend is Task 2 (`google_live_api_transport`); offline fakes implement this
/// trait in Task 2 tests.
pub(crate) trait LiveApiTransport {
    /// O1 `liveStreams.insert` (part=snippet,cdn): creates the ingestion
    /// stream; the response yields the stream id + `cdn.ingestionInfo`.
    fn create_stream(&self, command: &CreateStreamCommand) -> Result<LiveStream, LiveApiError>;

    /// O2 `liveBroadcasts.insert` (part=snippet,contentDetails,status):
    /// creates the broadcast; the response yields the broadcast id.
    fn create_broadcast(
        &self,
        command: &CreateBroadcastCommand,
    ) -> Result<LiveBroadcast, LiveApiError>;

    /// O3 `liveBroadcasts.bind` (no request body): links broadcast to
    /// stream; the response carries `contentDetails.boundStreamId`.
    fn bind(&self, command: &BindCommand) -> Result<LiveBroadcast, LiveApiError>;

    /// O4 `liveStreams.list` (part=status,cdn): readiness polling on
    /// `status.streamStatus`; also re-confirms `cdn.ingestionInfo`.
    fn lookup_stream(&self, id: &str) -> Result<Option<LiveStream>, LiveApiError>;

    /// O5 `liveBroadcasts.list` (part=status): readiness polling on
    /// `status.lifeCycleStatus`.
    fn lookup_broadcast(&self, id: &str) -> Result<Option<LiveBroadcast>, LiveApiError>;
}

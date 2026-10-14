//! Transport-agnostic command types for the O1–O3 operations. These are
//! validated inputs the runtime builds from config; they carry no serde
//! derives because the wire shapes are the request bodies in
//! `request_bodies`.
//!
//! Grounding: the operations, parts and parameters are the official
//! liveStreams/liveBroadcasts references
//! (https://developers.google.com/youtube/v3/live/docs/liveStreams and
//! https://developers.google.com/youtube/v3/live/docs/liveBroadcasts):
//! O1 `liveStreams.insert?part=snippet,cdn`, O2
//! `liveBroadcasts.insert?part=snippet,contentDetails,status`, O3
//! `liveBroadcasts.bind?id=&streamId=` (bind takes query parameters only —
//! "Do not provide a request body when calling this method").
//!
//! Staging: dead until Task 2's concrete transport consumes them as the
//! typed inputs of the O1–O3 calls.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.1, F1); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 1.

/// O1 (`liveStreams.insert`) command: validated title + CDN labels.
pub(crate) struct CreateStreamCommand {
    pub(crate) title: String,
    pub(crate) resolution: String,
    pub(crate) frame_rate: String,
}

/// O2 (`liveBroadcasts.insert`) command: validated broadcast metadata.
pub(crate) struct CreateBroadcastCommand {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) privacy_status: String,
    pub(crate) scheduled_start_time: String,
}

/// O3 (`liveBroadcasts.bind`) parameters: query-string pairs, no body
/// (official: "Do not provide a request body when calling this method").
pub(crate) struct BindCommand {
    pub(crate) broadcast_id: String,
    pub(crate) stream_id: String,
}

//! Transport-agnostic command types for the O1–O3 operations. These are
//! validated inputs the runtime builds from config; they carry no serde
//! derives because the wire shapes are the request bodies in
//! `request_bodies`.

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

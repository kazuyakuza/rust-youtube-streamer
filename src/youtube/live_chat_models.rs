//! The fixed wire form of the Phase-06 O6 chat lookup (record D-1,
//! plan §3.1): the broadcast lookup path with the documented query form
//! (`part=snippet` owns `liveChatId`, S5); never `hl`,
//! `profileImageSize` or `key=` (Bearer only, S2/S3). The O7 list models
//! and path retired with the Phase-07 streamList switch (Task-1 record
//! §12); the stream wire form lives in `stream_proto`.
//!
//! Grounding: S1/S2/S5 citations + access dates live in
//! `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` §1.
//! Offline tests only — fakes/`127.0.0.1` loopback; no test reaches Google.

use super::api_request::encode_query_value;

/// The O6 broadcast lookup path+query (`part=snippet` owns `liveChatId`, S5).
pub(super) fn broadcast_chat_lookup_path(broadcast_id: &str) -> String {
    format!(
        "liveBroadcasts?part=snippet&id={}",
        encode_query_value(broadcast_id)
    )
}

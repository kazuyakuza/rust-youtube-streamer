//! The youtube-owned chat text carrier of Phase 06 (record D-4), kept by
//! the Phase-07 stream mapping: `ChatTextMessage` plus the blank-text
//! helper the proto-side accept/skip policy reuses, and the RFC 3339 →
//! epoch parser child module. The O7 page mapper retired with the
//! streamList switch (Task-1 record §12); the skip policy over the proto
//! wire lives in `live_chat_stream_mapping`.
//!
//! Grounding: S1/S2 citations + access dates in
//! `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` §1.
//! Offline tests only — no test reaches Google.

#[path = "live_chat_rfc3339.rs"]
mod live_chat_rfc3339;

pub(crate) use self::live_chat_rfc3339::parse_rfc3339_seconds;

/// The youtube-owned carrier the mapping returns (record D-4): the app
/// side constructs `ChatMessage`, so `src/youtube/` stays free of any
/// `crate::chat` dependency. Debug only — never `Display`.
#[derive(Debug)]
pub(crate) struct ChatTextMessage {
    pub(crate) author: String,
    pub(crate) text: String,
    pub(crate) message_id: Option<String>,
    pub(crate) published_epoch: Option<u64>,
}

pub(super) fn non_blank_text(value: &Option<String>) -> Option<String> {
    let text = value.as_ref()?;
    if text.trim().is_empty() {
        return None;
    }
    Some(text.clone())
}

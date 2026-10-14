//! The wire-plumbing sibling of the concrete Live API transport (Amendment-2
//! cap re-cut applied to F-T2-1's single-file sketch): the request-form
//! parameter object and its three D-T2-4 constructors (`getting`,
//! `posting` — bodyless POST for bind — and `posting_json`), the five
//! documented path+query builders (ids percent-encoded per RFC 3986), the
//! locked request-body serialization seam, the shared 2xx body read (fixed
//! unreadable-body text, the Phase-04 precedent), the status-first
//! reason-extraction path into the staged `classify_api_error`, and the
//! deliberate D-T2-6 local duplicate of the verifier's private
//! `extract_reason` (a stable Phase-04 file; ~20 lines beat a risky
//! refactor). Response bodies are NEVER embedded into errors — only the
//! non-secret `reason` code is surfaced.
//!
//! Grounding: the five documented request forms and query parts (D-T2-4)
//! are the O1–O5 operations of the liveStreams / liveBroadcasts references
//! cited in `api_transport`; bind carries no request body per
//! https://developers.google.com/youtube/v3/live/docs/liveBroadcasts.
//!
//! Plan: `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
//! (F-T2-1, D-T2-4..6); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 2.

use serde::Deserialize;
use serde::Serialize;

use super::live_error::{classify_api_error, LiveApiError};
use super::live_models::BindCommand;

/// Fixed text when a 2xx reply body cannot be read.
const UNREADABLE_BODY_TEXT: &str = "the YouTube API response body could not be read";

#[derive(Clone, Copy)]
pub(super) enum LiveApiMethod {
    Get,
    Post,
}

/// The parameter object encoding WHICH request form (D-T2-4).
pub(super) struct LiveApiRequestSpec<'a> {
    pub(super) method: LiveApiMethod,
    pub(super) path_query: String,
    pub(super) body: Option<&'a serde_json::Value>,
}

impl<'a> LiveApiRequestSpec<'a> {
    /// GET with no body (O4/O5).
    pub(super) fn getting(path_query: String) -> Self {
        Self {
            method: LiveApiMethod::Get,
            path_query,
            body: None,
        }
    }

    /// Bodyless POST (O3 bind: "Do not provide a request body").
    pub(super) fn posting(path_query: String) -> Self {
        Self {
            method: LiveApiMethod::Post,
            path_query,
            body: None,
        }
    }

    /// POST with the locked JSON body (O1/O2).
    pub(super) fn posting_json(path_query: String, body: &'a serde_json::Value) -> Self {
        Self {
            method: LiveApiMethod::Post,
            path_query,
            body: Some(body),
        }
    }
}

pub(super) fn stream_insert_path() -> String {
    "liveStreams?part=snippet,cdn".to_string()
}

pub(super) fn broadcast_insert_path() -> String {
    "liveBroadcasts?part=snippet,contentDetails,status".to_string()
}

pub(super) fn bind_path(command: &BindCommand) -> String {
    format!(
        "liveBroadcasts/bind?id={}&streamId={}",
        encode_query_value(&command.broadcast_id),
        encode_query_value(&command.stream_id)
    )
}

pub(super) fn stream_lookup_path(id: &str) -> String {
    format!("liveStreams?part=status,cdn&id={}", encode_query_value(id))
}

pub(super) fn broadcast_lookup_path(id: &str) -> String {
    format!("liveBroadcasts?part=status&id={}", encode_query_value(id))
}

/// Serialization of the locked request-body structs (pure String/bool —
/// structurally unreachable); exists so no call site ever `expect`s.
pub(super) fn request_body_value<T: Serialize>(body: T) -> Result<serde_json::Value, LiveApiError> {
    serde_json::to_value(body).map_err(|_| LiveApiError::MalformedResponse {
        field: "request body".to_string(),
    })
}

pub(super) fn response_body(response: ureq::Response) -> Result<String, LiveApiError> {
    response.into_string().map_err(|_| LiveApiError::Network {
        reason: UNREADABLE_BODY_TEXT.to_string(),
    })
}

/// Reads the non-secret `reason` code from an error body (best-effort)
/// and hands the (status, reason) pair to the ONE staged classifier.
pub(super) fn response_reason_error(status: u16, response: ureq::Response) -> LiveApiError {
    let reason = response.into_string().ok();
    classify_api_error(
        status,
        reason.as_deref().and_then(extract_reason).as_deref(),
    )
}

fn extract_reason(body: &str) -> Option<String> {
    let document: ApiErrorDocument = serde_json::from_str(body).ok()?;
    let detail = document.error?;
    detail.errors.into_iter().find_map(|item| item.reason)
}

/// Percent-encodes every byte outside the RFC 3986 unreserved set
/// (defense in depth; Google ids are unreserved in practice).
pub(super) fn encode_query_value(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        if is_unreserved(byte) {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~')
}

#[derive(Deserialize)]
struct ApiErrorDocument {
    error: Option<ApiErrorDetail>,
}

#[derive(Deserialize)]
struct ApiErrorDetail {
    errors: Vec<ApiErrorItem>,
}

#[derive(Deserialize)]
struct ApiErrorItem {
    reason: Option<String>,
}

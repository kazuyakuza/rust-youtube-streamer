//! The widened `MessageSource` failure contract of Phase 06 Task 3
//! (record D-8, carry-forward C-4): the frame-pipeline seam now carries
//! TWO failure classes — the chat-domain store error (log-first
//! acceptance; `ChatStore::accept` rejections) and the typed chat
//! RETRIEVAL failure funnelled from the production worker's terminal
//! cell — so provider/network failures never collapse into misleading
//! chat-log errors. The pure [`pipeline_source_failure`] mapper keeps the
//! runner's error conversion in one place.

use std::fmt;

use super::error::FramePipelineError;
use crate::chat::ChatError;

/// The widened `MessageSource` failure contract (record D-8): chat-domain
/// store errors stay in the `Chat` class; retrieval (YouTube/network)
/// failures are typed separately and NEVER collapse into chat-log errors.
#[derive(Debug)]
pub(crate) enum ChatSourceError {
    // Phase-06 Task 3: the D-8 pinned `Chat` class (log-first contract)
    // is produced only by source fakes/tests today; the production
    // `ChatSource` yields `Retrieval`. Clippy-named allow (plan §6(f)).
    #[allow(dead_code)]
    Chat(ChatError),
    Retrieval(ChatRetrievalError),
}

/// A typed chat-retrieval failure carried from the worker's terminal
/// cell (record D-8). `detail` carries ONLY the redacted `Display` of the
/// inner typed `LiveApiError` — never a body, never a token.
#[derive(Debug)]
pub(crate) enum ChatRetrievalError {
    ChatUnavailable { detail: String },
    Quota { detail: String },
    Transport { detail: String },
}

impl fmt::Display for ChatSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatSourceError::Chat(error) => write!(formatter, "{error}"),
            ChatSourceError::Retrieval(error) => write!(formatter, "{error}"),
        }
    }
}

impl std::error::Error for ChatSourceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ChatSourceError::Chat(error) => Some(error),
            ChatSourceError::Retrieval(error) => Some(error),
        }
    }
}

impl fmt::Display for ChatRetrievalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatRetrievalError::ChatUnavailable { detail } => {
                write!(formatter, "live chat is unavailable: {detail}")
            }
            ChatRetrievalError::Quota { detail } => {
                write!(
                    formatter,
                    "live chat retrieval stopped by YouTube quota: {detail}"
                )
            }
            ChatRetrievalError::Transport { detail } => {
                write!(
                    formatter,
                    "live chat transport failed after repeated attempts: {detail}"
                )
            }
        }
    }
}

impl std::error::Error for ChatRetrievalError {}

/// The pure mapper consumed by the frame runner: one seam failure becomes
/// exactly one typed pipeline failure, class preserved.
pub(crate) fn pipeline_source_failure(error: ChatSourceError) -> FramePipelineError {
    match error {
        ChatSourceError::Chat(chat_error) => FramePipelineError::Chat(chat_error),
        ChatSourceError::Retrieval(retrieval) => FramePipelineError::ChatRetrieval(retrieval),
    }
}

#[cfg(test)]
#[path = "source_error_tests.rs"]
mod source_error_tests;

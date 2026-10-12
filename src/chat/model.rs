//! Application-level chat message model: author, visible text, and optional
//! internal metadata (API message identifier, epoch-second timestamp) that
//! never appears in visible text. Deliberately free of YouTube API types,
//! network, streaming, and FFmpeg (brief §8.2 message handling, §8.4 message
//! storage); scope authority `.agent/todos/20261010/20261010-todo-2-DONE.md` §1.

use std::fmt;

/// One ordinary chat message. The `Display` output is exactly the visible
/// `{author}: {text}` format; metadata stays off-screen.
#[derive(Debug, Clone)]
pub struct ChatMessage {
    author: String,
    text: String,
    message_id: Option<String>,
    timestamp: Option<u64>,
}

impl ChatMessage {
    /// Builds a message whose author and text are normalized to a single
    /// line (every line break flattened to one space, ends trimmed) and
    /// with no metadata attached.
    pub fn new(author: &str, text: &str) -> Self {
        Self {
            author: normalize(author),
            text: normalize(text),
            message_id: None,
            timestamp: None,
        }
    }

    /// Attaches the API message identifier verbatim, builder style, for the
    /// future API layer.
    pub fn with_message_id(mut self, id: &str) -> Self {
        self.message_id = Some(id.to_string());
        self
    }

    /// Attaches the timestamp verbatim, builder style: seconds since the
    /// UNIX epoch, or `None` to leave the message without one.
    // Reserved for the YouTube chat API layer (later phase); the demo attaches
    // deterministic message IDs but no timestamps. Deleting it would delete the
    // store's own-timestamp log-record coverage.
    #[allow(dead_code)]
    pub fn with_timestamp(mut self, seconds: Option<u64>) -> Self {
        self.timestamp = seconds;
        self
    }

    pub(crate) fn author(&self) -> &str {
        &self.author
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn timestamp(&self) -> Option<u64> {
        self.timestamp
    }
}

impl fmt::Display for ChatMessage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.author, self.text)
    }
}

/// Flattens every line-break sequence to one space and trims the ends, so a
/// normalized message occupies exactly one visible line and one log record
/// line; a CRLF pair becomes a single space, never two.
fn normalize(value: &str) -> String {
    value
        .replace("\r\n", " ")
        .replace(['\r', '\n'], " ")
        .trim()
        .to_string()
}

#[cfg(test)]
#[path = "model_tests.rs"]
mod model_tests;

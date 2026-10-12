//! Bounded visible-message queue [`ChatStore`] with an eagerly opened,
//! append-only chat log that is never read back. The visible capacity is
//! recomputed from validated video geometry with overflow-safe `u64`
//! arithmetic, and every accepted message is appended to the log before it
//! may enter the queue, so a failed append rejects the message instead of
//! letting queue and log diverge (brief §8.4 message storage, §11.2 chat
//! log); scope authority `.agent/todos/20261010/20261010-todo-2.md` §1. The
//! store cannot depend on YouTube API types, network, streaming, or FFmpeg.

use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::chat::error::ChatError;
use crate::chat::model::ChatMessage;
use crate::config::model::{AppConfig, RendererConfig, VideoConfig};

/// Recomputes `floor((height − top_margin − bottom_margin) / line_height)`
/// in `u64` so margin arithmetic can never overflow, and rejects any
/// geometry that leaves room for less than one visible line.
pub(crate) fn derive_visible_capacity(
    video: &VideoConfig,
    renderer: &RendererConfig,
) -> Result<usize, ChatError> {
    let line_height = u64::from(renderer.line_height);
    if line_height == 0 {
        return Err(capacity_error(video, renderer));
    }
    let occupied = u64::from(renderer.top_margin) + u64::from(renderer.bottom_margin);
    let usable_height = u64::from(video.height)
        .checked_sub(occupied)
        .ok_or_else(|| capacity_error(video, renderer))?;
    let capacity = usable_height / line_height;
    if capacity == 0 {
        return Err(capacity_error(video, renderer));
    }
    usize::try_from(capacity).map_err(|_| capacity_error(video, renderer))
}

fn capacity_error(video: &VideoConfig, renderer: &RendererConfig) -> ChatError {
    ChatError::Capacity {
        height: video.height,
        top_margin: renderer.top_margin,
        bottom_margin: renderer.bottom_margin,
        line_height: renderer.line_height,
    }
}

/// Validated construction inputs for a [`ChatStore`]: the derived visible
/// capacity and the append-only log path.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatStoreSettings {
    visible_capacity: usize,
    log_path: PathBuf,
}

impl ChatStoreSettings {
    /// Derives the visible capacity from already-validated video geometry
    /// and takes the configured chat log path.
    pub fn from_config(config: &AppConfig) -> Result<Self, ChatError> {
        let visible_capacity = derive_visible_capacity(&config.video, &config.renderer)?;
        Ok(Self {
            visible_capacity,
            log_path: PathBuf::from(&config.chat.log_file),
        })
    }
}

/// Bounded FIFO queue of the currently visible chat messages plus the
/// append-only log handle opened eagerly at construction.
#[derive(Debug)]
pub struct ChatStore {
    settings: ChatStoreSettings,
    log_file: File,
    messages: VecDeque<ChatMessage>,
}

impl ChatStore {
    /// Opens the log file in append mode, creating it when missing. Every
    /// fresh store starts with an empty queue regardless of prior log
    /// contents; the log is never read back.
    pub fn new(settings: ChatStoreSettings) -> Result<Self, ChatError> {
        let log_file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&settings.log_path)
            .map_err(|source| ChatError::LogOpen {
                path: settings.log_path.clone(),
                source,
            })?;
        Ok(Self {
            settings,
            log_file,
            messages: VecDeque::new(),
        })
    }

    /// Appends the log record first; only after a successful append does the
    /// message enter the visible queue, evicting the oldest beyond capacity.
    /// A failed append rejects the message entirely: it appears in neither
    /// the log nor the queue.
    pub fn accept(&mut self, message: ChatMessage) -> Result<(), ChatError> {
        self.write_log_record(&message)?;
        self.enqueue(message);
        Ok(())
    }

    /// Visible `{author}: {text}` lines joined with `'\n'`, oldest first;
    /// an empty store renders as an empty string.
    pub fn visible_text(&self) -> String {
        self.messages
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Whether no message is currently visible.
    // Retains the Phase-02 public contract surface; no consumer exists until a
    // later phase (deliberate, single-item allowance).
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Appends one record as `<epoch_seconds> <author>: <text>\n`, using the
    /// message's own timestamp when present, otherwise its generation time
    /// in seconds since the UNIX epoch. A single `write_all` surfaces any
    /// failure per message; `ChatStore::accept` turns it into a rejection.
    fn write_log_record(&mut self, message: &ChatMessage) -> Result<(), ChatError> {
        let generated_seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(u64::MAX, |elapsed| elapsed.as_secs());
        let seconds = message.timestamp().unwrap_or(generated_seconds);
        let author = message.author();
        let text = message.text();
        let record = format!("{seconds} {author}: {text}\n");
        self.log_file
            .write_all(record.as_bytes())
            .map_err(|source| ChatError::LogAppend {
                path: self.settings.log_path.clone(),
                source,
            })
    }

    fn enqueue(&mut self, message: ChatMessage) {
        self.messages.push_back(message);
        while self.messages.len() > self.settings.visible_capacity {
            self.messages.pop_front();
        }
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;

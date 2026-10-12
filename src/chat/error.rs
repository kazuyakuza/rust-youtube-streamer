//! Typed chat errors. Each variant names the failed operation and the
//! offending geometry or path, so callers can report actionable messages
//! without matching on error strings. Contracts follow brief §8.4 message
//! storage and §11.2 chat log; scope authority
//! `.agent/todos/20261010/20261010-todo-2-DONE.md` §1.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// All failures produced by the chat module, one variant per failure stage.
#[derive(Debug)]
pub enum ChatError {
    /// The configured geometry leaves no room for at least one visible line:
    /// usable height below one line, margins reaching the height, or a zero
    /// line height. Carries the offending values.
    Capacity {
        height: u32,
        top_margin: u32,
        bottom_margin: u32,
        line_height: u32,
    },
    /// `ChatStore::new` could not open the chat log file in append mode.
    /// Carries the log path and the underlying I/O cause.
    LogOpen { path: PathBuf, source: io::Error },
    /// `ChatStore::accept` failed to append the record, so the message was
    /// rejected: it was neither logged nor added to the visible queue.
    /// Carries the log path and the underlying I/O cause.
    LogAppend { path: PathBuf, source: io::Error },
}

impl fmt::Display for ChatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChatError::Capacity {
                height,
                top_margin,
                bottom_margin,
                line_height,
            } => write!(
                formatter,
                "chat visible capacity must allow at least one line (height {height}, top_margin {top_margin}, bottom_margin {bottom_margin}, line_height {line_height})"
            ),
            ChatError::LogOpen { path, source } => write!(
                formatter,
                "unable to open chat log file '{}' in append mode: {}",
                path.display(),
                source
            ),
            ChatError::LogAppend { path, source } => write!(
                formatter,
                "unable to append chat record to chat log file '{}': {}; the message was rejected and was not added to the visible queue",
                path.display(),
                source
            ),
        }
    }
}

impl Error for ChatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ChatError::Capacity { .. } => None,
            ChatError::LogOpen { source, .. } => Some(source),
            ChatError::LogAppend { source, .. } => Some(source),
        }
    }
}

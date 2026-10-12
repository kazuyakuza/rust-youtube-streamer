//! Application-level chat message model and the bounded visible-message
//! queue [`ChatStore`]: the module cannot depend on YouTube API types,
//! network access, streaming, or FFmpeg (brief §8.2 message handling, §8.4
//! message storage, §11.2 chat log). [`ChatStore`] holds only the currently
//! visible messages in a FIFO queue and appends every accepted message to an
//! independent append-only log file that is never read back.
//!
//! Scope authority: `.agent/todos/20261010/20261010-todo-2.md` §1.
//!
//! Access path note for future agents: tests reach items via `super::*`;
//! cross-module consumers go through the `pub(crate) use` re-exports below.
//! Staging allow: retires in TODO Task 3 when the demo mode (the pipeline's
//! CLI consumer) makes this module reachable from `main`; clippy
//! `-D warnings` guards everything else.
#![allow(dead_code)]

mod error;
mod model;
mod store;

pub(crate) use error::ChatError;
pub(crate) use model::ChatMessage;
#[cfg(test)]
pub(crate) use store::derive_visible_capacity;
pub(crate) use store::{ChatStore, ChatStoreSettings};

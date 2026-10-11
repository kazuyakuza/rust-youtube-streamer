//! Application-level chat message model and the bounded visible-message
//! queue [`ChatStore`]: the module cannot depend on YouTube API types,
//! network access, streaming, or FFmpeg (brief §8.2 message handling, §8.4
//! message storage, §11.2 chat log). [`ChatStore`] holds only the currently
//! visible messages in a FIFO queue and appends every accepted message to an
//! independent append-only log file that is never read back.
//!
//! Scope authority: `.agent/todos/20261010/20261010-todo-2.md` §1.
//!
//! Access path note for future agents: items are reached within this module
//! only (tests via `super::*`); the cross-module `pub use` re-exports return
//! when the first external consumer lands, and the `#[allow(dead_code)]` on
//! `mod chat;` in `main.rs` retires at that Phase 03 app-wiring step.

mod error;
mod model;
mod store;

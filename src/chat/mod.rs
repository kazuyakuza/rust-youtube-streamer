//! Application-level chat message model and the bounded visible-message
//! queue [`ChatStore`]: the module cannot depend on YouTube API types,
//! network access, streaming, or FFmpeg (brief §8.4 message storage,
//! §11.2 chat log). [`ChatStore`] holds only the currently visible messages
//! in a FIFO queue and appends every accepted message to an independent
//! append-only log file.
//!
//! Items are accessed within this module (tests via `super::*`); the
//! cross-module re-exports return when the first external consumer lands
//! in the later app-wiring phase.
//! In `main.rs`, `dead_code` is intentionally allowed until app wiring adds consumers.

mod error;
mod model;
mod store;

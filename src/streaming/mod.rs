//! FFmpeg process supervision and the raw-frame input pipeline (brief
//! §10.1–10.5; scope authority `.agent/todos/20261011/20261011-todo-1.md`
//! §1). The module launches FFmpeg as an external process with arguments
//! built purely from validated configuration, feeds raw RGB24 frames to its
//! standard input, waits for clean stdin-EOF finalization, and reports
//! process failures as typed errors. It depends only on the standard library
//! and the typed configuration; it knows nothing about chat messages,
//! rendering, or YouTube. Frame bytes arrive through the `FrameSink` seam as
//! opaque byte slices.
//!
//! Module-independence rules:
//! - `streaming` depends only on `std` and `crate::config::model` (typed
//!   `AppConfig`). It never depends on `crate::chat`, `crate::renderer`, or
//!   any YouTube type.
//! - The seam receives opaque frame bytes (`&[u8]`) only — it cannot know
//!   what a chat message is.
//! - `streaming` is imported by `src/main.rs` with a deliberate
//!   `#[allow(dead_code)]` that retires when Task 2 wires the frame
//!   pipeline (same pattern as `mod chat;` / `mod renderer;` today).
//!
//! Access path note for future agents: items are reached within this module
//! only (tests via `super::*`); the cross-module `pub use` re-exports return
//! when the first external consumer lands.

mod args;
mod error;
mod utc_timestamp;

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
//! - `streaming` is imported by `src/main.rs`; cross-module consumers go
//!   through the `pub(crate) use` re-exports below. Staging allow: retires
//!   in TODO Task 3 when the demo mode (the pipeline's CLI consumer) makes
//!   this module reachable from `main`; clippy `-D warnings` guards
//!   everything else.
//!
//! Access path note for future agents: tests reach items via `super::*`.
#![allow(dead_code)]

mod args;
mod error;
mod ffmpeg;
mod sink;
mod utc_timestamp;
mod video_output;

pub(crate) use error::StreamingError;
pub(crate) use ffmpeg::FfmpegProcess;
pub(crate) use sink::{CompletedVideo, FrameSink};
pub(crate) use video_output::{resolve_output_path, OutputKind};

// Widened for the app's test double (sink.rs): `CompletedVideo::new`
// changed `pub(super)` → `pub(crate)`; the test fake must fabricate a
// successful `finish` value without a real FFmpeg process.

//! FFmpeg process supervision and the raw-frame input pipeline (brief
//! §10.1–10.5; scope authority `.agent/todos/20261011/20261011-todo-1-DONE.md`
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
//!   through the `pub(crate) use` re-exports below. Reachable from `main`
//!   through the `demo` mode (TODO Task 3).
//!
//! Access path note for future agents: tests reach items via `super::*`.

mod args;
mod error;
mod ffmpeg;
mod local_finalize;
mod pipe_drain;
mod rtmps_destination;
mod sink;
mod spawn;
mod utc_timestamp;
mod video_output;

pub(crate) use error::StreamingError;
pub(crate) use ffmpeg::FfmpegProcess;
pub(crate) use rtmps_destination::RtmpsDestination;
pub(crate) use sink::{CompletedOutput, FrameSink};
pub(crate) use video_output::{resolve_output_path, OutputKind};

// Widened for the app's test doubles (sink.rs): `CompletedVideo::new`
// changed `pub(super)` → `pub(crate)`; the fake sinks fabricate a
// successful local-recording `finish` value without a real FFmpeg
// process. Test-only, so the crate re-export stays consumer-locked
// (nothing in the production build reaches for it).
#[cfg(test)]
pub(crate) use sink::CompletedVideo;

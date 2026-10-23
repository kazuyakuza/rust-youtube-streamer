//! The minimal contract for sending complete frames and finishing the
//! process: the frame pipeline (Task 2) sees only this trait, never
//! `std::process` types, keeping FFmpeg process management independent of
//! chat/render details (deliverable bullet: "Define a small interface for
//! sending complete frames and finishing the process").

use std::path::{Path, PathBuf};

use crate::streaming::error::StreamingError;

/// The finalized local recording produced by a successful
/// [`FrameSink::finish`]: proof that FFmpeg exited cleanly and the MP4 was
/// verified on disk.
pub struct CompletedVideo {
    output_path: PathBuf,
}

impl CompletedVideo {
    /// Fabricates a finalized-video value. Test doubles in the app crate
    /// fabricate a success value without a real process; construction stays
    /// crate-internal.
    pub(crate) fn new(output_path: PathBuf) -> Self {
        Self { output_path }
    }

    /// Full path of the finalized MP4 file.
    pub fn path(&self) -> &Path {
        &self.output_path
    }
}

/// Exit/exit-evidence bundle of a FINISHED streaming FFmpeg process
/// (run diagnostics task): the exit code verbatim (`None` = signal
/// death) and the captured stderr tail AFTER the destination has been
/// redacted to `[redacted]` — the redaction happens at CAPTURE time in
/// `FfmpegProcess::finish`, so no consumer ever sees the URL. No
/// `Display`: app-layer tracing consumes the accessors.
#[derive(Debug)]
pub struct StreamExit {
    exit_code: Option<i32>,
    stderr_tail: String,
}

impl StreamExit {
    /// The streaming-family constructor (`ffmpeg.rs` is the only
    /// production builder; tests never fabricate carriers).
    pub(super) fn new(exit_code: Option<i32>, stderr_tail: String) -> Self {
        Self {
            exit_code,
            stderr_tail,
        }
    }

    /// The child's exit status code; `None` means it died by signal.
    pub(crate) fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// The redacted trailing stderr diagnostics (≤ 4096 bytes).
    pub(crate) fn stderr_tail(&self) -> &str {
        &self.stderr_tail
    }
}

/// Sends complete RGB24 frames to the FFmpeg process, then finalizes it.
pub trait FrameSink {
    /// Writes exactly one complete frame. Implementations loop short writes
    /// (`write_all`) and surface pipe failures as
    /// [`StreamingError::FrameWrite`]. Frame length is simply forwarded;
    /// capacity-vs-size validation is the renderer's job, not the sink's.
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), StreamingError>;

    /// Ends the frame stream, waits for FFmpeg to exit after stdin-EOF,
    /// verifies the finalization target (the local MP4 on disk; nothing
    /// beyond a clean exit for RTMPS), and consumes the sink. Success means
    /// exit status 0 AND — for local recordings — an existing, non-empty
    /// output file; never less.
    fn finish(self) -> Result<CompletedOutput, StreamingError>;
}

/// Verified finalization proof of a finished frame sink (plan D-T3-5 F-S4,
/// D-T1-10): local recordings verify the MP4 on disk; RTMPS transmission
/// verifies only a clean FFmpeg exit (no local file exists to check).
pub enum CompletedOutput {
    /// A verified local MP4 recording (`demo`; the prior `CompletedVideo`
    /// semantics, unchanged).
    Recording(CompletedVideo),
    /// FFmpeg exited cleanly streaming to the RTMPS destination; the
    /// payload carries its exit status and redacted stderr tail for the
    /// app's shutdown evidence lines.
    Stream(StreamExit),
}

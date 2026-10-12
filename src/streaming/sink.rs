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

/// Sends complete RGB24 frames to the FFmpeg process, then finalizes it.
pub trait FrameSink {
    /// Writes exactly one complete frame. Implementations loop short writes
    /// (`write_all`) and surface pipe failures as
    /// [`StreamingError::FrameWrite`]. Frame length is simply forwarded;
    /// capacity-vs-size validation is the renderer's job, not the sink's.
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), StreamingError>;

    /// Ends the frame stream, waits for FFmpeg to exit after stdin-EOF,
    /// verifies the MP4 was finalized, and consumes the sink. Success means
    /// exit status 0 AND an existing, non-empty output file — never less.
    fn finish(self) -> Result<CompletedVideo, StreamingError>;
}

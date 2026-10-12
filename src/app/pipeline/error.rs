//! All fatal failures of the application-level frame pipeline. Streaming
//! failures reuse Task 1's typed [`StreamingError`] set verbatim (no new
//! streaming semantics); one variant per pipeline stage keeps reporting
//! actionable. Mapped to exit codes by the mode command in TODO Task 3.

use std::error::Error;
use std::fmt;

use crate::chat::ChatError;
use crate::renderer::RenderError;
use crate::streaming::StreamingError;

/// Fatal frame-pipeline failure, one variant per stage (chat message,
/// rendering, setup geometry defense, sink output).
#[derive(Debug)]
pub(crate) enum FramePipelineError {
    /// A message stage failure (`ChatStore::accept` rejection or source
    /// failure).
    Chat(ChatError),
    /// The renderer refused a frame.
    Render(RenderError),
    /// Setup-time defense: `width*height*3` overflows usize. Unreachable
    /// through the validated config loader.
    FrameByteOverrun { width: u32, height: u32 },
    /// Per-frame defense-in-depth: the rendered frame is not exactly
    /// `width*height*3` bytes (renderer contract violation never reported
    /// silently).
    FrameSizeMismatch { expected: usize, actual: usize },
    /// Streaming failures (spawn, frame write incl. broken pipe, wait,
    /// exit status, output finalization) reuse `StreamingError` as-is.
    Sink(StreamingError),
}

impl fmt::Display for FramePipelineError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FramePipelineError::Chat(error) => {
                write!(formatter, "chat message stage failed: {error}")
            }
            FramePipelineError::Render(error) => {
                write!(formatter, "frame rendering failed: {error}")
            }
            FramePipelineError::FrameByteOverrun { width, height } => write!(
                formatter,
                "frame byte count for {width}x{height} overflows usize; the pipeline refused to start"
            ),
            FramePipelineError::FrameSizeMismatch { expected, actual } => write!(
                formatter,
                "rendered frame holds {actual} bytes but the pipeline requires exactly {expected} (width*height*3)"
            ),
            FramePipelineError::Sink(error) => write!(formatter, "frame output failed: {error}"),
        }
    }
}

impl Error for FramePipelineError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FramePipelineError::Chat(error) => Some(error),
            FramePipelineError::Render(error) => Some(error),
            FramePipelineError::FrameByteOverrun { .. } => None,
            FramePipelineError::FrameSizeMismatch { .. } => None,
            FramePipelineError::Sink(error) => Some(error),
        }
    }
}

//! Typed streaming errors: one variant per failure stage of the FFmpeg
//! process wrapper and the local MP4 output pipeline, so callers can report
//! actionable messages without matching on error strings. Contracts follow
//! `.agent/todos/20261011/20261011-todo-1-DONE.md` §1 and the exact display
//! strings fixed by `.kilo/plans/20261011-phase03-task1-ffmpeg-wrapper.md`
//! §1.3. No panics; the argument vector is deliberately never echoed.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// All failures produced by the streaming module, one variant per failure
/// stage.
#[derive(Debug)]
pub enum StreamingError {
    /// The pixel format in the loaded configuration is not the pipeline's
    /// `rgb24`. Defensive check; the config validator makes this
    /// unreachable through `load_from_path`, but the argument builder still
    /// refuses to produce a wrong-format command.
    UnsupportedPixelFormat { value: String },
    /// The configured bitrate is not numeric-leading, so the CBR bufsize
    /// derivation cannot double it. Defensive check mirroring
    /// [`StreamingError::UnsupportedPixelFormat`]; the config validator
    /// accepts non-blank values only, making the reachable probability low
    /// but never zero. The FFmpeg invocation would fail anyway; this variant
    /// fails earlier and more clearly. The `value` field is the raw
    /// configured bitrate text.
    UnsupportedBitrateNotation { value: String },
    /// The application's own executable location could not be determined,
    /// so the `videos/` output directory cannot be placed beside it.
    ExecutableLocation { reason: String },
    /// The `videos/` output directory could not be created (or exists as a
    /// non-directory). Carries the final target path and the I/O cause.
    OutputDirectoryCreate { path: PathBuf, source: io::Error },
    /// All collision-suffix attempts for a recording filename were already
    /// taken — this proves an overwrite would have happened, so nothing is
    /// written.
    OutputNameExhausted { directory: PathBuf, attempts: u32 },
    /// FFmpeg could not be started at the configured executable path
    /// (missing executable, permissions, etc.). The `executable` field is
    /// the configured path text (needed for actionable output) — this is
    /// not a secret; there are no secrets in this phase.
    Spawn {
        executable: String,
        source: io::Error,
    },
    /// A frame write to FFmpeg's stdin failed (including broken pipe).
    /// Task 2's frame loop surfaces this to the user.
    FrameWrite { source: io::Error },
    /// Waiting for the child process raised an I/O error.
    WaitFailed { source: io::Error },
    /// FFmpeg exited with a non-zero status (or was killed by a signal).
    /// Carries the extracted exit code (`None` for signal death) and the
    /// captured stderr diagnostics tail.
    ExitFailure {
        exit_code: Option<i32>,
        diagnostics: String,
    },
    /// FFmpeg exited successfully but the output file is missing or empty —
    /// the MP4 was not finalized.
    OutputFinalize { path: PathBuf },
}

impl fmt::Display for StreamingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamingError::UnsupportedPixelFormat { value } => write!(
                formatter,
                "streaming pipeline requires pixel format 'rgb24' in configuration (got '{value}')"
            ),
            StreamingError::UnsupportedBitrateNotation { value } => write!(
                formatter,
                "streaming pipeline CBR bufsize derivation requires a numeric-leading bitrate in configuration (got '{value}')"
            ),
            StreamingError::ExecutableLocation { reason } => write!(
                formatter,
                "unable to resolve the application executable location: {reason}"
            ),
            StreamingError::OutputDirectoryCreate { path, source } => write!(
                formatter,
                "unable to create video output directory '{}': {}",
                path.display(),
                source
            ),
            StreamingError::OutputNameExhausted { directory, attempts } => write!(
                formatter,
                "video file naming gave up in directory '{}' after {attempts} collision suffixes; the recording was not started",
                directory.display()
            ),
            StreamingError::Spawn { executable, source } => {
                format_spawn_error(formatter, executable, source)
            }
            StreamingError::FrameWrite { source } => {
                format_frame_write_error(formatter, source)
            }
            StreamingError::WaitFailed { source } => {
                write!(formatter, "waiting for the FFmpeg process failed: {source}")
            }
            StreamingError::ExitFailure { exit_code, diagnostics } => write!(
                formatter,
                "{}; captured diagnostics follow: {diagnostics}",
                exit_failure_description(*exit_code)
            ),
            StreamingError::OutputFinalize { path } => write!(
                formatter,
                "FFmpeg reported success but the output file '{}' is missing or empty; the video was not finalized",
                path.display()
            ),
        }
    }
}

impl Error for StreamingError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            StreamingError::UnsupportedPixelFormat { .. } => None,
            StreamingError::UnsupportedBitrateNotation { .. } => None,
            StreamingError::ExecutableLocation { .. } => None,
            StreamingError::OutputDirectoryCreate { source, .. } => Some(source),
            StreamingError::OutputNameExhausted { .. } => None,
            StreamingError::Spawn { source, .. } => Some(source),
            StreamingError::FrameWrite { source } => Some(source),
            StreamingError::WaitFailed { source } => Some(source),
            StreamingError::ExitFailure { .. } => None,
            StreamingError::OutputFinalize { .. } => None,
        }
    }
}

fn is_broken_pipe(source: &io::Error) -> bool {
    source.kind() == io::ErrorKind::BrokenPipe
}

fn is_file_not_found(source: &io::Error) -> bool {
    source.kind() == io::ErrorKind::NotFound
}

fn format_spawn_error(
    formatter: &mut fmt::Formatter<'_>,
    executable: &str,
    source: &io::Error,
) -> fmt::Result {
    if is_file_not_found(source) {
        write!(
            formatter,
            "unable to start the FFmpeg executable '{}': {}; check that the configured path is correct and FFmpeg is installed and runnable",
            executable,
            source
        )
    } else {
        write!(
            formatter,
            "unable to start the FFmpeg executable '{}': {}",
            executable, source
        )
    }
}

fn format_frame_write_error(formatter: &mut fmt::Formatter<'_>, source: &io::Error) -> fmt::Result {
    if is_broken_pipe(source) {
        write!(
            formatter,
            "writing a frame to the FFmpeg process failed: {}; the FFmpeg process exited while receiving frames",
            source
        )
    } else {
        write!(
            formatter,
            "writing a frame to the FFmpeg process failed: {}",
            source
        )
    }
}

/// Keeps only the trailing `limit` bytes of a captured diagnostic stream so
/// the most recent — usually the error — lines survive the cut.
pub(super) fn trailing_bytes(captured: &[u8], limit: usize) -> String {
    let tail = captured.len().saturating_sub(limit);
    String::from_utf8_lossy(&captured[tail..]).into_owned()
}

fn exit_failure_description(exit_code: Option<i32>) -> String {
    match exit_code {
        Some(status_code) => format!("FFmpeg exited with failure status {status_code}"),
        None => "FFmpeg terminated by a signal".to_string(),
    }
}

#[cfg(test)]
#[path = "error_tests.rs"]
mod error_tests;

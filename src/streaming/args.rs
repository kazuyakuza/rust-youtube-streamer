//! Builds the full FFmpeg command-line argument vector purely from typed,
//! already-validated configuration plus the resolved output path. No
//! environment access, no file system, no spawning — pure and unit-testable.
//! The locked vector and its documented rationale (official FFmpeg doc
//! sources FFmpeg-formats §3.23/§4.4, FFmpeg main options, FFmpeg-protocols
//! §3.20) are in .kilo/plans/20261011-phase03-task1-ffmpeg-wrapper.md §0.2;
//! the vector order is asserted by tests and must not be reordered. The
//! production streaming vector lives in the `stream_args` sibling.
//!
//! Locked vector:
//! `-nostdin -n -f rawvideo -pix_fmt rgb24 -s <width>x<height> -framerate
//! <fps> -i pipe:0 -c:v <video_codec> -preset <preset> -b:v <bitrate>
//! -pix_fmt yuv420p -movflags +faststart <output_path>`
//!
//! Rationale record (plan §0.2, sources R1–R8):
//! - R1+R2+R3: rawvideo input needs explicit `pixel_format` (`rgb24`, which
//!   equals `config.video.pixel_format` — already validated by the config
//!   loader to be exactly `rgb24`), explicit `video_size` via the `-s`
//!   input shortcut, and an explicit rate via the demuxer's documented
//!   `framerate` option.
//! - R4: all input-side options before `-i pipe:0`; all output-side options
//!   after; output format is driven by the `.mp4` extension, so no `-f mp4`
//!   is passed (fewer args, doc-supported).
//! - R5: output `-pix_fmt yuv420p` is chosen for maximum
//!   player/browser/device compatibility (8-bit 4:2:0 H.264 is the most
//!   broadly decodable configuration). It is passed explicitly because
//!   ffmpeg would otherwise silently pick the encoder's best.
//! - R6: `+faststart` rewrites the container with the `moov` index at the
//!   front after encoding; it runs during FFmpeg's normal completion on our
//!   seekable file output, so success is reported only after process exit.
//! - R7: `-n` converts any accidental file-exists race into a clean
//!   non-zero FFmpeg exit instead of an interactive overwrite prompt
//!   reading our frame pipe. The filename generator already guarantees
//!   non-existent paths; `-n` is defense in depth.
//! - R8: `-nostdin` guarantees interactive command handling can never read
//!   a single byte of the frame pipe.
//! - R7/R8 position: first in the vector (global options).
//! - Program arguments are passed directly (`std::process::Command` +
//!   `.args`, no shell anywhere); the executable path comes from
//!   `config.ffmpeg.executable` (already validated non-blank).

use std::path::Path;

use crate::config::model::AppConfig;
use crate::streaming::error::StreamingError;

const REQUIRED_PIXEL_FORMAT: &str = "rgb24";
pub(super) const PIPE_SOURCE: &str = "pipe:0";
pub(super) const OUTPUT_PIXEL_FORMAT: &str = "yuv420p";
const MOVFLAGS_FASTSTART: &str = "+faststart";

pub(crate) fn build_arguments(
    config: &AppConfig,
    output_path: &Path,
) -> Result<Vec<String>, StreamingError> {
    if is_non_rgb24_pixel_format(&config.video.pixel_format) {
        return Err(StreamingError::UnsupportedPixelFormat {
            value: config.video.pixel_format.clone(),
        });
    }
    Ok(vec![
        "-nostdin".to_string(),
        "-n".to_string(),
        "-f".to_string(),
        "rawvideo".to_string(),
        "-pix_fmt".to_string(),
        config.video.pixel_format.clone(),
        "-s".to_string(),
        format!("{}x{}", config.video.width, config.video.height),
        "-framerate".to_string(),
        format!("{}", config.video.fps),
        "-i".to_string(),
        PIPE_SOURCE.to_string(),
        "-c:v".to_string(),
        config.ffmpeg.video_codec.clone(),
        "-preset".to_string(),
        config.ffmpeg.preset.clone(),
        "-b:v".to_string(),
        config.ffmpeg.bitrate.clone(),
        "-pix_fmt".to_string(),
        OUTPUT_PIXEL_FORMAT.to_string(),
        "-movflags".to_string(),
        MOVFLAGS_FASTSTART.to_string(),
        output_path.to_string_lossy().into_owned(),
    ])
}

pub(super) fn is_non_rgb24_pixel_format(pixel_format: &str) -> bool {
    pixel_format != REQUIRED_PIXEL_FORMAT
}

#[cfg(test)]
#[path = "args_tests.rs"]
mod args_tests;

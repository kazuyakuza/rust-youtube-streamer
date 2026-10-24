//! Builds the production streaming FFmpeg argument vector purely from typed,
//! already-validated configuration plus the resolved RTMPS destination. No
//! environment access, no file system, no spawning — pure and unit-testable.
//! Sibling of `args` (the local recording vector), split by the caller
//! ruling of 2026-10-24 so each file stays under its line cap.
//!
//! Locked streaming vector (plan D-T3-5 F-S2):
//! `-nostdin -n -f rawvideo -pix_fmt rgb24 -s <width>x<height> -framerate
//! <fps> -i pipe:0 -c:v <video_codec> -preset <preset> -b:v <bitrate> -g
//! <2 × fps> -pix_fmt yuv420p -f flv <destination_url>` — the input
//! segment is byte-identical to the local vector; the output group
//! replaces `+faststart` with the FLV muxer selection and the official
//! 2-second-target keyframe interval. The vector order is asserted by
//! tests and must not be reordered.
//!
//! Streaming audio extension (brief §10.4 explicit design change; plan
//! 20261024 §D4): when `ffmpeg.audio_enabled()` the streaming vector gains
//! the second input pair `-f lavfi -i <anullsrc source>` (the source token
//! is `anullsrc=channel_layout=stereo:sample_rate=48000`, kept as ONE argv
//! token) and the output tokens `-c:a aac -b:a 128k -shortest`. Constants
//! are the official-recommendation set (AAC; 44.1/48 kHz allowed — 48 kHz
//! here; 128 kbit/s stereo). Default output selection pairs the only video
//! stream with the only audio stream, so NO `-map` is needed. `-shortest`
//! sits immediately BEFORE `-f flv <url>`: an output option applied when
//! the shortest output stream ends, keeping the closed video stream the
//! terminator of the whole encode (FFmpeg HTML §5.11) so the stdin-EOF
//! shutdown stays clean and bounded; placing it before `-f flv` preserves
//! the locked `-f flv <url>` final pair asserted by the pinned tests.

use crate::config::model::AppConfig;
use crate::streaming::error::StreamingError;
use crate::streaming::rtmps_destination::RtmpsDestination;

use super::args::{is_non_rgb24_pixel_format, OUTPUT_PIXEL_FORMAT, PIPE_SOURCE};

const STREAM_MUXER: &str = "flv";
const GOP_MULTIPLIER: u32 = 2;
const AUDIO_CHANNEL_LAYOUT: &str = "stereo";
const AUDIO_SAMPLE_RATE: u32 = 48000;
const AUDIO_INPUT_FORMAT: &str = "lavfi";
const AUDIO_CODEC: &str = "aac";
const AUDIO_BITRATE: &str = "128k";
const VIDEO_SHORTEST_STREAM_BOUND: &str = "-shortest";

/// The locked streaming vector (plan D-T3-5 F-S2): the input segment is
/// byte-identical to the local vector; the output group differs by exactly
/// (i) the destination URL as the final argument, (ii) no `-movflags`
/// (MP4-only container rewrite, meaningless for the FLV muxer), (iii) the
/// explicit `-f flv` muxer (no output extension exists), and (iv) the
/// `-g <2 × fps>` keyframe interval (official guidance: target 2 s, never
/// above 4 s; derived from the configured FPS). The local
/// `build_arguments` vector stays byte-unchanged (demo never regresses).
pub(super) fn build_stream_arguments(
    config: &AppConfig,
    destination: &RtmpsDestination,
) -> Result<Vec<String>, StreamingError> {
    if is_non_rgb24_pixel_format(&config.video.pixel_format) {
        return Err(StreamingError::UnsupportedPixelFormat {
            value: config.video.pixel_format.clone(),
        });
    }
    let mut arguments = stream_input_arguments(config);
    if config.ffmpeg.audio_enabled() {
        arguments.extend(audio_input_arguments());
    }
    arguments.extend(stream_video_output_arguments(config));
    if config.ffmpeg.audio_enabled() {
        arguments.extend(audio_output_arguments());
    }
    arguments.push("-f".to_string());
    arguments.push(STREAM_MUXER.to_string());
    arguments.push(destination.as_url().to_string());
    Ok(arguments)
}

fn stream_input_arguments(config: &AppConfig) -> Vec<String> {
    vec![
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
    ]
}

fn stream_video_output_arguments(config: &AppConfig) -> Vec<String> {
    vec![
        "-c:v".to_string(),
        config.ffmpeg.video_codec.clone(),
        "-preset".to_string(),
        config.ffmpeg.preset.clone(),
        "-b:v".to_string(),
        config.ffmpeg.bitrate.clone(),
        "-g".to_string(),
        format!("{}", stream_keyframe_interval(config.video.fps)),
        "-pix_fmt".to_string(),
        OUTPUT_PIXEL_FORMAT.to_string(),
    ]
}

fn audio_source_argument() -> String {
    format!(
        "anullsrc=channel_layout={}:sample_rate={}",
        AUDIO_CHANNEL_LAYOUT, AUDIO_SAMPLE_RATE
    )
}

fn audio_input_arguments() -> [String; 4] {
    [
        "-f".to_string(),
        AUDIO_INPUT_FORMAT.to_string(),
        "-i".to_string(),
        audio_source_argument(),
    ]
}

fn audio_output_arguments() -> [String; 5] {
    [
        "-c:a".to_string(),
        AUDIO_CODEC.to_string(),
        "-b:a".to_string(),
        AUDIO_BITRATE.to_string(),
        VIDEO_SHORTEST_STREAM_BOUND.to_string(),
    ]
}

fn stream_keyframe_interval(fps: u32) -> u32 {
    fps * GOP_MULTIPLIER
}

#[cfg(test)]
#[path = "stream_args_tests.rs"]
mod stream_args_tests;

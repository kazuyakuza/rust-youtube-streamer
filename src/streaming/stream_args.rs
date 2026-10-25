//! Builds the production streaming FFmpeg argument vector purely from typed, already-validated
//! configuration plus the resolved RTMPS destination. No environment access, no file system, no
//! spawning — pure and unit-testable. Sibling of `args` (the local recording vector), split by
//! the caller ruling of 2026-10-24 so each file stays under its line cap.
//!
//! Locked streaming vector (plan D-T3-5 F-S2 + stream-health Task 2): `-nostdin -n -f rawvideo
//! -pix_fmt rgb24 -s <width>x<height> -framerate <fps> -i pipe:0 -c:v <video_codec> -preset
//! <preset> -b:v <bitrate> -maxrate <bitrate> -minrate <bitrate> -bufsize <2 × bitrate>
//! -x264-params nal-hrd=cbr -g <2 × fps> -pix_fmt yuv420p -f flv <destination_url>` — the input
//! segment is byte-identical to the local vector; the output group replaces `+faststart` with the
//! FLV muxer and the CBR rate-control block; the vector order is asserted by tests and must not be
//! reordered. The audio-OFF shapes (section absent / `enabled` absent / `false`) carry the same
//! derived CBR video parameters and keep no audio track; the demo/local `build_arguments` vector
//! and `ffmpeg.rs` stay byte-untouched.
//!
//! CBR remedy: `-maxrate`/`-minrate` mirror the configured bitrate, `-bufsize` doubles it
//! (`CBR_BUF_SIZE_MULTIPLIER`) and `-x264-params nal-hrd=cbr` engages the x264 filler, aligning
//! the measured bitrate with the target (YouTube's "not receiving enough video" warning).
//! Citations — ffmpeg-codecs: `nal-hrd` "Set signal HRD information (requires vbv-bufsize to be
//! set)", `cbr` = "Constant bit rate (not allowed in MP4 container)" and "If maxrate and minrate
//! are also set to the same value then it will use constant-bitrate mode", `minrate` "Most useful
//! in setting up a CBR encode"; FLV is not MP4, so the prohibition does not apply (FFmpeg wiki
//! Encode/H.264 §CBR: `-b:v B -minrate B -maxrate B -bufsize 2B -x264-params "nal-hrd=cbr"`).
//! Empirical FLV probe (pinned dev image, FFmpeg 5.1.9): exit 0, per-stream
//! `bit_rate=6000000`/`128000`, container total 5779767 at 10.408 s ≈ 94.3% of the declared sum;
//! the floor gate carries the tolerance. The CBR tokens are OUTPUT options and always follow the
//! audio input pair, so the input-url rejection shape is impossible.
//!
//! Streaming audio extension (brief §10.4; plan 20261024 §D4): when `ffmpeg.audio_enabled()` the
//! vector gains the second input pair `-f lavfi -i anullsrc=channel_layout=stereo:sample_rate=48000`
//! (ONE argv token) and the output tokens `-c:a libmp3lame -b:a 128k -shortest` (stream-health
//! Task 2 swap). Constants are the official set (YouTube: "AAC or MP3", 128-Kbps stereo, CBR;
//! MP3-CBR pads silence to the real bitrate where AAC-on-silence cannot). Default output
//! selection pairs the only video stream with the only audio stream, so NO `-map` is needed.
//! `-shortest` sits immediately BEFORE `-f flv <url>`: an output option applied when the shortest
//! output stream ends, keeping the closed video stream the terminator of the whole encode (FFmpeg
//! HTML §5.11) so the stdin-EOF shutdown stays clean and bounded; placing it before `-f flv`
//! preserves the locked `-f flv <url>` final pair asserted by the pinned tests. The operator's
//! FFmpeg build must include libmp3lame (standard full builds do).

use crate::config::model::AppConfig;
use crate::streaming::error::StreamingError;
use crate::streaming::rtmps_destination::RtmpsDestination;

use super::args::{is_non_rgb24_pixel_format, OUTPUT_PIXEL_FORMAT, PIPE_SOURCE};

const STREAM_MUXER: &str = "flv";
const GOP_MULTIPLIER: u32 = 2;
const AUDIO_CHANNEL_LAYOUT: &str = "stereo";
const AUDIO_SAMPLE_RATE: u32 = 48000;
const AUDIO_INPUT_FORMAT: &str = "lavfi";
const AUDIO_CODEC: &str = "libmp3lame";
const AUDIO_BITRATE: &str = "128k";
const VIDEO_SHORTEST_STREAM_BOUND: &str = "-shortest";
const VIDEO_MAXRATE_TOKEN: &str = "-maxrate";
const VIDEO_MINRATE_TOKEN: &str = "-minrate";
const VIDEO_BUFSIZE_TOKEN: &str = "-bufsize";
const VIDEO_X264_PARAMS_TOKEN: &str = "-x264-params";
const VIDEO_HRD_CBR_PARAM: &str = "nal-hrd=cbr";
/// The official x264 CBR recipe uses a 2× rate-control buffer
/// (ffmpeg wiki Encode/H.264 §CBR; `-bufsize` must exceed `-maxrate`).
const CBR_BUF_SIZE_MULTIPLIER: u32 = 2;

/// The locked streaming vector (plan D-T3-5 F-S2 + stream-health Task 2):
/// the input segment is byte-identical to the local vector; the output
/// group differs by (i) the destination URL as the final argument, (ii) no
/// `-movflags` (MP4-only container rewrite, meaningless for the FLV
/// muxer), (iii) the explicit `-f flv` muxer (no output extension exists),
/// (iv) the `-g <2 × fps>` keyframe interval (official guidance: target
/// 2 s, never above 4 s; derived from the configured FPS), and (v) the
/// derived CBR video parameters plus the MP3-CBR audio group. The local
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
    let audio_enabled = config.ffmpeg.audio_enabled();
    let mut arguments = stream_input_arguments(config);
    if audio_enabled {
        arguments.extend(audio_input_arguments());
    }
    arguments.extend(stream_video_output_arguments(config)?);
    if audio_enabled {
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

fn stream_video_output_arguments(config: &AppConfig) -> Result<Vec<String>, StreamingError> {
    let cbr_bufsize = cbr_bufsize_bitrate(&config.ffmpeg.bitrate)?;
    let arguments = vec![
        "-c:v".to_string(),
        config.ffmpeg.video_codec.clone(),
        "-preset".to_string(),
        config.ffmpeg.preset.clone(),
        "-b:v".to_string(),
        config.ffmpeg.bitrate.clone(),
        VIDEO_MAXRATE_TOKEN.to_string(),
        config.ffmpeg.bitrate.clone(),
        VIDEO_MINRATE_TOKEN.to_string(),
        config.ffmpeg.bitrate.clone(),
        VIDEO_BUFSIZE_TOKEN.to_string(),
        cbr_bufsize,
        VIDEO_X264_PARAMS_TOKEN.to_string(),
        VIDEO_HRD_CBR_PARAM.to_string(),
        "-g".to_string(),
        format!("{}", stream_keyframe_interval(config.video.fps)),
        "-pix_fmt".to_string(),
        OUTPUT_PIXEL_FORMAT.to_string(),
    ];
    Ok(arguments)
}

fn cbr_bufsize_bitrate(bitrate: &str) -> Result<String, StreamingError> {
    let digits_end = bitrate
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(bitrate.len());
    let (numeric_part, suffix) = bitrate.split_at(digits_end);
    let numeric: u64 =
        numeric_part
            .parse()
            .map_err(|_| StreamingError::UnsupportedBitrateNotation {
                value: bitrate.to_string(),
            })?;
    Ok(format!(
        "{}{}",
        numeric * u64::from(CBR_BUF_SIZE_MULTIPLIER),
        suffix
    ))
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

#[cfg(test)]
#[path = "stream_args_health_tests.rs"]
mod stream_args_health_tests;

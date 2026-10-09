//! Data model for the JSON configuration document: one strongly typed `serde`
//! struct per section. Deserialization rejects unknown fields and wrong value
//! types; semantic rules live in the sibling `validation` module. The
//! committed `config/config.example.json` mirrors this exact schema.

use serde::Deserialize;

/// Root configuration document. Every section is required, and unknown fields
/// are rejected so configuration typos fail fast instead of being ignored.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub youtube: YouTubeConfig,
    pub video: VideoConfig,
    pub renderer: RendererConfig,
    pub chat: ChatConfig,
    pub ffmpeg: FfmpegConfig,
}

/// YouTube integration settings; currently the live broadcast metadata.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YouTubeConfig {
    pub broadcast: BroadcastConfig,
}

/// Metadata for the live broadcast created on YouTube.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BroadcastConfig {
    /// Broadcast title. Must not be empty or whitespace-only.
    pub title: String,
    /// Broadcast description. Must not be empty or whitespace-only.
    pub description: String,
    /// Broadcast visibility: exactly one of `private`, `public`, or
    /// `unlisted` (case-sensitive; validated at load time).
    pub privacy_status: String,
}

/// Raw video frame geometry shared by the renderer and the FFmpeg input
/// configuration.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VideoConfig {
    /// Frame width in pixels. Must be positive.
    pub width: u32,
    /// Frame height in pixels. Must be positive; also bounds the vertical
    /// text layout validated with the renderer settings.
    pub height: u32,
    /// Frames rendered and emitted per second. Must be positive.
    pub fps: u32,
    /// Raw-frame pixel layout. The MVP pipeline supports only `rgb24` (three
    /// bytes per pixel); any other value is rejected, never substituted.
    pub pixel_format: String,
}

/// Text overlay geometry and colors for the chat renderer.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RendererConfig {
    /// Path to the font file the renderer must load. Only non-blank values
    /// are checked here; file existence belongs to the renderer, in later
    /// phases.
    pub font: String,
    /// Font size in pixels. Must be positive.
    pub font_size: u32,
    /// Vertical distance between rendered text lines in pixels. Must be
    /// positive and, together with the vertical margins and video height,
    /// must leave room for at least one line.
    pub line_height: u32,
    /// Left text margin in pixels from the frame edge.
    pub left_margin: u32,
    /// Top text margin in pixels from the frame edge.
    pub top_margin: u32,
    /// Right text margin in pixels from the frame edge.
    pub right_margin: u32,
    /// Bottom text margin in pixels from the frame edge. With the top margin
    /// and line height, must leave at least one line inside the video height.
    pub bottom_margin: u32,
    /// Text color as a plain string value, conventionally an RGB hex triplet
    /// such as `#FFFFFF`; only non-blankness is validated here.
    pub text_color: String,
    /// Background color as a plain string value, conventionally an RGB hex
    /// triplet such as `#000000`; only non-blankness is validated here.
    pub background_color: String,
}

/// Chat message persistence settings.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatConfig {
    /// Path of the append-only chat log file. Must not be empty or
    /// whitespace-only; the file is opened by the chat module in later
    /// phases.
    pub log_file: String,
}

/// Encoding settings passed to the externally provided FFmpeg process.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FfmpegConfig {
    /// FFmpeg executable location, configured explicitly for each
    /// environment. Only non-blank values are checked here; existence and
    /// process-start failures belong to the streaming module in later
    /// phases.
    pub executable: String,
    /// Video encoder name for FFmpeg (for example `libx264`). Must not be
    /// empty or whitespace-only.
    pub video_codec: String,
    /// Encoder preset (for example `veryfast`). Must not be empty or
    /// whitespace-only.
    pub preset: String,
    /// Target video bitrate in FFmpeg notation (for example `6000k`). Must
    /// not be empty or whitespace-only.
    pub bitrate: String,
}

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
    /// OAuth client and persisted-credential file locations.
    pub oauth: OauthConfig,
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
    /// Optional per-stream made-for-kids declaration, sent as
    /// `status.selfDeclaredMadeForKids` on liveBroadcasts.insert (documented
    /// insert-settable property). Absent means the field is never sent and
    /// YouTube applies its default handling (the channel's made-for-kids
    /// setting).
    pub made_for_kids: Option<bool>,
}

/// OAuth client and credential file locations. The two paths must be
/// distinct (enforced by validation, so a credential save can never
/// overwrite the client file); the example defaults live in
/// `config/config.example.json` and fall under the gitignored
/// `credentials/` directory.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OauthConfig {
    /// Path of the Google downloadable client-secret JSON file
    /// (`{"installed": {...}}`, Desktop-app client type). Only non-blank
    /// values are checked here; parsing belongs to the youtube module.
    /// The example default points into the gitignored `credentials/`
    /// directory so real client secrets stay out of source control.
    pub client_config_path: String,
    /// Path of the application-persisted credential file (issued tokens).
    /// Only non-blank values are checked here and validation additionally
    /// requires it to differ from `client_config_path`; the file is created
    /// by the credential store. The example default likewise lives under
    /// the gitignored `credentials/` directory so issued tokens stay out of
    /// source control.
    pub credentials_path: String,
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
    /// Optional silent-audio section for the production streaming vector. An
    /// absent section (or an absent/false `enabled`) keeps the streaming
    /// vector without any audio track (it still carries the derived CBR
    /// video parameters).
    pub audio: Option<AudioConfig>,
}

/// Silent-audio settings for the streaming FFmpeg vector. The demo/local
/// recording vector is unaffected: `demo` never gains audio.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    /// `true` joins a silent MP3 audio track (an `anullsrc` lavfi input at
    /// 48 kHz stereo, 128 kbit/s, cut by `-shortest`) to the STREAMING
    /// vector. Absent or `false` keeps the stream without any audio track
    /// (the CBR video parameters stay).
    pub enabled: Option<bool>,
}

impl FfmpegConfig {
    /// Whether the production streaming vector extends with the silent
    /// audio track: the section and the flag are both optional, so
    /// section-absent, `enabled`-absent, and `false` all mean video-only.
    pub(crate) fn audio_enabled(&self) -> bool {
        self.audio
            .as_ref()
            .and_then(|audio| audio.enabled)
            .unwrap_or(false)
    }
}

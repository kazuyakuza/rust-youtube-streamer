use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub youtube: YouTubeConfig,
    pub video: VideoConfig,
    pub renderer: RendererConfig,
    pub chat: ChatConfig,
    pub ffmpeg: FfmpegConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YouTubeConfig {
    pub broadcast: BroadcastConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BroadcastConfig {
    pub title: String,
    pub description: String,
    pub privacy_status: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VideoConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub pixel_format: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RendererConfig {
    pub font: String,
    pub font_size: u32,
    pub line_height: u32,
    pub left_margin: u32,
    pub top_margin: u32,
    pub right_margin: u32,
    pub bottom_margin: u32,
    pub text_color: String,
    pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatConfig {
    pub log_file: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FfmpegConfig {
    pub executable: String,
    pub video_codec: String,
    pub preset: String,
    pub bitrate: String,
}

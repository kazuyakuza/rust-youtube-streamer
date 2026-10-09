//! Test-only builders for an in-memory [`AppConfig`] mirroring the values of
//! the committed `config/config.example.json`, so tests never depend on a
//! developer's real configuration file.

use super::*;

pub(super) fn valid_config() -> AppConfig {
    AppConfig {
        youtube: YouTubeConfig {
            broadcast: BroadcastConfig {
                title: "Rust YouTube Streamer Prototype".to_string(),
                description: "YouTube Live streaming prototype".to_string(),
                privacy_status: "unlisted".to_string(),
            },
        },
        video: VideoConfig {
            width: 1920,
            height: 1080,
            fps: 30,
            pixel_format: "rgb24".to_string(),
        },
        renderer: RendererConfig {
            font: "fonts/console.ttf".to_string(),
            font_size: 32,
            line_height: 40,
            left_margin: 20,
            top_margin: 20,
            right_margin: 20,
            bottom_margin: 20,
            text_color: "#FFFFFF".to_string(),
            background_color: "#000000".to_string(),
        },
        chat: ChatConfig {
            log_file: "logs/chat.log".to_string(),
        },
        ffmpeg: FfmpegConfig {
            executable: "ffmpeg".to_string(),
            video_codec: "libx264".to_string(),
            preset: "veryfast".to_string(),
            bitrate: "6000k".to_string(),
        },
    }
}

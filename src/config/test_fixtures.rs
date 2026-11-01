//! Test-only builders for an in-memory [`AppConfig`] mirroring the values of
//! the committed `config/config.example.json`, so tests never depend on a
//! developer's real configuration file. The temp-file helpers live here so
//! every parse roster shares one writer instead of duplicating it.

use std::path::PathBuf;

use crate::config::diagnostics_config::DiagnosticsConfig;
use crate::config::error::ConfigError;
use crate::config::load_from_path;
use crate::config::model::{
    AppConfig, AudioConfig, BroadcastConfig, ChatConfig, FfmpegConfig, OauthConfig, RendererConfig,
    VideoConfig, YouTubeConfig,
};

pub(crate) fn valid_config() -> AppConfig {
    AppConfig {
        youtube: YouTubeConfig {
            broadcast: BroadcastConfig {
                title: "Rust YouTube Streamer Prototype".to_string(),
                description: "YouTube Live streaming prototype".to_string(),
                privacy_status: "unlisted".to_string(),
                made_for_kids: Some(false),
            },
        },
        oauth: OauthConfig {
            client_config_path: "credentials/client_secret.json".to_string(),
            credentials_path: "credentials/tokens.json".to_string(),
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
            audio: Some(AudioConfig {
                enabled: Some(true),
            }),
        },
        diagnostics: Some(DiagnosticsConfig {
            pipeline_timing_events: Some(false),
            queue_metrics: Some(false),
            periodic_summary: Some(false),
            summary_interval_seconds: Some(30),
            output_file: Some("logs/diagnostics.log".to_string()),
            max_file_bytes: Some(1_048_576),
            max_generations: Some(3),
        }),
    }
}

pub(crate) fn expect_load_error(path: &std::path::Path) -> ConfigError {
    match load_from_path(path) {
        Err(error) => error,
        Ok(_) => panic!("expected configuration loading to fail"),
    }
}

pub(crate) fn write_temp_config(dir: &tempfile::TempDir, contents: &str) -> PathBuf {
    let path = dir.path().join("config.json");
    std::fs::write(&path, contents).expect("test config file must be writable");
    path
}

pub(crate) fn example_json() -> String {
    r##"{
  "youtube": {
    "broadcast": {
      "title": "Rust YouTube Streamer Prototype",
      "description": "YouTube Live streaming prototype",
      "privacy_status": "unlisted",
      "made_for_kids": false
    }
  },
  "oauth": {
    "client_config_path": "credentials/client_secret.json",
    "credentials_path": "credentials/tokens.json"
  },
  "video": {
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "pixel_format": "rgb24"
  },
  "renderer": {
    "font": "fonts/console.ttf",
    "font_size": 32,
    "line_height": 40,
    "left_margin": 20,
    "top_margin": 20,
    "right_margin": 20,
    "bottom_margin": 20,
    "text_color": "#FFFFFF",
    "background_color": "#000000"
  },
  "chat": {
    "log_file": "logs/chat.log"
  },
  "ffmpeg": {
    "executable": "ffmpeg",
    "video_codec": "libx264",
    "preset": "veryfast",
    "bitrate": "6000k",
    "audio": { "enabled": true }
  },
  "diagnostics": {
    "pipeline_timing_events": false,
    "queue_metrics": false,
    "periodic_summary": false,
    "summary_interval_seconds": 30,
    "output_file": "logs/diagnostics.log",
    "max_file_bytes": 1048576,
    "max_generations": 3
  }
}"##
    .to_string()
}

pub(crate) fn example_json_value() -> serde_json::Value {
    serde_json::from_str(&example_json()).expect("embedded example JSON must be valid")
}

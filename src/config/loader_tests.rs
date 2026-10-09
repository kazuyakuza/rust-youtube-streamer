use std::path::PathBuf;

use crate::config::model::{
    BroadcastConfig, ChatConfig, FfmpegConfig, RendererConfig, VideoConfig, YouTubeConfig,
};

use super::*;

#[test]
fn loads_valid_example_config() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = write_temp_config(&dir, &example_json());
    let loaded = load_from_path(&path).expect("example config must load");
    assert_eq!(loaded, example_app_config());
}

#[test]
fn missing_file_reports_io_error() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("config.json");
    let error = expect_load_error(&path);
    assert!(matches!(error, ConfigError::Io { .. }));
    assert!(error.to_string().contains(&path.display().to_string()));
}

#[test]
fn unreadable_file_reports_io_error() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let directory_path = dir.path().join("some-dir");
    std::fs::create_dir(&directory_path).expect("test directory must be created");
    let error = expect_load_error(&directory_path);
    assert!(matches!(error, ConfigError::Io { .. }));
    let message = error.to_string();
    assert!(message.contains("unable to read configuration file"));
    assert!(message.contains(&directory_path.display().to_string()));
}

#[test]
fn malformed_json_reports_deserialization_error() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = write_temp_config(&dir, "{\"youtube\": {");
    let error = expect_load_error(&path);
    assert!(matches!(error, ConfigError::Deserialization { .. }));
    assert!(error.to_string().contains(&path.display().to_string()));
}

#[test]
fn missing_required_field_reports_error() {
    let dir = tempfile::tempdir().expect("temp dir must be created");

    let mut without_video_section = example_json_value();
    without_video_section
        .as_object_mut()
        .expect("example JSON must be an object")
        .remove("video");
    let path_without_video = write_temp_config(&dir, &without_video_section.to_string());
    let error_without_video = expect_load_error(&path_without_video);
    assert!(matches!(
        error_without_video,
        ConfigError::Deserialization { .. }
    ));
    assert!(error_without_video.to_string().contains("missing field"));

    let mut without_bitrate = example_json_value();
    without_bitrate["ffmpeg"]
        .as_object_mut()
        .expect("ffmpeg JSON must be an object")
        .remove("bitrate");
    let path_without_bitrate = write_temp_config(&dir, &without_bitrate.to_string());
    let error_without_bitrate = expect_load_error(&path_without_bitrate);
    assert!(matches!(
        error_without_bitrate,
        ConfigError::Deserialization { .. }
    ));
    assert!(error_without_bitrate.to_string().contains("missing field"));
}

#[test]
fn unknown_field_rejected() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let mut value = example_json_value();
    value["renderer"]
        .as_object_mut()
        .expect("renderer JSON must be an object")
        .insert("max_messages".to_string(), serde_json::json!(10));
    let path = write_temp_config(&dir, &value.to_string());
    let error = expect_load_error(&path);
    assert!(matches!(error, ConfigError::Deserialization { .. }));
}

#[test]
fn negative_number_rejected() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let mut value = example_json_value();
    value["video"]["width"] = serde_json::Value::from(-5);
    let path = write_temp_config(&dir, &value.to_string());
    let error = expect_load_error(&path);
    assert!(matches!(error, ConfigError::Deserialization { .. }));
}

fn expect_load_error(path: &std::path::Path) -> ConfigError {
    match load_from_path(path) {
        Err(error) => error,
        Ok(_) => panic!("expected configuration loading to fail"),
    }
}

fn write_temp_config(dir: &tempfile::TempDir, contents: &str) -> PathBuf {
    let path = dir.path().join("config.json");
    std::fs::write(&path, contents).expect("test config file must be writable");
    path
}

fn example_json() -> String {
    r##"{
  "youtube": {
    "broadcast": {
      "title": "Rust YouTube Streamer Prototype",
      "description": "YouTube Live streaming prototype",
      "privacy_status": "unlisted"
    }
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
    "bitrate": "6000k"
  }
}"##
    .to_string()
}

fn example_json_value() -> serde_json::Value {
    serde_json::from_str(&example_json()).expect("embedded example JSON must be valid")
}

fn example_app_config() -> AppConfig {
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

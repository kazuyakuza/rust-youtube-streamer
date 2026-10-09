use crate::config::model::{BroadcastConfig, ChatConfig, FfmpegConfig, YouTubeConfig};
use crate::config::validation::validate;

use super::*;

#[test]
fn zero_width_rejected() {
    let mut config = valid_config();
    config.video.width = 0;
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("video.width"));
}

#[test]
fn zero_height_or_fps_rejected() {
    let mut config_with_zero_height = valid_config();
    config_with_zero_height.video.height = 0;
    let height_message = expect_validation_error(&config_with_zero_height).to_string();
    assert!(height_message.contains("video.height"));

    let mut config_with_zero_fps = valid_config();
    config_with_zero_fps.video.fps = 0;
    let fps_message = expect_validation_error(&config_with_zero_fps).to_string();
    assert!(fps_message.contains("video.fps"));
}

#[test]
fn unsupported_pixel_format_rejected() {
    let mut config = valid_config();
    config.video.pixel_format = "rgba".to_string();
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("video.pixel_format"));
    assert!(message.contains("rgb24"));
}

#[test]
fn zero_font_size_or_line_height_rejected() {
    let mut config_with_zero_font_size = valid_config();
    config_with_zero_font_size.renderer.font_size = 0;
    let font_size_message = expect_validation_error(&config_with_zero_font_size).to_string();
    assert!(font_size_message.contains("renderer.font_size"));

    let mut config_with_zero_line_height = valid_config();
    config_with_zero_line_height.renderer.line_height = 0;
    let line_height_message = expect_validation_error(&config_with_zero_line_height).to_string();
    assert!(line_height_message.contains("renderer.line_height"));
}

#[test]
fn vertical_layout_without_one_line_rejected() {
    let mut config = valid_config();
    config.video.height = 100;
    config.renderer.top_margin = 40;
    config.renderer.bottom_margin = 40;
    config.renderer.line_height = 40;
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("vertical layout"));
    assert!(message.contains("height 100"));
    assert!(message.contains("line_height 40"));
}

#[test]
fn vertical_layout_boundary_accepts_one_line() {
    let mut config = valid_config();
    config.video.height = 100;
    config.renderer.top_margin = 30;
    config.renderer.bottom_margin = 30;
    config.renderer.line_height = 40;
    assert!(validate(&config).is_ok());
}

#[test]
fn unsupported_privacy_status_rejected() {
    let mut config_with_internal = valid_config();
    config_with_internal.youtube.broadcast.privacy_status = "internal".to_string();
    let internal_message = expect_validation_error(&config_with_internal).to_string();
    assert!(internal_message.contains("youtube.broadcast.privacy_status"));
    assert!(internal_message.contains("private, public, unlisted"));

    let mut config_with_case_mismatch = valid_config();
    config_with_case_mismatch.youtube.broadcast.privacy_status = "Public".to_string();
    let case_mismatch_message = expect_validation_error(&config_with_case_mismatch).to_string();
    assert!(case_mismatch_message.contains("youtube.broadcast.privacy_status"));
}

#[test]
fn empty_required_value_rejected() {
    let mut config_with_blank_title = valid_config();
    config_with_blank_title.youtube.broadcast.title = "  ".to_string();
    assert!(expect_validation_error(&config_with_blank_title)
        .to_string()
        .contains("youtube.broadcast.title"));

    let mut config_with_blank_description = valid_config();
    config_with_blank_description.youtube.broadcast.description = "".to_string();
    assert!(expect_validation_error(&config_with_blank_description)
        .to_string()
        .contains("youtube.broadcast.description"));

    let mut config_with_blank_font = valid_config();
    config_with_blank_font.renderer.font = "   ".to_string();
    assert!(expect_validation_error(&config_with_blank_font)
        .to_string()
        .contains("renderer.font"));

    let mut config_with_blank_log_file = valid_config();
    config_with_blank_log_file.chat.log_file = "".to_string();
    assert!(expect_validation_error(&config_with_blank_log_file)
        .to_string()
        .contains("chat.log_file"));

    let mut config_with_blank_executable = valid_config();
    config_with_blank_executable.ffmpeg.executable = "".to_string();
    assert!(expect_validation_error(&config_with_blank_executable)
        .to_string()
        .contains("ffmpeg.executable"));

    let mut config_with_blank_video_codec = valid_config();
    config_with_blank_video_codec.ffmpeg.video_codec = "".to_string();
    assert!(expect_validation_error(&config_with_blank_video_codec)
        .to_string()
        .contains("ffmpeg.video_codec"));

    let mut config_with_blank_preset = valid_config();
    config_with_blank_preset.ffmpeg.preset = " ".to_string();
    assert!(expect_validation_error(&config_with_blank_preset)
        .to_string()
        .contains("ffmpeg.preset"));

    let mut config_with_blank_bitrate = valid_config();
    config_with_blank_bitrate.ffmpeg.bitrate = "".to_string();
    assert!(expect_validation_error(&config_with_blank_bitrate)
        .to_string()
        .contains("ffmpeg.bitrate"));
}

#[test]
fn multiple_issues_reported_together() {
    let mut config = valid_config();
    config.video.width = 0;
    config.video.fps = 0;
    config.youtube.broadcast.privacy_status = "internal".to_string();
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("video.width"));
    assert!(message.contains("video.fps"));
    assert!(message.contains("youtube.broadcast.privacy_status"));
}

fn expect_validation_error(config: &AppConfig) -> ConfigError {
    match validate(config) {
        Err(error @ ConfigError::Validation { .. }) => error,
        Err(other) => panic!("expected a validation error, got: {other}"),
        Ok(()) => panic!("expected a validation error, got Ok"),
    }
}

fn valid_config() -> AppConfig {
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

//! Validation-rule tests: geometry, pixel format, privacy status, blank
//! values, and multi-issue aggregation, run on in-memory configuration
//! variants.

use crate::config::test_fixtures::valid_config;
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
    assert_blank_value_rejected("youtube.broadcast.title", |c| {
        c.youtube.broadcast.title = "  ".to_string()
    });
    assert_blank_value_rejected("youtube.broadcast.description", |c| {
        c.youtube.broadcast.description = "".to_string()
    });
    assert_blank_value_rejected("renderer.font", |c| c.renderer.font = "   ".to_string());
    assert_blank_value_rejected("chat.log_file", |c| c.chat.log_file = "".to_string());
    assert_blank_value_rejected("ffmpeg.executable", |c| {
        c.ffmpeg.executable = "".to_string()
    });
    assert_blank_value_rejected("ffmpeg.video_codec", |c| {
        c.ffmpeg.video_codec = "".to_string()
    });
    assert_blank_value_rejected("ffmpeg.preset", |c| c.ffmpeg.preset = " ".to_string());
    assert_blank_value_rejected("ffmpeg.bitrate", |c| c.ffmpeg.bitrate = "".to_string());
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

fn assert_blank_value_rejected(field: &str, make_blank: impl FnOnce(&mut AppConfig)) {
    let mut config = valid_config();
    make_blank(&mut config);
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains(field));
}

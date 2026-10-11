//! `RendererSettings` derivation and hex color parsing tests. Config
//! structs are constructed in place; the configuration validation module
//! is intentionally not invoked. Assertions are deterministic and use no
//! golden image files.

use crate::config::model::{RendererConfig, VideoConfig};

use super::*;

fn base_renderer_config() -> RendererConfig {
    RendererConfig {
        font: "fonts/Exo2-Light.otf".to_string(),
        font_size: 24,
        line_height: 40,
        left_margin: 20,
        top_margin: 20,
        right_margin: 20,
        bottom_margin: 20,
        text_color: "#FFFFFF".to_string(),
        background_color: "#000000".to_string(),
    }
}

fn base_video_config() -> VideoConfig {
    VideoConfig {
        width: 200,
        height: 200,
        fps: 30,
        pixel_format: "rgb24".to_string(),
    }
}

#[test]
fn settings_parse_mixed_case_hex_colors() {
    let mixed = settings::Rgb::parse_hex("#AbC012").expect("mixed case parses");
    let upper = settings::Rgb::parse_hex("#ABC012").expect("uppercase parses");
    assert_eq!(
        mixed,
        settings::Rgb {
            red: 0xAB,
            green: 0xC0,
            blue: 0x12,
        }
    );
    assert_eq!(mixed, upper);
}

#[test]
fn settings_reject_invalid_color_shapes() {
    for shape in ["#FFF", "FFFFFF", "#GGHHII", "#ABC0123", ""] {
        let mut config = base_renderer_config();
        config.text_color = shape.to_string();
        let error = settings::RendererSettings::from_config(&config, &base_video_config())
            .expect_err("invalid text color must be rejected");
        match &error {
            error::RenderError::InvalidColor { field, value } => {
                assert_eq!(*field, "text_color");
                assert_eq!(value, shape);
            }
            other => panic!("expected InvalidColor, got {other:?}"),
        }
    }
    let mut background_config = base_renderer_config();
    background_config.background_color = "not-a-color".to_string();
    let error = settings::RendererSettings::from_config(&background_config, &base_video_config())
        .expect_err("invalid background color must be rejected");
    match &error {
        error::RenderError::InvalidColor { field, .. } => assert_eq!(*field, "background_color"),
        other => panic!("expected InvalidColor, got {other:?}"),
    }
}

#[test]
fn settings_from_config_rejects_invalid_vertical_layout() {
    let mut config = base_renderer_config();
    config.top_margin = 80;
    config.bottom_margin = 80;
    config.line_height = 60;
    let error = settings::RendererSettings::from_config(&config, &base_video_config())
        .expect_err("impossible layout must be rejected");
    match error {
        error::RenderError::Layout {
            height,
            top_margin,
            bottom_margin,
            line_height,
        } => assert_eq!(
            (height, top_margin, bottom_margin, line_height),
            (200, 80, 80, 60)
        ),
        other => panic!("expected Layout, got {other:?}"),
    }
}

#[test]
fn settings_from_config_rejects_zero_frame_dimensions() {
    let video = VideoConfig {
        width: 0,
        ..base_video_config()
    };
    let error = settings::RendererSettings::from_config(&base_renderer_config(), &video)
        .expect_err("zero width must be rejected");
    assert!(matches!(
        error,
        error::RenderError::Geometry {
            width: 0,
            height: 200
        }
    ));
}

//! Shared constants and helpers for the renderer text tests, split out of
//! `text_tests.rs` (review-fix Fix 1; bodies relocated verbatim).

use std::ops::Range;

use crate::config::model::{RendererConfig, VideoConfig};

use super::contract::{RenderInput, Renderer};
use super::*;

pub(super) const WIDTH: u32 = 200;
pub(super) const HEIGHT: u32 = 200;
pub(super) const TOP_MARGIN: u32 = 20;
pub(super) const LINE_HEIGHT: u32 = 40;
pub(super) const LEFT_MARGIN: u32 = 20;
pub(super) const BOTTOM_MARGIN: u32 = 20;
pub(super) const FONT_SIZE: u32 = 24;
pub(super) const FONT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/Exo2-Light.otf");
pub(super) const SAMPLE_TEXT: &str = "abcdefghijklmnopqrstuvwxyz";

pub(super) fn base_renderer_config() -> RendererConfig {
    RendererConfig {
        font: FONT_PATH.to_string(),
        font_size: FONT_SIZE,
        line_height: LINE_HEIGHT,
        left_margin: LEFT_MARGIN,
        top_margin: TOP_MARGIN,
        right_margin: 20,
        bottom_margin: BOTTOM_MARGIN,
        text_color: "#FFFFFF".to_string(),
        background_color: "#000000".to_string(),
    }
}

pub(super) fn base_video_config() -> VideoConfig {
    VideoConfig {
        width: WIDTH,
        height: HEIGHT,
        fps: 30,
        pixel_format: "rgb24".to_string(),
    }
}

pub(super) fn settings_with_font(font: &str) -> settings::RendererSettings {
    let mut config = base_renderer_config();
    config.font = font.to_string();
    settings::RendererSettings::from_config(&config, &base_video_config())
        .expect("valid test settings")
}

pub(super) fn default_renderer() -> text::TextRenderer {
    text::TextRenderer::new(settings_with_font(FONT_PATH)).expect("vendored font must load")
}

pub(super) fn new_renderer_or_error(settings: settings::RendererSettings) -> error::RenderError {
    match text::TextRenderer::new(settings) {
        Err(error) => error,
        Ok(_) => panic!("renderer construction must fail for this input"),
    }
}

pub(super) fn render_frame(renderer: &mut text::TextRenderer, content: &str) -> frame::Frame {
    let input = RenderInput {
        text: content.to_string(),
    };
    renderer.render(&input).expect("render succeeds")
}

pub(super) fn pixel_at(frame: &frame::Frame, pixel: (u32, u32)) -> [u8; 3] {
    let (column, row) = pixel;
    let base = (row as usize * frame.width() as usize + column as usize) * 3;
    let pixels = frame.pixels();
    [pixels[base], pixels[base + 1], pixels[base + 2]]
}

pub(super) fn has_ink_at(frame: &frame::Frame, pixel: (u32, u32)) -> bool {
    pixel_at(frame, pixel).iter().any(|channel| *channel > 0)
}

pub(super) fn count_ink_in_row(frame: &frame::Frame, row: u32) -> usize {
    let mut count = 0;
    for column in 0..frame.width() {
        if has_ink_at(frame, (column, row)) {
            count += 1;
        }
    }
    count
}

pub(super) fn count_ink_in_rows(frame: &frame::Frame, rows: Range<u32>) -> usize {
    rows.map(|row| count_ink_in_row(frame, row)).sum()
}

pub(super) fn count_ink_in_column(frame: &frame::Frame, column: u32) -> usize {
    let mut count = 0;
    for row in 0..frame.height() {
        if has_ink_at(frame, (column, row)) {
            count += 1;
        }
    }
    count
}

pub(super) fn count_ink_in_columns(frame: &frame::Frame, columns: Range<u32>) -> usize {
    columns
        .map(|column| count_ink_in_column(frame, column))
        .sum()
}

pub(super) fn first_ink_row(frame: &frame::Frame) -> u32 {
    (0..frame.height())
        .find(|&row| count_ink_in_row(frame, row) > 0)
        .unwrap_or(frame.height())
}

pub(super) fn first_ink_column(frame: &frame::Frame, rows: Range<u32>) -> u32 {
    (0..frame.width())
        .find(|&column| rows.clone().any(|row| has_ink_at(frame, (column, row))))
        .unwrap_or(frame.width())
}

pub(super) fn find_pixel_in_band(
    frame: &frame::Frame,
    predicate: impl Fn(&[u8; 3]) -> bool,
) -> Option<(u32, u32)> {
    (TOP_MARGIN..TOP_MARGIN + LINE_HEIGHT)
        .flat_map(|row| (0..frame.width()).map(move |column| (column, row)))
        .find(|&(column, row)| predicate(&pixel_at(frame, (column, row))))
}

pub(super) fn is_full_white(pixel: &[u8; 3]) -> bool {
    *pixel == [255, 255, 255]
}

pub(super) fn is_partial_gray(pixel: &[u8; 3]) -> bool {
    pixel[0] > 0 && pixel[0] < 255
}

pub(super) fn is_grayscale(pixel: &[u8]) -> bool {
    pixel[0] == pixel[1] && pixel[1] == pixel[2]
}

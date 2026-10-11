//! `TextRenderer` rendering tests: deterministic pixel assertions against
//! the vendored Exo 2 Light font (never `console.ttf`); no golden files.

use std::ops::Range;

use crate::config::model::{RendererConfig, VideoConfig};

use super::contract::{RenderInput, Renderer};
use super::*;

const WIDTH: u32 = 200;
const HEIGHT: u32 = 200;
const TOP_MARGIN: u32 = 20;
const LINE_HEIGHT: u32 = 40;
const LEFT_MARGIN: u32 = 20;
const BOTTOM_MARGIN: u32 = 20;
const FONT_SIZE: u32 = 24;
const FONT_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/Exo2-Light.otf");
const SAMPLE_TEXT: &str = "abcdefghijklmnopqrstuvwxyz";

fn base_renderer_config() -> RendererConfig {
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

fn base_video_config() -> VideoConfig {
    VideoConfig {
        width: WIDTH,
        height: HEIGHT,
        fps: 30,
        pixel_format: "rgb24".to_string(),
    }
}

fn settings_with_font(font: &str) -> settings::RendererSettings {
    let mut config = base_renderer_config();
    config.font = font.to_string();
    settings::RendererSettings::from_config(&config, &base_video_config())
        .expect("valid test settings")
}

fn default_renderer() -> text::TextRenderer {
    match text::TextRenderer::new(settings_with_font(FONT_PATH)) {
        Ok(renderer) => renderer,
        Err(error) => panic!("vendored font must load: {error}"),
    }
}

fn new_renderer_or_error(settings: settings::RendererSettings) -> error::RenderError {
    match text::TextRenderer::new(settings) {
        Err(error) => error,
        Ok(_) => panic!("renderer construction must fail for this input"),
    }
}

fn render_frame(renderer: &mut text::TextRenderer, content: &str) -> frame::Frame {
    let input = RenderInput {
        text: content.to_string(),
    };
    renderer.render(&input).expect("render succeeds")
}

fn pixel_at(frame: &frame::Frame, pixel: (u32, u32)) -> [u8; 3] {
    let (column, row) = pixel;
    let base = (row as usize * frame.width() as usize + column as usize) * 3;
    let pixels = frame.pixels();
    [pixels[base], pixels[base + 1], pixels[base + 2]]
}

fn has_ink_at(frame: &frame::Frame, pixel: (u32, u32)) -> bool {
    pixel_at(frame, pixel).iter().any(|channel| *channel > 0)
}

fn count_ink_in_row(frame: &frame::Frame, row: u32) -> usize {
    let mut count = 0;
    for column in 0..frame.width() {
        if has_ink_at(frame, (column, row)) {
            count += 1;
        }
    }
    count
}

fn count_ink_in_rows(frame: &frame::Frame, rows: Range<u32>) -> usize {
    rows.map(|row| count_ink_in_row(frame, row)).sum()
}

fn count_ink_in_column(frame: &frame::Frame, column: u32) -> usize {
    let mut count = 0;
    for row in 0..frame.height() {
        if has_ink_at(frame, (column, row)) {
            count += 1;
        }
    }
    count
}

fn count_ink_in_columns(frame: &frame::Frame, columns: Range<u32>) -> usize {
    columns
        .map(|column| count_ink_in_column(frame, column))
        .sum()
}

fn first_ink_row(frame: &frame::Frame) -> u32 {
    (0..frame.height())
        .find(|&row| count_ink_in_row(frame, row) > 0)
        .unwrap_or(frame.height())
}

fn first_ink_column(frame: &frame::Frame, rows: Range<u32>) -> u32 {
    (0..frame.width())
        .find(|&column| rows.clone().any(|row| has_ink_at(frame, (column, row))))
        .unwrap_or(frame.width())
}

fn find_pixel_in_band(
    frame: &frame::Frame,
    predicate: impl Fn(&[u8; 3]) -> bool,
) -> Option<(u32, u32)> {
    (TOP_MARGIN..TOP_MARGIN + LINE_HEIGHT)
        .flat_map(|row| (0..frame.width()).map(move |column| (column, row)))
        .find(|&(column, row)| predicate(&pixel_at(frame, (column, row))))
}

fn is_full_white(pixel: &[u8; 3]) -> bool {
    *pixel == [255, 255, 255]
}

fn is_partial_gray(pixel: &[u8; 3]) -> bool {
    pixel[0] > 0 && pixel[0] < 255
}

fn is_grayscale(pixel: &[u8]) -> bool {
    pixel[0] == pixel[1] && pixel[1] == pixel[2]
}

#[test]
fn missing_font_file_returns_typed_error() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let missing = temp.path().join("no-such-font.otf");
    let error = new_renderer_or_error(settings_with_font(&missing.to_string_lossy()));
    assert!(matches!(error, error::RenderError::FontOpen { .. }));
    assert!(error.to_string().contains(&missing.display().to_string()));
}

#[test]
fn garbage_font_file_returns_parse_error() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let garbage = temp.path().join("garbage.otf");
    std::fs::write(&garbage, [7u8; 64]).expect("write garbage font bytes");
    let error = new_renderer_or_error(settings_with_font(&garbage.to_string_lossy()));
    assert!(matches!(error, error::RenderError::FontParse { .. }));
}

#[test]
fn invalid_color_at_construction() {
    let mut config = base_renderer_config();
    config.text_color = "#XYZ123".to_string();
    let error = settings::RendererSettings::from_config(&config, &base_video_config())
        .expect_err("invalid color must be rejected");
    assert!(matches!(
        error,
        error::RenderError::InvalidColor {
            field: "text_color",
            ..
        }
    ));
}

#[test]
fn empty_text_yields_completely_black_frame() {
    let mut renderer = default_renderer();
    let frame = render_frame(&mut renderer, "");
    assert_eq!(frame.pixels().len(), (WIDTH * HEIGHT * 3) as usize);
    assert!(frame.pixels().iter().all(|byte| *byte == 0));
}

#[test]
fn multiline_orders_top_to_bottom() {
    let mut renderer = default_renderer();
    let frame = render_frame(&mut renderer, "abcdefgh\nijklmnop");
    assert_eq!(count_ink_in_rows(&frame, 0..TOP_MARGIN), 0);
    assert!(count_ink_in_rows(&frame, TOP_MARGIN..(TOP_MARGIN + LINE_HEIGHT)) > 0);
    assert!(
        count_ink_in_rows(
            &frame,
            (TOP_MARGIN + LINE_HEIGHT)..(TOP_MARGIN + 2 * LINE_HEIGHT)
        ) > 0
    );
    assert_eq!(
        count_ink_in_rows(&frame, (TOP_MARGIN + 2 * LINE_HEIGHT)..HEIGHT),
        0
    );
}

#[test]
fn margins_take_effect() {
    let mut renderer = default_renderer();
    let frame = render_frame(&mut renderer, "aaaa");
    let first_column = first_ink_column(&frame, TOP_MARGIN..(TOP_MARGIN + LINE_HEIGHT));
    assert!(first_column >= LEFT_MARGIN);
    assert!(first_ink_row(&frame) >= TOP_MARGIN);
}

#[test]
fn horizontal_clips_at_frame_edge() {
    let mut renderer = default_renderer();
    let frame = render_frame(&mut renderer, &"a".repeat(400));
    assert_eq!(frame.pixels().len(), (WIDTH * HEIGHT * 3) as usize);
    assert!(count_ink_in_columns(&frame, WIDTH - 1..WIDTH) > 0);
}

#[test]
fn vertical_clips_at_frame_bottom() {
    let mut renderer = default_renderer();
    let frame = render_frame(&mut renderer, &"a\n".repeat(20));
    assert_eq!(frame.pixels().len(), (WIDTH * HEIGHT * 3) as usize);
    assert!(count_ink_in_rows(&frame, 0..HEIGHT) > 0);
    assert!(count_ink_in_rows(&frame, HEIGHT - 3..HEIGHT) > 0);
}

#[test]
fn white_text_on_black_background_with_default_colors() {
    let mut renderer = default_renderer();
    let frame = render_frame(&mut renderer, SAMPLE_TEXT);
    assert_eq!(pixel_at(&frame, (0, 0)), [0, 0, 0]);
    find_pixel_in_band(&frame, is_full_white).expect("full white pixel exists");
    assert!(frame.pixels().chunks(3).all(is_grayscale));
}

#[test]
fn coverage_blend_is_neither_background_nor_text() {
    let mut renderer = default_renderer();
    let frame = render_frame(&mut renderer, SAMPLE_TEXT);
    find_pixel_in_band(&frame, is_partial_gray).expect("antialiased pixel exists");
}

#[test]
fn render_is_repeatable() {
    let mut renderer = default_renderer();
    let first = render_frame(&mut renderer, SAMPLE_TEXT);
    let second = render_frame(&mut renderer, SAMPLE_TEXT);
    assert_eq!(first.pixels(), second.pixels());
}

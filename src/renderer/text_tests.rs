//! `TextRenderer` rendering tests: deterministic pixel assertions against
//! the vendored Exo 2 Light font (never `console.ttf`); no golden files.

use super::*;
use crate::renderer::text_test_helpers::*;

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

//! `Frame` construction and blending tests.

use super::*;

#[test]
fn frame_filled_has_exact_byte_count() {
    let frame = frame::Frame::filled(64, 24, settings::Rgb::black()).expect("filled frame");
    assert_eq!(frame.pixels().len(), 64 * 24 * 3);
    assert_eq!(frame.pixels().len(), 4608);
    assert_eq!(frame.width(), 64);
    assert_eq!(frame.height(), 24);
}

#[test]
fn frame_filled_is_uniform_color() {
    let color = settings::Rgb {
        red: 10,
        green: 20,
        blue: 30,
    };
    let frame = frame::Frame::filled(8, 4, color).expect("filled frame");
    for pixel in frame.pixels().chunks(3) {
        assert_eq!(pixel[0], color.red);
        assert_eq!(pixel[1], color.green);
        assert_eq!(pixel[2], color.blue);
    }
}

#[test]
fn frame_rejects_zero_or_overflowed_sizes() {
    let zero = frame::Frame::filled(0, 24, settings::Rgb::black())
        .expect_err("zero width must be rejected");
    assert!(matches!(zero, error::RenderError::Geometry { .. }));
    let overflow = frame::Frame::filled(u32::MAX, u32::MAX, settings::Rgb::black())
        .expect_err("overflowing size must be rejected");
    assert!(matches!(overflow, error::RenderError::FrameSize { .. }));
}

#[test]
fn blend_at_within_bounds_only_paints_inside() {
    let paint = settings::Rgb {
        red: 9,
        green: 9,
        blue: 9,
    };
    let mut frame = frame::Frame::filled(4, 4, settings::Rgb::black()).expect("filled frame");
    let outside = frame::PixelBlend {
        coverage: 1.0,
        color: paint,
    };
    frame.blend_at(frame::PixelPoint { x: 4, y: 0 }, outside);
    frame.blend_at(frame::PixelPoint { x: 99, y: 99 }, outside);
    assert!(frame.pixels().iter().all(|byte| *byte == 0));
    frame.blend_at(frame::PixelPoint { x: 1, y: 2 }, outside);
    let base = (2 * 4 + 1) * 3;
    assert_eq!(&frame.pixels()[base..base + 3], &[9, 9, 9]);
    assert_eq!(frame.pixels()[0], 0);
    assert_eq!(frame.pixels()[base - 1], 0);
}

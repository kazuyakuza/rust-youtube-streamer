//! RGB24 frame buffer: checked-size construction and clipped coverage
//! blending. Every frame is exactly `width * height * 3` bytes, row-major,
//! origin at the top-left pixel, channels in R, G, B order (brief §9.4).
//! Blending one pixel applies a deterministic, gamma-free round-half-up
//! linear interpolation between the stored background channel and the text
//! channel for the given coverage.

use crate::renderer::error::RenderError;
use crate::renderer::settings::Rgb;

/// Pixel coordinate inside the frame; origin at the top-left corner.
#[derive(Clone, Copy)]
pub struct PixelPoint {
    pub x: u32,
    pub y: u32,
}

/// One coverage-blend instruction for a single pixel.
#[derive(Clone, Copy)]
pub struct PixelBlend {
    pub coverage: f32,
    pub color: Rgb,
}

/// A full RGB24 frame whose pixel buffer is always exactly
/// `width * height * 3` bytes (row-major, R, G, B per pixel); constructed
/// only via [`Frame::filled`].
#[derive(Debug)]
pub struct Frame {
    width: u32,
    height: u32,
    pixels: Vec<u8>,
}

impl Frame {
    /// Builds a frame of exactly `width * height * 3` bytes filled with
    /// `color`, rejecting non-positive dimensions and size overflow before
    /// any allocation happens.
    pub fn filled(width: u32, height: u32, color: Rgb) -> Result<Self, RenderError> {
        if has_non_positive_dimension(width, height) {
            return Err(RenderError::Geometry { width, height });
        }
        let byte_count =
            rgb24_byte_count(width, height).ok_or(RenderError::FrameSize { width, height })?;
        let mut pixels = vec![0u8; byte_count];
        for channel in pixels.chunks_mut(3) {
            channel[0] = color.red;
            channel[1] = color.green;
            channel[2] = color.blue;
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Blends one pixel in place; coordinates outside the frame are a
    /// no-op, so callers can never write beyond the buffer.
    pub(super) fn blend_at(&mut self, point: PixelPoint, blend: PixelBlend) {
        if !self.is_inside_frame(point) {
            return;
        }
        let base = (point.y as usize * self.width as usize + point.x as usize) * 3;
        self.pixels[base] = blend_channel(ChannelBlend {
            background: self.pixels[base],
            coverage: blend.coverage,
            text: blend.color.red,
        });
        self.pixels[base + 1] = blend_channel(ChannelBlend {
            background: self.pixels[base + 1],
            coverage: blend.coverage,
            text: blend.color.green,
        });
        self.pixels[base + 2] = blend_channel(ChannelBlend {
            background: self.pixels[base + 2],
            coverage: blend.coverage,
            text: blend.color.blue,
        });
    }

    fn is_inside_frame(&self, point: PixelPoint) -> bool {
        point.x < self.width && point.y < self.height
    }
}

pub(super) fn has_non_positive_dimension(width: u32, height: u32) -> bool {
    width == 0 || height == 0
}

pub(super) fn rgb24_byte_count(width: u32, height: u32) -> Option<usize> {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(3))
}

/// Inputs for blending one channel of one pixel.
struct ChannelBlend {
    background: u8,
    coverage: f32,
    text: u8,
}

/// Round-half-up of the linear interpolation between `background` and
/// `text`; with equal endpoints the result is exactly that endpoint.
fn blend_channel(blend: ChannelBlend) -> u8 {
    let delta = blend.text as f32 - blend.background as f32;
    let blended = blend.background as f32 + (blend.coverage * delta + 0.5).floor();
    blended.clamp(0.0, 255.0) as u8
}

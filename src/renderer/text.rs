//! `TextRenderer`: ab_glyph rasterization of the complete visible text into
//! fresh RGB24 frames (brief §9.3–9.6).
//!
//! Line-box placement (plan amendment A8): the baseline of line `i` is
//! `top_margin + scale_font.ascent() + i * line_height`, so ink never
//! enters the top margin, and the pen starts at `left_margin`. Clipping is
//! per pixel at the physical frame boundary: horizontally text is cut at
//! `x == width` with no wrapping and no font resizing (brief §9.5), and
//! `right_margin` is a start-offset config, not a clip. Vertically, ink
//! past the frame bottom is dropped; `bottom_margin` keeps a config-only
//! role (documented deviation: frame-boundary clipping, not margin
//! clipping, is the MVP contract). Unknown codepoints map to notdef
//! (`GlyphId(0)`) and are skipped while their advance is kept (documented
//! limitation).

use std::fs;

use ab_glyph::{point, Font, FontVec, Glyph, GlyphId, PxScale, ScaleFont};

use crate::renderer::contract::{RenderInput, Renderer};
use crate::renderer::error::RenderError;
use crate::renderer::frame::{Frame, PixelBlend, PixelPoint};
use crate::renderer::settings::RendererSettings;

/// One text line scheduled for rasterization.
struct LineJob<'a> {
    text: &'a str,
    index: usize,
}

/// Concrete MVP renderer: one validated settings set plus one font parsed
/// once at construction.
pub struct TextRenderer {
    settings: RendererSettings,
    font: FontVec,
}

impl TextRenderer {
    /// Loads and parses the configured font once; all validation happens
    /// at construction, so `render` only fails on frame-allocation
    /// overflow. Missing files surface as `FontOpen`, non-fonts as
    /// `FontParse` — no silent fallback to any other font.
    pub fn new(settings: RendererSettings) -> Result<Self, RenderError> {
        let bytes = fs::read(&settings.font_path).map_err(|source| RenderError::FontOpen {
            path: settings.font_path.clone(),
            source,
        })?;
        let font = FontVec::try_from_vec(bytes).map_err(|source| RenderError::FontParse {
            path: settings.font_path.clone(),
            source,
        })?;
        Ok(Self { settings, font })
    }

    fn rasterize_line(&self, frame: &mut Frame, job: &LineJob<'_>) {
        let scale = PxScale::from(self.settings.font_size as f32);
        let scale_font = self.font.as_scaled(scale);
        let baseline_y = self.settings.top_margin as f32
            + scale_font.ascent()
            + job.index as f32 * self.settings.line_height as f32;
        let mut pen_x = self.settings.left_margin as f32;
        for character in job.text.chars() {
            let glyph_id = scale_font.glyph_id(character);
            let glyph = glyph_id.with_scale_and_position(scale, point(pen_x, baseline_y));
            pen_x += scale_font.h_advance(glyph_id);
            if is_missing_glyph(glyph_id) {
                continue;
            }
            self.draw_glyph(frame, glyph);
        }
    }

    fn draw_glyph(&self, frame: &mut Frame, glyph: Glyph) {
        let Some(outlined) = self.font.outline_glyph(glyph) else {
            return;
        };
        let bounds = outlined.px_bounds();
        let origin_x = bounds.min.x.floor() as i32;
        let origin_y = bounds.min.y.floor() as i32;
        let text_color = self.settings.text_color;
        outlined.draw(|x, y, coverage| {
            if !has_influence(coverage) {
                return;
            }
            let absolute_x = origin_x + x as i32;
            let absolute_y = origin_y + y as i32;
            let pixel = PixelPoint {
                x: absolute_x as u32,
                y: absolute_y as u32,
            };
            frame.blend_at(
                pixel,
                PixelBlend {
                    coverage,
                    color: text_color,
                },
            );
        });
    }
}

impl Renderer for TextRenderer {
    /// Renders the complete input into a fresh background-filled frame:
    /// every `'\n'`-separated line is laid out top to bottom; empty input
    /// yields the untouched background frame.
    fn render(&mut self, input: &RenderInput) -> Result<Frame, RenderError> {
        let mut frame = Frame::filled(
            self.settings.width,
            self.settings.height,
            self.settings.background_color,
        )?;
        for (index, line) in input.text.split('\n').enumerate() {
            let job = LineJob { text: line, index };
            self.rasterize_line(&mut frame, &job);
        }
        Ok(frame)
    }
}

fn is_missing_glyph(glyph_id: GlyphId) -> bool {
    glyph_id.0 == 0
}

fn has_influence(coverage: f32) -> bool {
    coverage > 0.0
}

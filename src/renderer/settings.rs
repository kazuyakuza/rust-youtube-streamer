//! Renderer settings derived from typed configuration: frame geometry,
//! layout metrics, colors parsed from `#RRGGBB` hex, and the font path.
//! Settings are built directly from the config data structs; the crate's
//! config-validation module is never invoked here — geometry, vertical
//! layout, and color strings are re-checked independently (brief §5). No
//! file-existence check happens at all (Phase 01 config-validation
//! precedent); the renderer opens and parses the font at construction.

use std::fmt;
use std::path::PathBuf;

use crate::config::model::{RendererConfig, VideoConfig};
use crate::renderer::error::RenderError;
use crate::renderer::frame::has_non_positive_dimension;

const TEXT_COLOR_FIELD: &str = "text_color";
const BACKGROUND_COLOR_FIELD: &str = "background_color";

/// One 8-bit-per-channel color. Its `Display` form is the canonical
/// uppercase `#RRGGBB` string, and its `PartialEq` compares the three
/// channel bytes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    /// Parses a `#RRGGBB` hex color, case-insensitive. Construction paths
    /// use the field-aware internal parser, so errors reaching callers of
    /// [`RendererSettings::from_config`] always carry the exact config
    /// field name; this standalone entry attributes failures to
    /// `text_color`.
    // Test-only convenience entry: production parses through the
    // field-aware internal parser below.
    #[cfg(test)]
    pub fn parse_hex(value: &str) -> Result<Rgb, RenderError> {
        parse_hex_field(TEXT_COLOR_FIELD, value)
    }

    /// The all-black color (zero channels).
    // Test-only color: production backgrounds come from validated config.
    #[cfg(test)]
    pub fn black() -> Rgb {
        Rgb {
            red: 0,
            green: 0,
            blue: 0,
        }
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "#{:02X}{:02X}{:02X}",
            self.red, self.green, self.blue
        )
    }
}

/// Validated geometry, colors, and font path for one renderer instance.
#[derive(Debug, Clone, PartialEq)]
pub struct RendererSettings {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) font_size: u32,
    pub(super) line_height: u32,
    pub(super) left_margin: u32,
    pub(super) top_margin: u32,
    pub(super) bottom_margin: u32,
    pub(super) text_color: Rgb,
    pub(super) background_color: Rgb,
    pub(super) font_path: PathBuf,
}

impl RendererSettings {
    /// Derives and validates renderer settings from already-typed
    /// configuration: positive frame dimensions, a vertical layout that
    /// leaves room for at least one line, and parseable `#RRGGBB` colors.
    /// The font file itself is not touched here.
    pub fn from_config(config: &RendererConfig, video: &VideoConfig) -> Result<Self, RenderError> {
        if has_non_positive_dimension(video.width, video.height) {
            return Err(RenderError::Geometry {
                width: video.width,
                height: video.height,
            });
        }
        if lacks_visible_line(config, video) {
            return Err(RenderError::Layout {
                height: video.height,
                top_margin: config.top_margin,
                bottom_margin: config.bottom_margin,
                line_height: config.line_height,
            });
        }
        Ok(Self {
            width: video.width,
            height: video.height,
            font_size: config.font_size,
            line_height: config.line_height,
            left_margin: config.left_margin,
            top_margin: config.top_margin,
            bottom_margin: config.bottom_margin,
            text_color: parse_hex_field(TEXT_COLOR_FIELD, &config.text_color)?,
            background_color: parse_hex_field(BACKGROUND_COLOR_FIELD, &config.background_color)?,
            font_path: PathBuf::from(&config.font),
        })
    }
}

fn lacks_visible_line(config: &RendererConfig, video: &VideoConfig) -> bool {
    let occupied = u64::from(config.top_margin)
        + u64::from(config.bottom_margin)
        + u64::from(config.line_height);
    u64::from(config.line_height) == 0 || occupied > u64::from(video.height)
}

fn parse_hex_field(field: &'static str, value: &str) -> Result<Rgb, RenderError> {
    parse_six_hex(value).ok_or_else(|| RenderError::InvalidColor {
        field,
        value: value.to_string(),
    })
}

fn parse_six_hex(value: &str) -> Option<Rgb> {
    let digits = value.strip_prefix('#')?;
    if !has_six_ascii_hex_digits(digits) {
        return None;
    }
    Some(Rgb {
        red: u8::from_str_radix(&digits[0..2], 16).ok()?,
        green: u8::from_str_radix(&digits[2..4], 16).ok()?,
        blue: u8::from_str_radix(&digits[4..6], 16).ok()?,
    })
}

fn has_six_ascii_hex_digits(value: &str) -> bool {
    value.len() == 6 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

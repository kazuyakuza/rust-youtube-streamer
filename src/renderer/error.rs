//! Typed renderer errors. Each variant names the failed stage (geometry,
//! layout, color, or font loading) with the offending values, mirroring
//! `src/chat/error.rs`: hand-written `Display`, `std::error::Error` with
//! `source()`, no panics, no external derive crates.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// All failures produced by the renderer, one variant per stage.
#[derive(Debug)]
pub enum RenderError {
    /// Frame width or height is not positive (construction input).
    Geometry { width: u32, height: u32 },
    /// `width * height * 3` overflows or is otherwise unusable.
    FrameSize { width: u32, height: u32 },
    /// The vertical layout leaves room for less than one visible line.
    Layout {
        height: u32,
        top_margin: u32,
        bottom_margin: u32,
        line_height: u32,
    },
    /// A configured color is not `#RRGGBB`; `field` is `text_color` or
    /// `background_color`.
    InvalidColor { field: &'static str, value: String },
    /// The configured font file could not be opened for reading.
    FontOpen { path: PathBuf, source: io::Error },
    /// The opened file is not a parseable font.
    FontParse {
        path: PathBuf,
        source: ab_glyph::InvalidFont,
    },
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::Geometry { width, height } => write!(
                formatter,
                "renderer frame dimensions must be positive (width {width}, height {height})"
            ),
            RenderError::FrameSize { width, height } => {
                let expected = expected_buffer_bytes(*width, *height);
                write!(
                    formatter,
                    "frame size of width {width} and height {height} does not fit a usable RGB24 buffer of {expected} bytes"
                )
            }
            RenderError::Layout { height, top_margin, bottom_margin, line_height } => write!(
                formatter,
                "renderer vertical layout must leave room for at least one line (height {height}, top_margin {top_margin}, bottom_margin {bottom_margin}, line_height {line_height})"
            ),
            RenderError::InvalidColor { field, value } => write!(
                formatter,
                "renderer {field} '{value}' is not a valid #RRGGBB hex color (example #FFFFFF; '#' prefix plus six hex digits required)"
            ),
            RenderError::FontOpen { path, source } => write!(
                formatter,
                "unable to open font file '{}': {source}",
                path.display()
            ),
            RenderError::FontParse { path, source } => write!(
                formatter,
                "unable to parse font file '{}': {source}",
                path.display()
            ),
        }
    }
}

impl Error for RenderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RenderError::Geometry { .. } => None,
            RenderError::FrameSize { .. } => None,
            RenderError::Layout { .. } => None,
            RenderError::InvalidColor { .. } => None,
            RenderError::FontOpen { source, .. } => Some(source),
            RenderError::FontParse { source, .. } => Some(source),
        }
    }
}

fn expected_buffer_bytes(width: u32, height: u32) -> usize {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(3))
        .unwrap_or(usize::MAX)
}

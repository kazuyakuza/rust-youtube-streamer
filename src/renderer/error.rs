//! Typed renderer errors. Each variant names the failed stage (geometry,
//! layout, color, or font loading) with the offending values, mirroring
//! `src/chat/error.rs`: hand-written `Display`, `std::error::Error` with
//! `source()`, no panics, no external derive crates. Contracts follow
//! brief §9.2–§9.6; scope authority
//! `.agent/todos/20261010/20261010-todo-2.md` Task 2.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::renderer::frame::rgb24_byte_count;

/// All failures produced by the renderer, one variant per stage.
#[derive(Debug)]
pub enum RenderError {
    /// Frame width or height is not positive; raised when settings are
    /// derived or a frame is sized, carrying the offending dimensions.
    Geometry { width: u32, height: u32 },
    /// `width * height * 3` overflows or is otherwise unusable; raised at
    /// frame sizing (settings derivation and every render), echoing the
    /// dimensions that cannot form a buffer.
    FrameSize { width: u32, height: u32 },
    /// The vertical layout leaves room for less than one visible line;
    /// raised by settings derivation, echoing the height, both vertical
    /// margins, and the line height.
    Layout {
        height: u32,
        top_margin: u32,
        bottom_margin: u32,
        line_height: u32,
    },
    /// A configured color is not `#RRGGBB`; raised by settings derivation.
    /// `field` is `text_color` or `background_color`.
    InvalidColor { field: &'static str, value: String },
    /// `TextRenderer::new` could not open the configured font file for
    /// reading. Carries the font path and the underlying I/O cause.
    FontOpen { path: PathBuf, source: io::Error },
    /// `TextRenderer::new` read the configured file but it is not a
    /// parseable font. Carries the font path and the underlying parse
    /// cause.
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
                let expected = rgb24_byte_count(*width, *height).unwrap_or(usize::MAX);
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

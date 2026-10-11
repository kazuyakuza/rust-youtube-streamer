//! Renderer contract (brief §9.2): the input carries the complete visible
//! multiline text, and the trait produces one raw RGB24 frame or a typed
//! error. The renderer owns no message history, scrolling, visible-message
//! capacity, YouTube access, or FFmpeg behavior. Implementations validate
//! all geometry, colors, and font data at construction (see
//! `TextRenderer::new`), so `render` reports failures only for genuine
//! render-time problems.

use crate::renderer::error::RenderError;
use crate::renderer::frame::Frame;

/// Complete visible multiline text (`'\n'`-separated lines) for one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderInput {
    pub text: String,
}

/// Contract for turning render input into a raw video frame.
pub trait Renderer {
    /// Produces one complete frame sized by the implementation's validated
    /// settings, or a typed error. All configuration is checked at
    /// construction, so an implementation's `render` fails only when frame
    /// allocation itself cannot be satisfied.
    fn render(&mut self, input: &RenderInput) -> Result<Frame, RenderError>;
}

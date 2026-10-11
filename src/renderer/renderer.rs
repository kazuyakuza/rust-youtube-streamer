//! Renderer contract (brief §9.2): the input carries the complete visible
//! multiline text, and the trait produces one raw RGB24 frame or a typed
//! error. The renderer owns no message history, scrolling, chat, or
//! FFmpeg behavior.

use crate::renderer::error::RenderError;
use crate::renderer::frame::Frame;

/// Complete visible multiline text (`'\n'`-separated lines) for one frame.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderInput {
    pub text: String,
}

/// Contract for turning render input into a raw video frame.
pub trait Renderer {
    fn render(&mut self, input: &RenderInput) -> Result<Frame, RenderError>;
}

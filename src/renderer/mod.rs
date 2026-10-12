//! RGB24 text renderer: the `Renderer` trait contract, construction-
//! validated settings, and the ab_glyph-based `TextRenderer` MVP (brief
//! §9.1–9.4). Scope authority: TODO Task 2 in
//! `.agent/todos/20261010/20261010-todo-2.md`. The module is a pure local
//! component: no YouTube API types, chat state, network, or FFmpeg
//! dependency. Cross-module consumers go through the `pub(crate) use`
//! re-exports below. Staging allow: retires in TODO Task 3 when the demo
//! mode (the pipeline's CLI consumer) makes this module reachable from
//! `main`; clippy `-D warnings` guards everything else. File/module mapping
//! note: the trait and `RenderInput` live in `renderer.rs` but are declared
//! as `contract` via `#[path]` (plan amendment A9 — avoids
//! `clippy::module_inception`); in-tree consumers import
//! `contract::Renderer` / `contract::RenderInput`.
#![allow(dead_code)]

#[path = "renderer.rs"]
mod contract;
mod error;
mod frame;
mod settings;
mod text;

pub(crate) use contract::{RenderInput, Renderer};
pub(crate) use error::RenderError;
pub(crate) use frame::Frame;
pub(crate) use settings::RendererSettings;
// Test-only construction access: `StubRenderer` fills frames in tests.
#[cfg(test)]
pub(crate) use settings::Rgb;
pub(crate) use text::TextRenderer;

#[cfg(test)]
mod frame_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod text_test_helpers;
#[cfg(test)]
mod text_tests;

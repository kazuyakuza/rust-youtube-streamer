//! RGB24 text renderer: the `Renderer` trait contract, construction-
//! validated settings, and the ab_glyph-based `TextRenderer` MVP (brief
//! §9.1–9.4). Scope authority: TODO Task 2 in
//! `.agent/todos/20261010/20261010-todo-2.md`. The module is a pure local
//! component: no YouTube API types, chat state, network, or FFmpeg
//! dependency. Cross-module re-exports return at the Phase 03 wiring step
//! (precedent: `src/chat/mod.rs`). File/module mapping note: the trait and
//! `RenderInput` live in `renderer.rs` but are declared as `contract` via
//! `#[path]` (plan amendment A9 — avoids `clippy::module_inception`);
//! in-tree consumers import `contract::Renderer` / `contract::RenderInput`.

#[path = "renderer.rs"]
mod contract;
mod error;
mod frame;
mod settings;
mod text;

#[cfg(test)]
mod frame_tests;
#[cfg(test)]
mod settings_tests;
#[cfg(test)]
mod text_tests;

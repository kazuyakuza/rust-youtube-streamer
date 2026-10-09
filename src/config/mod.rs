//! Typed application configuration: schema, JSON loading, and validation.
//!
//! Public API: [`load_from_path`] reads one explicit configuration file and
//! returns either a validated [`AppConfig`] or a [`ConfigError`] naming the
//! failed stage (I/O, deserialization, or validation with per-field issues).
//! A returned configuration has always passed every validation rule, and
//! expected file, JSON, or rule failures surface as typed errors instead of
//! panics.
//!
//! The on-disk schema is mirrored by the committed `config/config.example.json`
//! template; the real `config/config.json` is gitignored and never committed.
//! Property meanings and validation requirements follow section 5 of the
//! project brief
//! ([`.agent/project-info/brief.md`](../../.agent/project-info/brief.md)).

mod error;
mod loader;
mod model;
mod validation;

#[cfg(test)]
mod test_fixtures;

pub use error::{ConfigError, ConfigIssue};
pub use loader::load_from_path;
pub use model::{
    AppConfig, BroadcastConfig, ChatConfig, FfmpegConfig, RendererConfig, VideoConfig,
    YouTubeConfig,
};

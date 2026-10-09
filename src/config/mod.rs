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

// Test-only crate-wide alias so in-crate tests outside `config` (for example
// the application mode tests) build sample configurations through the same
// fixture instead of duplicating it. The `config` module's normal public
// surface is unchanged.
#[cfg(test)]
pub(crate) use test_fixtures::valid_config;

pub use error::ConfigError;
pub use loader::load_from_path;
pub use model::AppConfig;

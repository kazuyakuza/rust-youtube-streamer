//! Configuration loading pipeline: read one explicit file, deserialize it,
//! then validate it before anything is returned.

use std::fs;
use std::path::Path;

use crate::config::error::ConfigError;
use crate::config::model::AppConfig;
use crate::config::validation;

/// Loads and validates the JSON configuration stored at `path`.
///
/// The path is used exactly as provided — resolved by the caller, with no
/// current-working-directory assumptions inside this module. A successful
/// return means the configuration passed every validation rule. Failures
/// stop the call with a typed [`ConfigError`]: `Io` when the file cannot be
/// read, `Deserialization` when the JSON is malformed or violates the
/// schema, and `Validation` when semantic rules report all collected issues
/// together. Expected file, JSON, and validation errors never panic.
pub fn load_from_path(path: &Path) -> Result<AppConfig, ConfigError> {
    let raw = read_file(path)?;
    let config = parse(path, &raw)?;
    validation::validate(&config)?;
    Ok(config)
}

fn read_file(path: &Path) -> Result<String, ConfigError> {
    fs::read_to_string(path).map_err(|source| ConfigError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn parse(path: &Path, raw: &str) -> Result<AppConfig, ConfigError> {
    serde_json::from_str(raw).map_err(|error| ConfigError::Deserialization {
        path: path.to_path_buf(),
        message: error.to_string(),
    })
}

#[cfg(test)]
#[path = "loader_tests.rs"]
mod loader_tests;

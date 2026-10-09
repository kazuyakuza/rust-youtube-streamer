use std::fs;
use std::path::Path;

use crate::config::error::ConfigError;
use crate::config::model::AppConfig;
use crate::config::validation;

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

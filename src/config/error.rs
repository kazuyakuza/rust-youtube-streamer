//! Typed configuration errors. Each variant names the failed loading stage and
//! the offending file or fields, so callers can report actionable messages
//! without matching on error strings.

use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// One field-level validation problem: the dotted JSON field path and a
/// human-readable description of why the value is invalid.
#[derive(Debug)]
pub struct ConfigIssue {
    field: String,
    problem: String,
}

impl ConfigIssue {
    pub(super) fn new(field: String, problem: String) -> Self {
        Self { field, problem }
    }
}

impl fmt::Display for ConfigIssue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.problem)
    }
}

/// All failures produced while loading a configuration file, one variant per
/// stage of the loading pipeline.
#[derive(Debug)]
pub enum ConfigError {
    /// The file could not be read (missing, unreadable, or a directory).
    /// Produced by the loader's read step with the offending path and the
    /// underlying I/O cause.
    Io { path: PathBuf, source: io::Error },
    /// The contents are not valid JSON or do not match the required schema
    /// (missing or unknown fields, wrong value types). Produced by the
    /// loader's deserialization step; `message` carries the `serde_json`
    /// explanation.
    Deserialization { path: PathBuf, message: String },
    /// The document parsed successfully but violates semantic rules. Produced
    /// by the validation module and always carries every issue found in one
    /// validation pass rather than only the first.
    Validation { issues: Vec<ConfigIssue> },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io { path, source } => write!(
                formatter,
                "unable to read configuration file '{}': {}",
                path.display(),
                source
            ),
            ConfigError::Deserialization { path, message } => write!(
                formatter,
                "failed to parse configuration file '{}': {}",
                path.display(),
                message
            ),
            ConfigError::Validation { issues } => write_validation(formatter, issues),
        }
    }
}

fn write_validation(formatter: &mut fmt::Formatter<'_>, issues: &[ConfigIssue]) -> fmt::Result {
    let issues_text = issues
        .iter()
        .map(|issue| issue.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    write!(
        formatter,
        "invalid configuration ({} issue(s)): {}",
        issues.len(),
        issues_text
    )
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io { source, .. } => Some(source),
            ConfigError::Deserialization { .. } => None,
            ConfigError::Validation { .. } => None,
        }
    }
}

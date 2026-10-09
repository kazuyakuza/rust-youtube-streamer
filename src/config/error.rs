use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

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

#[derive(Debug)]
pub enum ConfigError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Deserialization {
        path: PathBuf,
        message: String,
    },
    Validation {
        issues: Vec<ConfigIssue>,
    },
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

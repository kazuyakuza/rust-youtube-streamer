//! Typed startup failures for the application layer. Every fatal problem the
//! startup pipeline can hit funnels through [`StartupError`], so the process
//! entry point writes exactly one actionable stderr line and returns a
//! stable, distinct exit code per failure class.

use std::fmt;

use crate::config::ConfigError;

/// Exit code for configuration and logging startup failures.
pub(super) const STARTUP_FAILURE_EXIT: u8 = 1;

/// Exit code for command-line usage errors.
pub(super) const USAGE_ERROR_EXIT: u8 = 2;

/// All fatal failures produced while starting the application.
#[derive(Debug)]
pub(super) enum StartupError {
    /// The command line could not be parsed; holds the actionable message.
    Usage(String),
    /// The configuration file could not be read, parsed, or validated.
    Configuration(ConfigError),
    /// Structured logging could not be initialized.
    LogInit(LogInitError),
}

impl StartupError {
    /// Usage failure for a token that cannot appear where it was found.
    pub(super) fn unexpected_argument(token: &str) -> Self {
        StartupError::Usage(format!("unexpected argument '{token}'"))
    }

    /// Maps each failure class to its stable process exit code: usage errors
    /// exit 2, configuration and logging startup failures exit 1.
    pub(super) fn exit_code(&self) -> u8 {
        match self {
            StartupError::Usage(_) => USAGE_ERROR_EXIT,
            StartupError::Configuration(_) | StartupError::LogInit(_) => STARTUP_FAILURE_EXIT,
        }
    }
}

impl fmt::Display for StartupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StartupError::Usage(message) => formatter.write_str(message),
            StartupError::Configuration(error) => write!(formatter, "{error}"),
            StartupError::LogInit(error) => {
                write!(formatter, "failed to initialize logging: {error}")
            }
        }
    }
}

/// Failure raised while initializing structured logging. The Task 2 seam is
/// infallible, so this type has no constructors yet; Task 3 shapes it when
/// tracing initialization can genuinely fail.
#[derive(Debug)]
pub(super) enum LogInitError {}

impl fmt::Display for LogInitError {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {}
    }
}

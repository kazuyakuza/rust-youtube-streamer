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

/// All fatal failures produced while starting the application. Each variant
/// is one user-facing failure class with a fixed exit code; every value
/// funnels through the setup pipeline to `main` and is reported exactly once
/// there, never at the layer that produced it.
#[derive(Debug)]
pub(super) enum StartupError {
    /// Bad command line (missing, unknown, or repeated token). Holds the
    /// actionable message; exits with code 2.
    Usage(String),
    /// The configuration file could not be read, parsed, or validated.
    /// Startup failure; exits with code 1.
    Configuration(ConfigError),
    /// Structured logging could not be initialized. Startup failure;
    /// exits with code 1.
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

/// Fatal failures raised while initializing structured logging. The filter
/// itself never fails startup: unset, empty, whitespace-only, or invalid
/// `RUST_LOG` values all resolve to the default filter, with one stderr
/// warning for an invalid value. Every variant aborts startup with exit code
/// 1 through [`StartupError::LogInit`].
#[derive(Debug)]
pub(super) enum LogInitError {
    /// The global default subscriber could not be installed. Unreachable while
    /// the startup funnel calls `init_logging` exactly once, and reported here
    /// so a future second call site can never silently drop logs.
    SubscriberInstall { reason: String },
}

/// Formats the variant as the logging-failure message that
/// [`StartupError::LogInit`] prepends its `failed to initialize logging:`
/// wrapper to, forming the single stderr line reported for a fatal logging
/// failure.
impl fmt::Display for LogInitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogInitError::SubscriberInstall { reason } => {
                write!(formatter, "failed to install logging subscriber: {reason}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn subscriber_install_startup_error() -> StartupError {
        StartupError::LogInit(LogInitError::SubscriberInstall {
            reason: "already set".to_string(),
        })
    }

    #[test]
    fn log_init_subscriber_install_maps_to_startup_failure_exit() {
        let startup_error = subscriber_install_startup_error();
        assert_eq!(startup_error.exit_code(), STARTUP_FAILURE_EXIT);
    }

    #[test]
    fn subscriber_install_display_wraps_message_and_keeps_reason() {
        let startup_error = subscriber_install_startup_error();
        let message = format!("{startup_error}");
        assert!(message.starts_with("failed to initialize logging: "));
        assert!(message.contains("failed to install logging subscriber"));
        assert!(message.contains("already set"));
    }
}

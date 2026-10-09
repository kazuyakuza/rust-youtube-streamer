//! Placeholder handlers for the two execution modes. Configuration is always
//! already loaded and validated when either handler runs; neither handler
//! contacts YouTube, requests credentials, or starts FFmpeg. Handlers return
//! the raw exit code so unit tests can assert it; `app::run` performs the
//! single `ExitCode` conversion at the process boundary.

use crate::config::AppConfig;

/// Exit code reported while a mode exists only as a documented placeholder.
pub(super) const NOT_IMPLEMENTED_EXIT: u8 = 3;

/// `auth` placeholder until a later phase: the configuration is already
/// loaded and validated when this runs (the parameter enforces that
/// ordering), the handler reports that authentication is not implemented yet
/// on standard error, and returns `NOT_IMPLEMENTED_EXIT` (3). It never
/// contacts YouTube and never requests or stores credentials.
pub(super) fn auth(_config: &AppConfig) -> u8 {
    eprintln!(
        "error: authentication is not implemented yet: the OAuth authorization \
         flow has not been built; no credentials were requested or stored"
    );
    NOT_IMPLEMENTED_EXIT
}

/// `run` placeholder until a later phase: the configuration is already
/// loaded and validated when this runs (the parameter enforces that
/// ordering), the handler reports that the runtime pipeline is not
/// implemented yet on standard error, and returns `NOT_IMPLEMENTED_EXIT`
/// (3). It never creates YouTube resources and never starts FFmpeg.
pub(super) fn run(_config: &AppConfig) -> u8 {
    eprintln!(
        "error: runtime pipeline is not implemented yet: the YouTube/FFmpeg \
         pipeline has not been built; no YouTube resources were created and \
         FFmpeg was not started"
    );
    NOT_IMPLEMENTED_EXIT
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::valid_config;

    #[test]
    fn auth_reports_not_implemented_exit_code() {
        let config = valid_config();
        assert_eq!(auth(&config), NOT_IMPLEMENTED_EXIT);
    }

    #[test]
    fn run_reports_not_implemented_exit_code() {
        let config = valid_config();
        assert_eq!(run(&config), NOT_IMPLEMENTED_EXIT);
    }

    #[test]
    fn exit_codes_remain_distinct_and_stable() {
        assert_eq!(crate::app::error::STARTUP_FAILURE_EXIT, 1);
        assert_eq!(crate::app::error::USAGE_ERROR_EXIT, 2);
        assert_eq!(NOT_IMPLEMENTED_EXIT, 3);
    }
}

//! Placeholder handlers for the `auth` and `run` execution modes. Configuration is always
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

/// The single stderr line the `run` placeholder reports: the four
/// not-yet-built YouTube prerequisites, the non-interactive boundary,
/// the later-phase intent, and the explicit no-local-fallback pointer to
/// the `demo` command.
const RUN_NOT_IMPLEMENTED_MESSAGE: &str = "runtime pipeline is not implemented yet: \
YouTube streaming needs OAuth authorization, broadcast lifecycle, RTMPS ingestion \
URL acquisition and YouTube live chat integration, none of which is built; the run \
mode never reads input, never runs the local demo pipeline, and will start the \
YouTube/FFmpeg pipeline only in a later phase. There is no local fallback: for a \
local demonstration recording use the demo command";

/// `run` placeholder until a later phase: the configuration is already
/// loaded and validated when this runs (the parameter enforces that
/// ordering), the handler reports the not-yet-built YouTube
/// prerequisites (OAuth, broadcast lifecycle, RTMPS ingestion URL
/// acquisition, YouTube live chat) on standard error, states that the
/// YouTube/FFmpeg pipeline starts in a later phase with no local
/// fallback, and returns `NOT_IMPLEMENTED_EXIT` (3). It never reads
/// input, never runs the local demo pipeline, never creates YouTube
/// resources, and never starts FFmpeg.
pub(super) fn run(_config: &AppConfig) -> u8 {
    eprintln!("error: {RUN_NOT_IMPLEMENTED_MESSAGE}");
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

    #[test]
    fn run_placeholder_message_names_the_four_missing_prerequisites() {
        assert_eq!(
            RUN_NOT_IMPLEMENTED_MESSAGE,
            "runtime pipeline is not implemented yet: YouTube streaming needs \
             OAuth authorization, broadcast lifecycle, RTMPS ingestion URL \
             acquisition and YouTube live chat integration, none of which is \
             built; the run mode never reads input, never runs the local demo \
             pipeline, and will start the YouTube/FFmpeg pipeline only in a \
             later phase. There is no local fallback: for a local \
             demonstration recording use the demo command"
        );
    }

    #[test]
    fn run_placeholder_message_denies_interactive_input_and_points_to_demo() {
        assert!(RUN_NOT_IMPLEMENTED_MESSAGE.contains("never reads input"));
        assert!(RUN_NOT_IMPLEMENTED_MESSAGE.contains("never runs the local demo pipeline"));
        assert!(RUN_NOT_IMPLEMENTED_MESSAGE.contains("no local fallback"));
        assert!(RUN_NOT_IMPLEMENTED_MESSAGE.contains("use the demo command"));
        assert!(!RUN_NOT_IMPLEMENTED_MESSAGE.contains("stdin"));
        assert!(!RUN_NOT_IMPLEMENTED_MESSAGE.contains("keyboard"));
        assert!(!RUN_NOT_IMPLEMENTED_MESSAGE.contains("press"));
    }
}

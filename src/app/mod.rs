//! Application orchestration: parse the command line, initialize structured
//! logging, load and validate configuration, then dispatch the selected mode.
//! Every fatal failure funnels through typed [`StartupError`] values so the
//! process writes each error exactly once and exits with a stable code. The
//! pipeline works in raw `u8` exit codes so tests can assert them; [`run`]
//! performs the single `ExitCode` conversion at the process boundary. The
//! exit codes are a stable contract: 1 = configuration or logging-init
//! startup failure, 2 = command-line usage error, 3 = selected mode not
//! implemented yet (both modes stay placeholders until later phases).

mod cli;
mod error;
mod logging;
mod modes;

use std::ffi::OsString;
use std::path::Path;
use std::process::ExitCode;

use crate::config::{self, AppConfig};
use cli::ParseOutcome;
use error::StartupError;
use tracing::info;

/// Raw command-line tokens that follow the program name. They are kept as
/// [`OsString`]s so platform arguments with unusual encoding can never panic
/// the parser; non-UTF-8 tokens become usage errors.
pub(crate) type ArgsTokens = [OsString];

/// Exit code for a successful `--help` request.
const SUCCESS_EXIT: u8 = 0;

/// Runs the startup pipeline and returns the process exit code.
pub(crate) fn run(args: &ArgsTokens) -> ExitCode {
    ExitCode::from(evaluate(args))
}

/// Executes every startup stage in order and returns the raw exit code.
fn evaluate(args: &ArgsTokens) -> u8 {
    let tokens = match string_tokens(args) {
        Ok(tokens) => tokens,
        Err(startup_error) => return report_failure(startup_error),
    };
    match cli::parse(&tokens) {
        Ok(ParseOutcome::Help) => print_help(),
        Ok(ParseOutcome::Invocation(invocation)) => start_mode(invocation),
        Err(startup_error) => report_failure(startup_error),
    }
}

fn string_tokens(args: &ArgsTokens) -> Result<Vec<String>, StartupError> {
    args.iter().map(string_token).collect()
}

fn string_token(token: &OsString) -> Result<String, StartupError> {
    token
        .to_str()
        .map(str::to_string)
        .ok_or_else(|| StartupError::unexpected_argument(&token.to_string_lossy()))
}

fn print_help() -> u8 {
    println!("{}", cli::USAGE);
    SUCCESS_EXIT
}

/// Runs the mode for a parsed invocation: initialize logging, emit the
/// mode-selected event, load and validate configuration, then dispatch and
/// emit the completion event. These three `info` events are the only tracing
/// output on the success path, and every field they carry comes from the
/// whitelist (`command`, `config_path`, `exit_code`) — never configuration
/// values.
fn start_mode(invocation: cli::Invocation) -> u8 {
    if let Err(startup_error) = initialize_logging() {
        return report_failure(startup_error);
    }
    info!(
        command = command_name(&invocation.mode),
        config_path = %invocation.config_path,
        "mode selected"
    );
    match load_configuration(&invocation.config_path) {
        Ok(app_config) => {
            info!("configuration loaded and validated");
            let code = dispatch(&invocation.mode, &app_config);
            info!(
                command = command_name(&invocation.mode),
                exit_code = code,
                "mode finished"
            );
            code
        }
        Err(startup_error) => report_failure(startup_error),
    }
}

fn command_name(mode: &cli::Mode) -> &'static str {
    match mode {
        cli::Mode::Auth => "auth",
        cli::Mode::Run => "run",
    }
}

fn initialize_logging() -> Result<(), StartupError> {
    logging::init_logging().map_err(StartupError::LogInit)
}

fn load_configuration(config_path: &str) -> Result<AppConfig, StartupError> {
    config::load_from_path(Path::new(config_path)).map_err(StartupError::Configuration)
}

fn dispatch(mode: &cli::Mode, app_config: &AppConfig) -> u8 {
    match mode {
        cli::Mode::Auth => modes::auth(app_config),
        cli::Mode::Run => modes::run(app_config),
    }
}

/// Reports a fatal startup failure as exactly one `error:` line on standard
/// error and returns its mapped exit code. The failure is never re-emitted
/// through tracing, so it is not reported at two layers.
fn report_failure(startup_error: StartupError) -> u8 {
    eprintln!("error: {startup_error}");
    startup_error.exit_code()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigError;

    #[test]
    fn usage_errors_map_to_exit_code_two() {
        let startup_error = StartupError::Usage("message".to_string());
        assert_eq!(startup_error.exit_code(), error::USAGE_ERROR_EXIT);
    }

    #[test]
    fn configuration_errors_map_to_exit_code_one() {
        let startup_error =
            StartupError::Configuration(ConfigError::Validation { issues: Vec::new() });
        assert_eq!(startup_error.exit_code(), error::STARTUP_FAILURE_EXIT);
    }

    #[test]
    fn evaluate_maps_empty_arguments_to_usage_exit() {
        assert_eq!(evaluate(&[]), error::USAGE_ERROR_EXIT);
    }

    #[test]
    fn evaluate_maps_unknown_command_to_usage_exit() {
        let args = vec![OsString::from("status")];
        assert_eq!(evaluate(&args), error::USAGE_ERROR_EXIT);
    }

    #[test]
    fn evaluate_maps_help_to_success_exit() {
        let args = vec![OsString::from("--help")];
        assert_eq!(evaluate(&args), SUCCESS_EXIT);
    }

    #[cfg(unix)]
    #[test]
    fn evaluate_maps_non_utf8_argument_to_usage_exit() {
        use std::os::unix::ffi::OsStrExt;
        let args = vec![OsString::from(std::ffi::OsStr::from_bytes(b"-\xff"))];
        assert_eq!(evaluate(&args), error::USAGE_ERROR_EXIT);
    }
}

//! Command-line parsing for the service executable. The accepted surface is
//! exactly two commands (`auth`, `run`), one option (`--config <path>`) that
//! may appear before or after the command and at most once, and `--help`,
//! which is accepted only before the command and prints the usage text to
//! standard output before exiting successfully. When `--config` is absent,
//! the default path [`DEFAULT_CONFIG_PATH`] is used. The surface is fixed
//! and minimal, so a small deterministic standard-library matcher covers it
//! cleanly; per the project dependency policy, a crate is not added when the
//! standard library suffices, which rules out a CLI framework here.

use super::error::StartupError;

/// Configuration file used whenever `--config` is absent; an explicit
/// `--config <path>` overrides it. The example template
/// `config/config.example.json` is never referenced, defaulted to, or
/// implicitly loaded as a live configuration.
pub(crate) const DEFAULT_CONFIG_PATH: &str = "config/config.json";

/// Usage text printed on standard output for `--help`.
pub(super) const USAGE: &str = "\
usage: rust-youtube-streamer-service [--config <path>] <command>

commands:
  auth                 run the OAuth authorization flow (not implemented yet)
  run                  start the streaming runtime (not implemented yet)

options:
  --config <path>      configuration file to load (default: config/config.json)
  --help               print this help text";

const CONFIG_FLAG: &str = "--config";
const HELP_FLAG: &str = "--help";
const AUTH_COMMAND: &str = "auth";
const RUN_COMMAND: &str = "run";

/// The execution mode selected on the command line.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Auth,
    Run,
}

/// One successfully parsed command line.
#[derive(Debug, PartialEq, Eq)]
pub(super) struct Invocation {
    pub(super) mode: Mode,
    pub(super) config_path: String,
}

/// Outcome of parsing: a resolved invocation, or a `--help` request that
/// exits successfully without starting the application pipeline.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum ParseOutcome {
    Help,
    Invocation(Invocation),
}

type TokenCursor<'a> = std::slice::Iter<'a, String>;

/// Parses raw string arguments into an invocation, a help request, or a
/// typed usage error. Pure: no environment, file-system, or output access.
pub(super) fn parse(args: &[String]) -> Result<ParseOutcome, StartupError> {
    let mut config_path: Option<String> = None;
    let mut mode: Option<Mode> = None;
    let mut tokens = args.iter();
    while let Some(token) = tokens.next() {
        if is_help_token(token) {
            return help_result(&mode);
        }
        if is_config_flag(token) {
            config_path = Some(config_value(&mut tokens, config_path.is_some())?);
            continue;
        }
        if is_option_token(token) {
            return Err(unknown_option_error(token));
        }
        mode = Some(resolved_mode(token, mode.is_some())?);
    }
    finished_invocation(mode, config_path)
}

fn help_result(mode: &Option<Mode>) -> Result<ParseOutcome, StartupError> {
    match mode {
        None => Ok(ParseOutcome::Help),
        Some(_) => Err(StartupError::unexpected_argument(HELP_FLAG)),
    }
}

fn is_help_token(token: &str) -> bool {
    token == HELP_FLAG
}

fn is_config_flag(token: &str) -> bool {
    token == CONFIG_FLAG
}

fn is_option_token(token: &str) -> bool {
    token.starts_with("--")
}

fn config_value(tokens: &mut TokenCursor<'_>, already_given: bool) -> Result<String, StartupError> {
    if already_given {
        return Err(StartupError::Usage(
            "'--config' given more than once".to_string(),
        ));
    }
    match tokens.next() {
        Some(value) if !is_option_token(value) => Ok(value.clone()),
        _ => Err(StartupError::Usage(
            "missing value for '--config'".to_string(),
        )),
    }
}

fn resolved_mode(token: &str, already_given: bool) -> Result<Mode, StartupError> {
    if already_given {
        return Err(StartupError::unexpected_argument(token));
    }
    match token {
        AUTH_COMMAND => Ok(Mode::Auth),
        RUN_COMMAND => Ok(Mode::Run),
        _ => Err(StartupError::Usage(format!(
            "unknown command '{token}' (expected 'auth' or 'run')"
        ))),
    }
}

fn unknown_option_error(token: &str) -> StartupError {
    StartupError::Usage(format!("unknown option '{token}'"))
}

fn finished_invocation(
    mode: Option<Mode>,
    config_path: Option<String>,
) -> Result<ParseOutcome, StartupError> {
    match mode {
        Some(mode) => Ok(ParseOutcome::Invocation(Invocation {
            mode,
            config_path: config_path.unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string()),
        })),
        None => Err(StartupError::Usage(
            "expected a command ('auth' or 'run')".to_string(),
        )),
    }
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod cli_tests;

//! Command-line parsing for the service executable. The accepted surface is
//! exactly two commands (`auth`, `run`), one option (`--config <path>`), and
//! `--help`, so a small deterministic standard-library matcher covers it
//! without adding a CLI framework dependency.

use super::error::StartupError;

/// Configuration file used whenever `--config` is absent. The committed
/// example template is never referenced, defaulted, or implicitly loaded.
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
        Some(_) => Err(StartupError::Usage(format!(
            "unexpected argument '{HELP_FLAG}'"
        ))),
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
        return Err(StartupError::Usage(format!(
            "unexpected argument '{token}'"
        )));
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
mod tests {
    use super::*;

    fn args(tokens: &[&str]) -> Vec<String> {
        tokens.iter().map(|token| token.to_string()).collect()
    }

    fn parsed_invocation(tokens: &[&str]) -> Invocation {
        match parse(&args(tokens)) {
            Ok(ParseOutcome::Invocation(invocation)) => invocation,
            other => panic!("expected invocation, got {other:?}"),
        }
    }

    fn usage_error_message(tokens: &[&str]) -> String {
        match parse(&args(tokens)) {
            Err(StartupError::Usage(message)) => message,
            other => panic!("expected usage error, got {other:?}"),
        }
    }

    #[test]
    fn default_config_path_constant_is_stable() {
        assert_eq!(DEFAULT_CONFIG_PATH, "config/config.json");
    }

    #[test]
    fn parses_auth_and_run_alone_with_default_path() {
        let auth = parsed_invocation(&["auth"]);
        assert_eq!(auth.mode, Mode::Auth);
        assert_eq!(auth.config_path, DEFAULT_CONFIG_PATH);
        let run = parsed_invocation(&["run"]);
        assert_eq!(run.mode, Mode::Run);
        assert_eq!(run.config_path, DEFAULT_CONFIG_PATH);
    }

    #[test]
    fn accepts_config_option_before_and_after_command() {
        let auth_first = parsed_invocation(&["--config", "a.json", "auth"]);
        assert_eq!(auth_first.config_path, "a.json");
        let auth_last = parsed_invocation(&["auth", "--config", "b.json"]);
        assert_eq!(auth_last.config_path, "b.json");
        let run_first = parsed_invocation(&["--config", "c.json", "run"]);
        assert_eq!(run_first.config_path, "c.json");
        let run_last = parsed_invocation(&["run", "--config", "d.json"]);
        assert_eq!(run_last.config_path, "d.json");
    }

    #[test]
    fn accepts_help_flag_forms() {
        assert_eq!(parse(&args(&["--help"])).unwrap(), ParseOutcome::Help);
        let config_then_help = args(&["--config", "x.json", "--help"]);
        assert_eq!(parse(&config_then_help).unwrap(), ParseOutcome::Help);
    }

    #[test]
    fn rejects_empty_arguments() {
        assert_eq!(
            usage_error_message(&[]),
            "expected a command ('auth' or 'run')"
        );
    }

    #[test]
    fn rejects_config_option_without_command() {
        assert_eq!(
            usage_error_message(&["--config", "x.json"]),
            "expected a command ('auth' or 'run')"
        );
    }

    #[test]
    fn rejects_unknown_command() {
        assert_eq!(
            usage_error_message(&["status"]),
            "unknown command 'status' (expected 'auth' or 'run')"
        );
    }

    #[test]
    fn rejects_unknown_option() {
        assert_eq!(
            usage_error_message(&["--verbose", "auth"]),
            "unknown option '--verbose'"
        );
    }

    #[test]
    fn rejects_missing_config_value() {
        assert_eq!(
            usage_error_message(&["--config"]),
            "missing value for '--config'"
        );
        assert_eq!(
            usage_error_message(&["--config", "--help"]),
            "missing value for '--config'"
        );
    }

    #[test]
    fn rejects_duplicate_config_option() {
        assert_eq!(
            usage_error_message(&["--config", "a.json", "--config", "b.json", "auth"]),
            "'--config' given more than once"
        );
    }

    #[test]
    fn rejects_extra_positional_after_command() {
        assert_eq!(
            usage_error_message(&["auth", "extra"]),
            "unexpected argument 'extra'"
        );
        assert_eq!(
            usage_error_message(&["run", "auth"]),
            "unexpected argument 'auth'"
        );
        assert_eq!(
            usage_error_message(&["run", "--help"]),
            "unexpected argument '--help'"
        );
    }
}

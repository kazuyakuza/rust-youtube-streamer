//! Command-line parser tests: accepted forms, default-path resolution, and
//! every usage-error message, against in-memory argument slices only.

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
fn usage_text_advertises_default_config_path() {
    assert!(USAGE.contains(DEFAULT_CONFIG_PATH));
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

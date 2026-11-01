//! Pins for the six diagnostics validation rules: each named predicate
//! reports its own dotted field path, and an absent section stays neutral so
//! pre-0.14.0 configuration passes unchanged.

use crate::config::diagnostics_config::DiagnosticsConfig;
use crate::config::test_fixtures::valid_config;
use crate::config::validation::validate;
use crate::config::ConfigError;

use super::*;

fn diagnostics_of(config: &mut AppConfig) -> &mut DiagnosticsConfig {
    config
        .diagnostics
        .as_mut()
        .expect("the valid fixture carries a diagnostics section")
}

fn expect_validation_error(config: &AppConfig) -> ConfigError {
    match validate(config) {
        Err(error @ ConfigError::Validation { .. }) => error,
        Err(other) => panic!("expected a validation error, got: {other}"),
        Ok(()) => panic!("expected a validation error, got Ok"),
    }
}

#[test]
fn blank_output_file_reports_issue() {
    let mut config = valid_config();
    diagnostics_of(&mut config).output_file = Some("   ".to_string());
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("diagnostics.output_file"));
    assert!(message.contains("must not be empty or whitespace-only"));
}

#[test]
fn output_file_must_differ_from_chat_log() {
    let mut config = valid_config();
    let chat_log_file = config.chat.log_file.clone();
    diagnostics_of(&mut config).output_file = Some(chat_log_file);
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("diagnostics.output_file"));
    assert!(message.contains("chat log file"));
}

#[test]
fn nonpositive_summary_interval_reports_issue() {
    let mut config = valid_config();
    diagnostics_of(&mut config).summary_interval_seconds = Some(0);
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("diagnostics.summary_interval_seconds"));
    assert!(message.contains("must be positive"));
}

#[test]
fn nonpositive_max_file_bytes_reports_issue() {
    let mut config = valid_config();
    diagnostics_of(&mut config).max_file_bytes = Some(0);
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("diagnostics.max_file_bytes"));
    assert!(message.contains("must be positive"));
}

#[test]
fn nonpositive_max_generations_reports_issue() {
    let mut config = valid_config();
    diagnostics_of(&mut config).max_generations = Some(0);
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("diagnostics.max_generations"));
    assert!(message.contains("must be positive"));
}

#[test]
fn generation_count_over_limit_reports_issue() {
    let mut config = valid_config();
    diagnostics_of(&mut config).max_generations = Some(MAX_GENERATIONS_LIMIT + 1);
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains("diagnostics.max_generations"));
    assert!(message.contains("must be at most 1000"));
}

#[test]
fn section_absent_passes_every_rule() {
    let mut config = valid_config();
    config.diagnostics = None;
    assert!(validate(&config).is_ok());
}

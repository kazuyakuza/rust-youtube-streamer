//! Diagnostics-section validation rules: the six additional predicates that
//! guard the optional `diagnostics` section. Every rule is section-absent
//! neutral — an absent section or an absent field never reports an issue, so
//! pre-0.14.0 operator configuration keeps parsing unchanged.

use crate::config::diagnostics_config::DiagnosticsConfig;
use crate::config::error::ConfigIssue;
use crate::config::model::AppConfig;

use super::{blank_issue, is_blank, positive_value_issue};

pub(super) const MAX_GENERATIONS_LIMIT: u32 = 1000;
const CHAT_LOG_COLLISION_PROBLEM: &str =
    "must be different from the chat log file (diagnostics output must stay separate from the ordinary chat log)";

pub(super) fn validate_diagnostics(config: &AppConfig, issues: &mut Vec<ConfigIssue>) {
    let Some(diagnostics) = config.diagnostics.as_ref() else {
        return;
    };
    validate_positive_fields(diagnostics, issues);
    validate_generation_count(diagnostics, issues);
    if is_unusable_output_file(diagnostics) {
        issues.extend(blank_issue(
            "diagnostics.output_file",
            diagnostics.output_file().unwrap_or_default(),
        ));
    }
    if is_chat_log_collision(diagnostics, &config.chat.log_file) {
        issues.push(ConfigIssue::new(
            "diagnostics.output_file".to_string(),
            CHAT_LOG_COLLISION_PROBLEM.to_string(),
        ));
    }
}

fn validate_positive_fields(diagnostics: &DiagnosticsConfig, issues: &mut Vec<ConfigIssue>) {
    if is_inconsistent_summary_interval(diagnostics) {
        issues.push(positive_value_issue(
            "diagnostics.summary_interval_seconds",
            diagnostics.summary_interval_seconds.unwrap_or_default(),
        ));
    }
    if is_inconsistent_max_file_bytes(diagnostics) {
        issues.push(positive_value_issue(
            "diagnostics.max_file_bytes",
            diagnostics.max_file_bytes.unwrap_or_default(),
        ));
    }
    if is_inconsistent_max_generations(diagnostics) {
        issues.push(positive_value_issue(
            "diagnostics.max_generations",
            diagnostics.max_generations.unwrap_or_default(),
        ));
    }
}

fn validate_generation_count(diagnostics: &DiagnosticsConfig, issues: &mut Vec<ConfigIssue>) {
    if is_out_of_range_generation_count(diagnostics) {
        issues.push(ConfigIssue::new(
            "diagnostics.max_generations".to_string(),
            format!(
                "must be at most {MAX_GENERATIONS_LIMIT} (got {})",
                diagnostics.max_generations.unwrap_or_default()
            ),
        ));
    }
}

fn is_inconsistent_summary_interval(diagnostics: &DiagnosticsConfig) -> bool {
    matches!(diagnostics.summary_interval_seconds, Some(0))
}

fn is_inconsistent_max_file_bytes(diagnostics: &DiagnosticsConfig) -> bool {
    matches!(diagnostics.max_file_bytes, Some(0))
}

fn is_inconsistent_max_generations(diagnostics: &DiagnosticsConfig) -> bool {
    matches!(diagnostics.max_generations, Some(0))
}

fn is_out_of_range_generation_count(diagnostics: &DiagnosticsConfig) -> bool {
    diagnostics
        .max_generations
        .is_some_and(|generations| generations > MAX_GENERATIONS_LIMIT)
}

fn is_unusable_output_file(diagnostics: &DiagnosticsConfig) -> bool {
    diagnostics.output_file().is_some_and(is_blank)
}

fn is_chat_log_collision(diagnostics: &DiagnosticsConfig, chat_log_file: &str) -> bool {
    diagnostics.output_file() == Some(chat_log_file)
}

#[cfg(test)]
#[path = "validation_diagnostics_tests.rs"]
mod validation_diagnostics_tests;

//! Parse pins for the optional `diagnostics` section: the committed example
//! mirrors the schema, section-absent and field-absent shapes parse to `None`,
//! explicit flags parse verbatim, and unknown or wrong-typed values are
//! rejected at deserialize time.

use crate::config::diagnostics_config::{
    DiagnosticsConfig, DEFAULT_MAX_FILE_BYTES, DEFAULT_MAX_GENERATIONS,
    DEFAULT_SUMMARY_INTERVAL_SECONDS,
};
use crate::config::test_fixtures::{example_json_value, expect_load_error, write_temp_config};

use super::*;

fn load_modified(value: &serde_json::Value) -> AppConfig {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = write_temp_config(&dir, &value.to_string());
    load_from_path(&path).expect("modified test config must load")
}

fn load_example_section() -> DiagnosticsConfig {
    load_modified(&example_json_value())
        .diagnostics
        .expect("the embedded example declares the diagnostics section")
}

#[test]
fn loads_example_diagnostics_section() {
    let diagnostics = load_example_section();
    assert_eq!(diagnostics.pipeline_timing_events, Some(false));
    assert_eq!(diagnostics.queue_metrics, Some(false));
    assert_eq!(diagnostics.periodic_summary, Some(false));
    assert_eq!(diagnostics.summary_interval_seconds, Some(30));
    assert_eq!(
        diagnostics.output_file(),
        Some("logs/diagnostics.log"),
        "the example keeps diagnostics separate from logs/chat.log"
    );
    assert_eq!(diagnostics.max_file_bytes, Some(1_048_576));
    assert_eq!(diagnostics.max_generations, Some(3));
}

#[test]
fn diagnostics_section_absent_parses_to_none() {
    let mut value = example_json_value();
    value
        .as_object_mut()
        .expect("example JSON must be an object")
        .remove("diagnostics");
    let loaded = load_modified(&value);
    assert!(loaded.diagnostics.is_none());
}

#[test]
fn diagnostics_fields_absent_parse_to_none() {
    let mut value = example_json_value();
    value["diagnostics"] = serde_json::json!({});
    let loaded = load_modified(&value);
    let diagnostics = loaded
        .diagnostics
        .expect("the empty section still parses as present");
    assert_eq!(
        diagnostics,
        DiagnosticsConfig {
            pipeline_timing_events: None,
            queue_metrics: None,
            periodic_summary: None,
            summary_interval_seconds: None,
            output_file: None,
            max_file_bytes: None,
            max_generations: None,
        }
    );
    assert!(!diagnostics.pipeline_timing_enabled());
    assert!(!diagnostics.queue_metrics_enabled());
    assert!(!diagnostics.periodic_summary_enabled());
    assert_eq!(
        diagnostics.summary_interval(),
        DEFAULT_SUMMARY_INTERVAL_SECONDS
    );
    assert_eq!(diagnostics.max_file_bytes(), DEFAULT_MAX_FILE_BYTES);
    assert_eq!(diagnostics.max_generations(), DEFAULT_MAX_GENERATIONS);
}

#[test]
fn enabled_flags_parse_to_true() {
    let mut value = example_json_value();
    value["diagnostics"]["pipeline_timing_events"] = serde_json::json!(true);
    value["diagnostics"]["queue_metrics"] = serde_json::json!(true);
    value["diagnostics"]["periodic_summary"] = serde_json::json!(true);
    let diagnostics = load_modified(&value)
        .diagnostics
        .expect("the section parses as present");
    assert!(diagnostics.pipeline_timing_enabled());
    assert!(diagnostics.queue_metrics_enabled());
    assert!(diagnostics.periodic_summary_enabled());
}

#[test]
fn unknown_diagnostics_field_rejected() {
    let mut value = example_json_value();
    value["diagnostics"]
        .as_object_mut()
        .expect("diagnostics JSON must be an object")
        .insert("bogus".to_string(), serde_json::json!(true));
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = write_temp_config(&dir, &value.to_string());
    let error = expect_load_error(&path);
    assert!(matches!(error, ConfigError::Deserialization { .. }));
}

#[test]
fn wrong_type_flag_rejected() {
    let mut value = example_json_value();
    value["diagnostics"]["pipeline_timing_events"] = serde_json::json!("yes");
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = write_temp_config(&dir, &value.to_string());
    let error = expect_load_error(&path);
    assert!(matches!(error, ConfigError::Deserialization { .. }));
}

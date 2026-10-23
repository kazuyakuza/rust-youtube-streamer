//! Pins for the optional `youtube.broadcast.made_for_kids` property:
//! absent parses as `None`, true/false parse as the boolean, and
//! non-boolean values are rejected at deserialize time.

use std::path::PathBuf;

use crate::config::model::BroadcastConfig;

use super::*;

#[test]
fn committed_example_declares_made_for_kids_false() {
    let example_path = std::path::Path::new("config/config.example.json");
    let loaded = load_from_path(example_path).expect("committed example config must load");
    assert_eq!(broadcast_of(&loaded).made_for_kids, Some(false));
}

#[test]
fn made_for_kids_absent_parses_as_none() {
    let mut value = example_value();
    value["youtube"]["broadcast"]
        .as_object_mut()
        .expect("broadcast JSON must be an object")
        .remove("made_for_kids");
    let loaded = load_modified(&value);
    assert!(broadcast_of(&loaded).made_for_kids.is_none());
    assert_eq!(
        broadcast_of(&loaded).title,
        "Rust YouTube Streamer Prototype"
    );
}

#[test]
fn made_for_kids_true_parses() {
    let mut value = example_value();
    value["youtube"]["broadcast"]["made_for_kids"] = serde_json::json!(true);
    let loaded = load_modified(&value);
    assert_eq!(broadcast_of(&loaded).made_for_kids, Some(true));
}

#[test]
fn made_for_kids_wrong_type_rejected() {
    let mut value = example_value();
    value["youtube"]["broadcast"]["made_for_kids"] = serde_json::json!("yes");
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = write_temp_config(&dir, &value.to_string());
    let error = expect_load_error(&path);
    assert!(matches!(error, ConfigError::Deserialization { .. }));
    assert!(error.to_string().contains(&path.display().to_string()));
}

fn broadcast_of(config: &AppConfig) -> &BroadcastConfig {
    &config.youtube.broadcast
}

fn example_value() -> serde_json::Value {
    let contents = std::fs::read_to_string("config/config.example.json")
        .expect("committed example config must be readable");
    serde_json::from_str(&contents).expect("committed example config must be valid JSON")
}

fn load_modified(value: &serde_json::Value) -> AppConfig {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = write_temp_config(&dir, &value.to_string());
    load_from_path(&path).expect("modified test config must load")
}

fn write_temp_config(dir: &tempfile::TempDir, contents: &str) -> PathBuf {
    let path = dir.path().join("config.json");
    std::fs::write(&path, contents).expect("test config file must be writable");
    path
}

fn expect_load_error(path: &std::path::Path) -> ConfigError {
    match load_from_path(path) {
        Err(error) => error,
        Ok(_) => panic!("expected configuration loading to fail"),
    }
}

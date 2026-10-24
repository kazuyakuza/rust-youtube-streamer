//! Parse pins for the optional `ffmpeg.audio` silent-audio section:
//! section-absent/en-flag-absent → `None` (video-only), `true`/`false` parse
//! as the boolean, wrong types are rejected, and the committed example
//! ships `true`.

use std::path::PathBuf;

use crate::config::model::AudioConfig;

use super::*;

#[test]
fn committed_example_ships_audio_enabled_true() {
    let example_path = std::path::Path::new("config/config.example.json");
    let loaded = load_from_path(example_path).expect("committed example config must load");
    assert_eq!(
        loaded.ffmpeg.audio,
        Some(AudioConfig {
            enabled: Some(true),
        })
    );
}

#[test]
fn audio_section_absent_masks_to_none() {
    let mut value = example_value();
    value["ffmpeg"]
        .as_object_mut()
        .expect("ffmpeg JSON must be an object")
        .remove("audio");
    let loaded = load_modified(&value);
    assert!(loaded.ffmpeg.audio.is_none());
}

#[test]
fn audio_enabled_absent_inside_the_section_masks_to_none() {
    let mut value = example_value();
    value["ffmpeg"]["audio"] = serde_json::json!({});
    let loaded = load_modified(&value);
    assert_eq!(loaded.ffmpeg.audio, Some(AudioConfig { enabled: None }));
}

#[test]
fn audio_enabled_true_parses() {
    let mut value = example_value();
    value["ffmpeg"]["audio"] = serde_json::json!({ "enabled": true });
    let loaded = load_modified(&value);
    assert_eq!(
        loaded.ffmpeg.audio,
        Some(AudioConfig {
            enabled: Some(true),
        })
    );
}

#[test]
fn audio_enabled_false_parses() {
    let mut value = example_value();
    value["ffmpeg"]["audio"] = serde_json::json!({ "enabled": false });
    let loaded = load_modified(&value);
    assert_eq!(
        loaded.ffmpeg.audio,
        Some(AudioConfig {
            enabled: Some(false),
        })
    );
}

#[test]
fn audio_enabled_wrong_type_rejected() {
    let mut value = example_value();
    value["ffmpeg"]["audio"] = serde_json::json!({ "enabled": "yes" });
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = write_temp_config(&dir, &value.to_string());
    let error = expect_load_error(&path);
    assert!(matches!(error, ConfigError::Deserialization { .. }));
    assert!(error.to_string().contains(&path.display().to_string()));
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

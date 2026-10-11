//! `ChatStore` capacity, FIFO eviction, append-only log, and I/O-failure
//! tests, all against temporary directories only.

use std::fs;
use std::path::Path;

use crate::config::valid_config;

use super::*;

/// Test geometry as (height, top_margin, bottom_margin, line_height).
type TestGeometry = (u32, u32, u32, u32);

/// Exact division: (100 − 20 − 20) / 20 = 3 visible lines.
const EXACT_GEOMETRY: TestGeometry = (100, 20, 20, 20);
/// Fractional remainder: (110 − 20 − 20) / 20 = 3.5, floored to 3.
const FRACTIONAL_GEOMETRY: TestGeometry = (110, 20, 20, 20);
/// Margins exceed the height: the `checked_sub` overflow path.
const MARGINS_EXCEED_HEIGHT: TestGeometry = (u32::MAX, u32::MAX, u32::MAX, u32::MAX);
/// Zero line height is rejected before any division.
const ZERO_LINE_GEOMETRY: TestGeometry = (100, 20, 20, 0);
/// Usable height 20 is smaller than one 40-pixel line: capacity 0.
const TOO_SMALL_GEOMETRY: TestGeometry = (100, 40, 40, 40);

fn config_with_geometry(geometry: TestGeometry) -> AppConfig {
    let (height, top_margin, bottom_margin, line_height) = geometry;
    let mut config = valid_config();
    config.video.height = height;
    config.renderer.top_margin = top_margin;
    config.renderer.bottom_margin = bottom_margin;
    config.renderer.line_height = line_height;
    config
}

fn store_config(geometry: TestGeometry, log_path: &Path) -> AppConfig {
    let mut config = config_with_geometry(geometry);
    config.chat.log_file = log_path.to_string_lossy().into_owned();
    config
}

fn exact_geometry_store() -> (ChatStore, tempfile::TempDir, PathBuf) {
    let temp = tempfile::tempdir().expect("create temp dir");
    let log_path = temp.path().join("chat.log");
    let config = store_config(EXACT_GEOMETRY, &log_path);
    let settings = ChatStoreSettings::from_config(&config).expect("derive settings");
    let store = ChatStore::new(settings).expect("open store");
    (store, temp, log_path)
}

fn accept_all(store: &mut ChatStore, authors: &[&str]) {
    for author in authors {
        let message = ChatMessage::new(author, &format!("message from {author}"));
        store.accept(message).expect("accept must succeed");
    }
}

fn assert_capacity_error(config: &AppConfig) {
    let result = derive_visible_capacity(&config.video, &config.renderer);
    assert!(matches!(result, Err(ChatError::Capacity { .. })));
}

#[test]
fn capacity_uses_floor_division_of_usable_height() {
    let exact = config_with_geometry(EXACT_GEOMETRY);
    let fractional = config_with_geometry(FRACTIONAL_GEOMETRY);
    let exact_capacity = derive_visible_capacity(&exact.video, &exact.renderer);
    let fractional_capacity = derive_visible_capacity(&fractional.video, &fractional.renderer);
    assert_eq!(exact_capacity.expect("derive exact capacity"), 3);
    assert_eq!(fractional_capacity.expect("derive fractional capacity"), 3);
}

#[test]
fn capacity_rejects_unusable_or_overflowing_geometry() {
    let maxed_out = config_with_geometry(MARGINS_EXCEED_HEIGHT);
    assert_capacity_error(&maxed_out);
    let zero_line = config_with_geometry(ZERO_LINE_GEOMETRY);
    assert_capacity_error(&zero_line);
    let too_small = config_with_geometry(TOO_SMALL_GEOMETRY);
    assert_capacity_error(&too_small);
}

#[test]
fn new_store_starts_empty_and_formats_single_message() {
    let (mut store, _temp, _log_path) = exact_geometry_store();
    assert!(store.is_empty());
    assert_eq!(store.visible_text(), "");
    let hello = ChatMessage::new("alice", "Hello!");
    store.accept(hello).expect("accept");
    assert_eq!(store.visible_text(), "alice: Hello!");
}

#[test]
fn fifo_evicts_oldest_beyond_capacity() {
    let (mut store, _temp, _log_path) = exact_geometry_store();
    let authors = ["m1", "m2", "m3", "m4"];
    accept_all(&mut store, &authors);
    assert_eq!(
        store.visible_text(),
        "m2: message from m2\nm3: message from m3\nm4: message from m4"
    );
}

#[test]
fn log_family_and_ordering_preserved_beyond_queue() {
    let (mut store, _temp, log_path) = exact_geometry_store();
    let authors = ["m1", "m2", "m3", "m4"];
    accept_all(&mut store, &authors);
    let contents = fs::read_to_string(&log_path).expect("read log");
    let records: Vec<&str> = contents.lines().collect();
    assert_eq!(records.len(), 4);
    for (record, author) in records.iter().zip(authors) {
        assert!(record.contains(&format!(" {author}: ")));
    }
}

#[test]
fn log_record_format_with_message_timestamp() {
    let (mut store, _temp, log_path) = exact_geometry_store();
    let message = ChatMessage::new("alice", "Hello!").with_timestamp(Some(1728381296));
    store.accept(message).expect("accept must succeed");
    let contents = fs::read_to_string(&log_path).expect("read log");
    assert_eq!(contents, "1728381296 alice: Hello!\n");
}

#[test]
fn log_record_uses_generation_timestamp_when_message_has_none() {
    let (mut store, _temp, log_path) = exact_geometry_store();
    let hello = ChatMessage::new("alice", "Hello!");
    store.accept(hello).expect("accept");
    let contents = fs::read_to_string(&log_path).expect("read log");
    let record = contents.strip_suffix('\n').expect("record ends with LF");
    let (seconds, text) = record.split_once(' ').expect("space separator");
    assert!(seconds.parse::<u64>().is_ok());
    assert_eq!(text, "alice: Hello!");
}

#[test]
fn newlines_in_messages_flatten_to_one_record_one_display_line() {
    let (mut store, _temp, log_path) = exact_geometry_store();
    let first = ChatMessage::new("alice", "line1\nline2");
    store.accept(first).expect("accept");
    let second = ChatMessage::new("bob", "line3\r\nline4");
    store.accept(second).expect("accept");
    let contents = fs::read_to_string(&log_path).expect("read log");
    assert_eq!(contents.lines().count(), 2);
    assert_eq!(store.visible_text(), "alice: line1 line2\nbob: line3 line4");
}

#[test]
fn log_open_failure_is_typed_and_surfaces_path() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let log_path = temp.path().join("does-not-exist").join("chat.log");
    let config = store_config(EXACT_GEOMETRY, &log_path);
    let settings = ChatStoreSettings::from_config(&config).expect("derive settings");
    let error = ChatStore::new(settings).expect_err("open must fail");
    assert!(matches!(error, ChatError::LogOpen { .. }));
    assert!(error.to_string().contains(&log_path.display().to_string()));
}

#[test]
fn append_failure_rejects_message_and_log_stays_consistent() {
    let (mut store, _temp, log_path) = exact_geometry_store();
    let hello = ChatMessage::new("alice", "Hello!");
    store.accept(hello).expect("accept");
    let read_only = fs::OpenOptions::new()
        .read(true)
        .open(&log_path)
        .expect("open read-only");
    store.log_file = read_only;
    let rejected = ChatMessage::new("bob", "Rejected!");
    let error = store.accept(rejected).expect_err("append must fail");
    assert!(matches!(error, ChatError::LogAppend { .. }));
    let message = error.to_string();
    assert!(message.contains(&log_path.display().to_string()));
    assert!(message.contains("rejected"));
    assert_eq!(store.visible_text(), "alice: Hello!");
    let contents = fs::read_to_string(&log_path).expect("read log");
    assert_eq!(contents.lines().count(), 1);
}

#[test]
fn new_store_ignores_existing_log_history() {
    let temp = tempfile::tempdir().expect("create temp dir");
    let log_path = temp.path().join("chat.log");
    fs::write(&log_path, "old1\nold2\nold3\n").expect("pre-create log");
    let config = store_config(EXACT_GEOMETRY, &log_path);
    let settings = ChatStoreSettings::from_config(&config).expect("derive settings");
    let mut store = ChatStore::new(settings).expect("open store");
    assert!(store.is_empty());
    assert_eq!(store.visible_text(), "");
    let hello = ChatMessage::new("alice", "Hello!");
    store.accept(hello).expect("accept");
    let contents = fs::read_to_string(&log_path).expect("read log");
    let records: Vec<&str> = contents.lines().collect();
    assert_eq!(records.len(), 4);
    assert_eq!(records[..3].join(","), "old1,old2,old3");
    assert!(records[3].ends_with(" alice: Hello!"));
}

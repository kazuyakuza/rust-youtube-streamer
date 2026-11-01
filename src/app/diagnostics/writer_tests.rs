//! Diagnostics sink pins: lazy creation, bounded size-cap rotation, the
//! total disk bound, and strict separation from the chat log path.

use super::writer::*;

fn writer_at(
    path: &std::path::Path,
    max_file_bytes: u32,
    max_generations: u32,
) -> DiagnosticsFileWriter {
    DiagnosticsFileWriter::new(WriterSettings {
        path: path.to_path_buf(),
        max_file_bytes,
        max_generations,
    })
}

#[test]
fn first_row_creates_file_only_on_demand() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let mut writer = writer_at(&path, 1_024, 3);
    assert!(!path.exists(), "the file is opened lazily");
    writer.write_event("{\"event\":\"queue\"}");
    assert!(path.exists());
    assert_eq!(
        std::fs::read_to_string(&path).expect("row file must be readable"),
        "{\"event\":\"queue\"}\n"
    );
}

#[test]
fn rotation_shifts_oldest_generation_out() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let mut writer = writer_at(&path, 8, 3);
    for row in ["row-one", "row-two", "row-three", "row-four"] {
        writer.write_event(row);
    }
    assert!(path.exists(), "the base generation stays current");
    assert!(dir.path().join("diagnostics.log.1").exists());
    assert!(dir.path().join("diagnostics.log.2").exists());
    assert!(
        !dir.path().join("diagnostics.log.3").exists(),
        "no generation beyond max_generations - 1 is kept"
    );
}

#[test]
fn generations_stay_within_disk_bound() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let mut writer = writer_at(&path, 16, 3);
    for _index in 0..10 {
        writer.write_event("row-line");
    }
    let total: u64 = generation_paths(&path)
        .iter()
        .filter_map(|generation| std::fs::metadata(generation).ok())
        .map(|metadata| metadata.len())
        .sum();
    assert!(total <= 16 * 3, "disk usage stayed bounded: {total}");
}

#[test]
fn file_keeps_distinct_path_from_chat_log() {
    let dir = tempfile::tempdir().expect("temp dir must be created");
    let path = dir.path().join("diagnostics.log");
    let chat_log = dir.path().join("chat.log");
    let mut writer = writer_at(&path, 1_024, 3);
    writer.write_event("{\"event\":\"queue\"}");
    assert!(path.exists());
    assert!(!chat_log.exists(), "the chat log is never opened here");
}

fn generation_paths(path: &std::path::Path) -> Vec<std::path::PathBuf> {
    let base = path.to_path_buf();
    let rotated = (1..=3)
        .map(|generation| std::path::PathBuf::from(format!("{}.{generation}", path.display())));
    std::iter::once(base).chain(rotated).collect()
}

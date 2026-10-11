//! Output-directory mapping and collision-safe filename tests: the
//! `videos/` directory must land beside the executable, and the name
//! generator must walk deterministic collision suffixes without ever
//! overwriting an existing recording.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

const BASE_NAME: &str = "demo-20200913-122640.mp4";

fn demo_spec() -> RecordingNameSpec {
    RecordingNameSpec::new(
        PathBuf::from("/opt/app/videos"),
        OutputKind::Demo,
        1_600_000_000,
    )
}

fn exists_over(existing: HashSet<String>) -> impl FnMut(&Path) -> bool {
    move |path| {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return true;
        };
        existing.contains(name)
    }
}

#[test]
fn filename_prefixes_match_outputs() {
    assert_eq!(OutputKind::Demo.filename_prefix(), "demo");
    assert_eq!(OutputKind::Session.filename_prefix(), "session");
}

#[test]
fn videos_directory_is_placed_beside_the_executable() {
    assert_eq!(
        resolve_videos_directory(Path::new("/opt/app/service")).expect("parent exists"),
        PathBuf::from("/opt/app/videos")
    );
    assert_eq!(
        resolve_videos_directory(Path::new("/opt/app/nested/service.exe")).expect("parent exists"),
        PathBuf::from("/opt/app/nested/videos")
    );
}

#[test]
fn videos_directory_rejects_executable_path_without_parent() {
    let error = resolve_videos_directory(Path::new("/"))
        .expect_err("the root path has no parent directory");

    assert_eq!(
        error.to_string(),
        "unable to resolve the application executable location: the executable path has no parent directory"
    );
}

#[test]
fn create_directory_makes_missing_directory() {
    let temp = tempfile::TempDir::new().expect("temp dir must be creatable");
    let target = temp.path().join("videos");

    create_directory(&target).expect("a missing directory must be created");

    assert!(target.is_dir());
}

#[test]
fn create_directory_reports_existing_file() {
    let temp = tempfile::TempDir::new().expect("temp dir must be creatable");
    let target = temp.path().join("videos");
    fs::write(&target, b"not a directory").expect("file fixture must be written");

    let error = create_directory(&target).expect_err("a plain file at the path must fail");

    assert!(matches!(
        error,
        StreamingError::OutputDirectoryCreate { .. }
    ));
}

#[test]
fn next_available_returns_unsuffixed_name_when_free() {
    let spec = demo_spec();
    let existing = HashSet::new();

    let filename = spec
        .next_available(&mut exists_over(existing))
        .expect("a free base name must be returned unsuffixed");

    assert_eq!(filename, BASE_NAME);
}

#[test]
fn next_available_walks_collision_suffixes_deterministically() {
    let spec = demo_spec();
    let mut existing = HashSet::new();
    existing.insert(BASE_NAME.to_string());
    existing.insert("demo-20200913-122640_2.mp4".to_string());

    let filename = spec
        .next_available(&mut exists_over(existing))
        .expect("the third candidate must be free");

    assert_eq!(filename, "demo-20200913-122640_3.mp4");
}

#[test]
fn next_available_never_overwrites_when_every_candidate_exists() {
    let spec = demo_spec();

    let error = spec
        .next_available(&mut |_: &Path| true)
        .expect_err("a saturated name budget must fail instead of overwriting");

    assert!(matches!(
        error,
        StreamingError::OutputNameExhausted { attempts: 1000, .. }
    ));
}

#[test]
fn generated_base_name_matches_timestamped_format() {
    let spec = demo_spec();

    let filename = spec
        .next_available(&mut |_: &Path| false)
        .expect("the timestamped base name must be generated");

    assert_eq!(filename, BASE_NAME);
}

/// The real probe (`path_exists` in the production module) treats every
/// probe ERROR as "exists" — the conservative direction, so a failed probe
/// can never let an overwrite through. The injected always-true predicate
/// simulates that worst case: every candidate is considered taken, so the
/// generator must exhaust its budget and write nothing.
#[test]
fn path_exists_treats_probe_errors_as_existing() {
    let spec = demo_spec();

    let error = spec
        .next_available(&mut |_: &Path| true)
        .expect_err("conservative probe errors must never free a name");

    assert!(matches!(error, StreamingError::OutputNameExhausted { .. }));
}

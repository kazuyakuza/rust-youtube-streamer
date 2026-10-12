//! Fake FFmpeg executables as `#!/bin/sh` scripts written into a
//! `tempfile::TempDir` (0o755) at test time; they consume stdin and/or
//! fail deliberately and never consult a real ffmpeg. The last argument
//! each script receives is the MP4 target (the locked vector appends the
//! output path last). Gated `#[cfg(all(test, unix))]` via the `ffmpeg.rs`
//! declaration. Body 1 intentionally ends at `cat > "$1"` without
//! `exit 0` (task-5 planner ruling, verified in the dev image: writable
//! target exits 0, directory target exits 2) so test 6's non-zero-exit
//! contract holds.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::config::valid_config;

use super::*;

const SUCCESS_WRITER_BODY: &str = r#"while [ "$#" -gt 1 ]; do shift; done
cat > "$1""#;

const NONZERO_EXITER_BODY: &str = r#"awk 'BEGIN{for(i=0;i<200;i++)printf "diagnostic filler line %04d\n",i;print "FFMPEG_FAKE_FATAL_TAIL_MARKER"}' 1>&2
exit 2"#;

const SILENT_SUCCESS_BODY: &str = "cat > /dev/null; exit 0";

const IMMEDIATE_EXITER_BODY: &str = "exit 0";

fn write_fake_executable(directory: &Path, body: &str) -> PathBuf {
    let script_path = directory.join("fake-ffmpeg.sh");
    let script_content = format!("#!/bin/sh\n{body}\n");
    fs::write(&script_path, script_content).expect("the TempDir must accept the fake script");
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
        .expect("the fake script must become executable");
    script_path
}

fn pipeline_config(executable_path: &Path) -> AppConfig {
    let mut config = valid_config();
    config.ffmpeg.executable = executable_path.to_string_lossy().into_owned();
    config
}

fn started_fake(directory: &Path, body: &str, output_path: PathBuf) -> FfmpegProcess {
    let executable_path = write_fake_executable(directory, body);
    let config = pipeline_config(&executable_path);
    FfmpegProcess::start(&config, output_path).expect("the fake FFmpeg script must spawn")
}

#[test]
fn fake_ffmpeg_success_writes_exactly_the_piped_bytes() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let output_path = directory.path().join("out.mp4");
    let mut process = started_fake(directory.path(), SUCCESS_WRITER_BODY, output_path.clone());

    process
        .write_frame(&[7u8; 100])
        .expect("slice one must be accepted");
    process
        .write_frame(&[0u8; 250])
        .expect("slice two must be accepted");
    process
        .write_frame(&[42u8; 77])
        .expect("slice three must be accepted");

    let completed = process
        .finish()
        .expect("a natural stdin EOF must finalize the fake recording");
    assert_eq!(completed.path(), output_path.as_path());
    let recorded_bytes = fs::metadata(&output_path)
        .expect("the fake output must exist")
        .len();
    assert_eq!(recorded_bytes, 427);
}

#[test]
fn fake_ffmpeg_nonzero_exit_surfaces_code_and_diagnostics_tail() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let output_path = directory.path().join("out.mp4");
    let process = started_fake(directory.path(), NONZERO_EXITER_BODY, output_path);

    let Err(error) = process.finish() else {
        panic!("a non-zero fake exit must surface as ExitFailure");
    };
    let diagnostics = match &error {
        StreamingError::ExitFailure {
            exit_code,
            diagnostics,
        } => {
            assert_eq!(*exit_code, Some(2));
            diagnostics
        }
        other => panic!("expected ExitFailure, got {other}"),
    };
    assert!(error
        .to_string()
        .contains("FFmpeg exited with failure status 2"));
    assert!(diagnostics.contains("FFMPEG_FAKE_FATAL_TAIL_MARKER"));
    assert!(!diagnostics.contains("diagnostic filler line 0000"));
    assert!(diagnostics.chars().count() <= 4096);
}

#[test]
fn fake_ffmpeg_exit_zero_without_output_file_reports_output_finalize() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let output_path = directory.path().join("out.mp4");
    let process = started_fake(directory.path(), SILENT_SUCCESS_BODY, output_path.clone());

    let Err(error) = process.finish() else {
        panic!("exit 0 without an output file must fail finalization");
    };
    match &error {
        StreamingError::OutputFinalize { path } => assert_eq!(path, &output_path),
        other => panic!("expected OutputFinalize, got {other}"),
    }
    assert!(error
        .to_string()
        .contains("is missing or empty; the video was not finalized"));
}

#[test]
fn missing_executable_reports_typed_spawn_error() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let missing_path = directory.path().join("definitely-missing-ffmpeg");
    let config = pipeline_config(&missing_path);

    let Err(error) = FfmpegProcess::start(&config, directory.path().join("missing.mp4")) else {
        panic!("a missing executable must surface as a Spawn error");
    };

    let spawn_source = match &error {
        StreamingError::Spawn { source, .. } => source,
        other => panic!("expected Spawn, got {other}"),
    };
    assert_eq!(spawn_source.kind(), io::ErrorKind::NotFound);
    assert!(error.to_string().contains(
        "check that the configured path is correct and FFmpeg is installed and runnable",
    ));
}

#[test]
fn exited_child_surfaces_a_broken_pipe_frame_write() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let output_path = directory.path().join("out.mp4");
    let mut process = started_fake(directory.path(), IMMEDIATE_EXITER_BODY, output_path);
    let oversized_frame = vec![0u8; 76800];

    let error = process
        .write_frame(&oversized_frame)
        .expect_err("an exited child must fail an oversized frame write");

    assert!(matches!(error, StreamingError::FrameWrite { .. }));
    assert!(error
        .to_string()
        .contains("the FFmpeg process exited while receiving frames"));
}

#[test]
fn output_locked_by_existing_directory_surfaces_nonzero_exit() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let output_path = directory.path().join("locked.mp4");
    fs::create_dir(&output_path).expect("the lock-directory surrogate must exist");
    let process = started_fake(directory.path(), SUCCESS_WRITER_BODY, output_path);

    let Err(error) = process.finish() else {
        panic!("a locked output target must surface as ExitFailure");
    };
    match &error {
        StreamingError::ExitFailure { exit_code, .. } => assert!(exit_code.is_some()),
        other => panic!("expected ExitFailure, got {other}"),
    }
    assert!(error
        .to_string()
        .contains("FFmpeg exited with failure status"));
}

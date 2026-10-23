//! Streaming-spawn process tests (plan D-T3-5 §8 TS 5–7): fake FFmpeg
//! `#!/bin/sh` scripts in a `tempfile::TempDir` (0o755), never a real
//! ffmpeg. The stream target leaves no local file, so success is a clean
//! exit only, and the destination URL must never survive into diagnostics
//! or spawn errors.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::config::valid_config;

use super::*;

const FIXTURE_URL: &str = "rtmps://ingest.example.test/ingest/stream-key-fixture";
const STDIN_DRAIN_EXIT_ZERO_BODY: &str = "cat > /dev/null; exit 0";
const STDIN_DRAIN_ECHO_EXIT_ZERO_BODY: &str = r#"cat > /dev/null; echo "Output #0, flv, to 'rtmps://ingest.example.test/ingest/stream-key-fixture'" 1>&2; exit 0"#;
const URL_ECHOING_EXITER_BODY: &str = r#"echo "Output #0, flv, to 'rtmps://ingest.example.test/ingest/stream-key-fixture'" 1>&2
exit 1"#;
const ETXTBSY_SPAWN_ATTEMPTS: u8 = 8;
const ETXTBSY_ERRNO: i32 = 26;

fn write_fake_executable(directory: &Path, body: &str) -> PathBuf {
    let script_path = directory.join("fake-ffmpeg-stream.sh");
    let script_content = format!("#!/bin/sh\n{body}\n");
    fs::write(&script_path, script_content).expect("the TempDir must accept the fake script");
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
        .expect("the fake script must become executable");
    script_path
}

fn stream_config(executable_path: &Path) -> AppConfig {
    let mut config = valid_config();
    config.ffmpeg.executable = executable_path.to_string_lossy().into_owned();
    config
}

/// Bounded immediate retry (the `ffmpeg_tests.rs` ETXTBSY pattern, errno
/// 26): a parallel libtest fork can inherit our script write-fd.
fn start_stream_fake(config: &AppConfig, url: &str) -> Result<FfmpegProcess, StreamingError> {
    let mut attempt = FfmpegProcess::start_stream(config, destination(url));
    for _ in 1..ETXTBSY_SPAWN_ATTEMPTS {
        if !is_transient_etxtbsy(&attempt) {
            break;
        }
        std::thread::yield_now();
        attempt = FfmpegProcess::start_stream(config, destination(url));
    }
    attempt
}

fn is_transient_etxtbsy(attempt: &Result<FfmpegProcess, StreamingError>) -> bool {
    matches!(
        attempt,
        Err(StreamingError::Spawn { source, .. })
            if source.raw_os_error() == Some(ETXTBSY_ERRNO)
    )
}

fn destination(url: &str) -> RtmpsDestination {
    RtmpsDestination::new(url.to_string())
}

#[test]
fn clean_stream_exit_returns_the_stream_completion() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let executable_path = write_fake_executable(directory.path(), STDIN_DRAIN_EXIT_ZERO_BODY);
    let config = stream_config(&executable_path);
    let mut process = start_stream_fake(&config, FIXTURE_URL).expect("the fake must spawn");

    process
        .write_frame(&[0u8; 2304])
        .expect("one frame must be accepted");

    let completed = process
        .finish()
        .expect("a clean streaming exit must complete without a local finalize check");
    assert!(matches!(completed, CompletedOutput::Stream(_)));
}

#[test]
fn stream_completion_carries_exit_code_and_redacted_stderr_tail() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let executable_path = write_fake_executable(directory.path(), STDIN_DRAIN_ECHO_EXIT_ZERO_BODY);
    let config = stream_config(&executable_path);
    let mut process = start_stream_fake(&config, FIXTURE_URL).expect("the fake must spawn");

    process
        .write_frame(&[0u8; 2304])
        .expect("one frame must be accepted");

    let Ok(CompletedOutput::Stream(stream_exit)) = process.finish() else {
        panic!("a clean echo-exiting fake must complete as a Stream carrier");
    };
    assert_eq!(stream_exit.exit_code(), Some(0));
    assert!(stream_exit.stderr_tail().contains("[redacted]"));
    assert!(!stream_exit.stderr_tail().contains("rtmps://"));
    assert!(!stream_exit.stderr_tail().contains("stream-key-fixture"));
}

#[test]
fn stream_exit_failure_diagnostics_redact_the_destination() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let executable_path = write_fake_executable(directory.path(), URL_ECHOING_EXITER_BODY);
    let config = stream_config(&executable_path);
    let process = start_stream_fake(&config, FIXTURE_URL).expect("the fake must spawn");

    let Err(error) = process.finish() else {
        panic!("a non-zero fake exit must surface as ExitFailure");
    };
    let StreamingError::ExitFailure { diagnostics, .. } = &error else {
        panic!("expected ExitFailure, got {error}");
    };
    assert!(diagnostics.contains("[redacted]"));
    assert!(!diagnostics.contains("rtmps://"));
    assert!(!diagnostics.contains("stream-key-fixture"));
}

#[test]
fn stream_spawn_failure_never_carries_the_destination() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let missing_path = directory.path().join("definitely-missing-ffmpeg");
    let config = stream_config(&missing_path);

    let Err(error) = start_stream_fake(&config, FIXTURE_URL) else {
        panic!("a missing executable must surface as a Spawn error");
    };
    assert!(matches!(error, StreamingError::Spawn { .. }));
    let display = error.to_string();
    assert!(display.contains("definitely-missing-ffmpeg"));
    assert!(!display.contains("rtmps"));
    assert!(!display.contains("stream-key-fixture"));
}

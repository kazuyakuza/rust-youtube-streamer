//! Stop-path isolation process tests (Task 1): the isolation contract under
//! the container's delivery constraint (caller ruling 2026-10-25). The fake
//! FFmpeg child spawned through the PRODUCTION `spawn_with` path must be
//! UNAFFECTED by the stop signal while the app-shaped handler receives it,
//! then finalize cleanly at stdin EOF with exit 0; the structural half pins
//! the child's OWN process group. Offline, container-only: group facts via
//! `/proc/<pid>/stat` + `std::fs`, image coreutils `/bin/kill` (never a dep),
//! `ctrlc::set_handler` (never `crate::app`) sealing the module to std+config;
//! ETXTBSY precedent: the spawn retry below mirrors the sibling rosters.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::valid_config;

use super::*;

const FIXTURE_URL: &str = "rtmps://ingest.example.test/ingest/stream-key-fixture";
const GROUP_SIGINT_IMMUNE_STDIN_READER_BODY: &str = "cat > /dev/null; exit 0";
const ETXTBSY_SPAWN_ATTEMPTS: u8 = 8;
const ETXTBSY_ERRNO: i32 = 26;

/// Bounded poll (~5 s: 200 × 25 ms) for the app-shaped handler to see the stop signal.
const HANDLER_FLAG_WAIT_ATTEMPTS: u8 = 200;
const HANDLER_FLAG_WAIT_SLEEP_MS: u64 = 25;

/// One survival-window step and the step count after the signal was sent.
const PARENT_GROUP_SIGINT_PROOF_WINDOW_MS: u64 = 50;
const PARENT_GROUP_SIGINT_PROOF_ATTEMPTS: u8 = 20;

fn write_fake_executable(directory: &Path, body: &str) -> PathBuf {
    let script_path = directory.join("fake-ffmpeg-isolation.sh");
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

/// Reads `/proc/<pid>/stat` — `pid (comm) state ppid pgrp …` — past the LAST
/// `)` so a `comm` with spaces or parens cannot poison the field count;
/// `None` on any structural surprise (no panics).
fn process_group_from_stat(stat_text: &str) -> Option<i32> {
    let mut fields = stat_text.rsplit_once(')')?.1.split_whitespace();
    fields.next()?;
    fields.next()?;
    fields.next()?.parse::<i32>().ok()
}

/// Coreutils `/bin/kill -s INT <pid>` — direct-pid delivery to the test
/// process only; a sender that fails must fail THIS test, never skip it.
fn send_stop_signal_to_test_process() {
    let target_pid = std::process::id().to_string();
    let sender = Command::new("/bin/kill")
        .args(["-s", "INT", target_pid.as_str()])
        .output()
        .expect("the image must provide the coreutils /bin/kill sender");
    assert!(
        sender.status.success(),
        "the direct-pid stop signal must succeed: {}",
        String::from_utf8_lossy(&sender.stderr)
    );
}

/// The handler must observe the signal, else the survival proof is vacuous.
fn wait_for_handler_flag(handler_flag: &AtomicBool) {
    for _ in 0..HANDLER_FLAG_WAIT_ATTEMPTS {
        if handler_flag.load(Ordering::SeqCst) {
            return;
        }
        std::thread::sleep(Duration::from_millis(HANDLER_FLAG_WAIT_SLEEP_MS));
    }
    panic!("the stop signal must flip the app-shaped handler");
}

/// Honest proof structure (binding plan §1.3; caller ruling 2026-10-25): (a)
/// this container cannot re-enact a group-wide signal — every process sits in
/// pgid 1, so a group-directed SIGINT is a container-wide broadcast that
/// `/bin/kill` rejects and is unsafe here; (b) DIRECT-pid delivery proves the
/// app-shaped handler receives the stop signal while the isolated child is
/// UNAFFECTED; (c) the child's non-membership in the spawning group is pinned
/// by `stream_child_leads_its_own_process_group` (child pgid == child pid,
/// never the parent's); (d) the union of (b)+(c) — non-membership plus POSIX
/// kill(2)'s negative-pid group addressing — is the feasible honest isolation
/// proof here; Windows stays compile-checked + documented container-untestable.
#[test]
fn stream_child_survives_parent_group_sigint_and_finalizes_on_stdin_eof() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let executable_path =
        write_fake_executable(directory.path(), GROUP_SIGINT_IMMUNE_STDIN_READER_BODY);
    let config = stream_config(&executable_path);

    let handler_flag = Arc::new(AtomicBool::new(false));
    let flag_for_handler = Arc::clone(&handler_flag);
    ctrlc::set_handler(move || flag_for_handler.store(true, Ordering::SeqCst))
        .map_err(|error| error.to_string())
        .expect("the app-shaped handler must install before the child spawns");

    let mut process = start_stream_fake(&config, FIXTURE_URL).expect("the fake must spawn");
    process
        .write_frame(&[0u8; 2304])
        .expect("one frame must be accepted");

    send_stop_signal_to_test_process();
    wait_for_handler_flag(&handler_flag);

    let start = Instant::now();
    for attempt in 1..=PARENT_GROUP_SIGINT_PROOF_ATTEMPTS {
        let child_state = process
            .child
            .try_wait()
            .expect("the child state must stay readable");
        assert!(
            child_state.is_none(),
            "the isolated child must stay unaffected by the stop signal (attempt {attempt})"
        );
        std::thread::sleep(Duration::from_millis(PARENT_GROUP_SIGINT_PROOF_WINDOW_MS));
    }
    let steps = u64::from(PARENT_GROUP_SIGINT_PROOF_ATTEMPTS);
    let proof_window = Duration::from_millis(steps * PARENT_GROUP_SIGINT_PROOF_WINDOW_MS);
    assert!(
        start.elapsed() >= proof_window,
        "the survival window must span the full {proof_window:?}"
    );

    let completed = process
        .finish()
        .expect("stdin EOF must finalize the isolated child");
    let CompletedOutput::Stream(stream_exit) = completed else {
        panic!("the isolated fake must complete as a Stream carrier");
    };
    assert_eq!(stream_exit.exit_code(), Some(0));
}

#[test]
fn stream_child_leads_its_own_process_group() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let executable_path =
        write_fake_executable(directory.path(), GROUP_SIGINT_IMMUNE_STDIN_READER_BODY);
    let config = stream_config(&executable_path);
    let process = start_stream_fake(&config, FIXTURE_URL).expect("the fake must spawn");

    let child_pid = process.child.id();
    let child_stat = fs::read_to_string(format!("/proc/{child_pid}/stat"))
        .expect("the container must expose the child /proc entry");
    let child_group = process_group_from_stat(&child_stat).expect("the pgrp field must parse");
    let expected_group = i32::try_from(child_pid).expect("the child pid must fit an i32");
    assert_eq!(
        child_group, expected_group,
        "the isolated child must lead its own process group"
    );

    let completed = process
        .finish()
        .expect("stdin EOF must finalize the isolated child");
    let CompletedOutput::Stream(stream_exit) = completed else {
        panic!("the isolated fake must complete as a Stream carrier");
    };
    assert_eq!(stream_exit.exit_code(), Some(0));
}

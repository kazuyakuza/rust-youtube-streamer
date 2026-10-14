//! Starts FFmpeg without a shell, keeps refusing pipe deadlocks by draining
//! stdout and stderr on dedicated threads (the `pipe_drain` split), waits
//! for clean stdin-EOF finalization, and maps exit codes/diagnostic tails
//! to typed errors. Drain threads are joined BEFORE `child.wait()` so a
//! fast-exiting FFmpeg can never fill its pipe while we wait; there is no
//! timeout in Task 1 (hang handling belongs to Task 2's pacing/shutdown
//! design); the child is killed on Drop so an abandoned process cannot
//! leak into the operator's machine. The argument vector is deliberately
//! never echoed into logs or error text; diagnostics carry the captured
//! stderr tail only, with the streaming destination URL redacted
//! (`[redacted]`) before any error payload is built (plan D-T3-5 F-S3).
//! Two spawn paths share one engine: `start` targets a local MP4 (demo),
//! `start_stream` targets the RTMPS destination (production; no local
//! file).

use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ExitStatus};
use std::thread::JoinHandle;

use crate::config::model::AppConfig;
use crate::streaming::args;
use crate::streaming::error::StreamingError;
use crate::streaming::local_finalize::verify_local_output;
use crate::streaming::pipe_drain::{exit_diagnostics, join_drain, CapturedPipe};
use crate::streaming::rtmps_destination::RtmpsDestination;
use crate::streaming::sink::{CompletedOutput, FrameSink};
use crate::streaming::spawn::{spawn_with, SpawnPlan};

/// The redaction marker replacing every occurrence of the destination URL
/// in captured diagnostics (D-T1-11/D-T1-13; §7 pinned).
pub(super) const REDACTED_TEXT: &str = "[redacted]";

/// Supervised FFmpeg child process: complete frames go to its stdin, both
/// output pipes are continuously drained, and `finish` reports only a
/// verified finalization (a local recording, or a clean streaming exit).
/// No `Debug` derive is introduced: the RTMPS destination lives inside the
/// struct and must never become formattable.
pub struct FfmpegProcess {
    pub(super) child: Child,
    pub(super) stdin: Option<ChildStdin>,
    pub(super) local_output: Option<PathBuf>,
    pub(super) redaction_target: Option<String>,
    pub(super) stdout_drain: Option<JoinHandle<CapturedPipe>>,
    pub(super) stderr_drain: Option<JoinHandle<CapturedPipe>>,
}

impl FfmpegProcess {
    /// Builds arguments (validated config in, locked local vector out),
    /// spawns the child with piped stdin/stdout/stderr and no shell, then
    /// hands both output pipes to dedicated drain threads.
    pub fn start(config: &AppConfig, output_path: PathBuf) -> Result<Self, StreamingError> {
        let arguments = args::build_arguments(config, &output_path)?;
        let plan = SpawnPlan {
            arguments,
            local_output: Some(output_path),
            redaction_target: None,
        };
        spawn_with(config, plan)
    }

    /// The production streaming spawn: locked stream vector, the RTMPS
    /// destination as the final argument, no local output file, and the
    /// destination recorded as the stderr-redaction target.
    pub fn start_stream(
        config: &AppConfig,
        destination: RtmpsDestination,
    ) -> Result<Self, StreamingError> {
        let arguments = args::build_stream_arguments(config, &destination)?;
        let plan = SpawnPlan {
            arguments,
            local_output: None,
            redaction_target: Some(destination.as_url().to_string()),
        };
        spawn_with(config, plan)
    }

    /// Non-zero exit maps to the typed failure whose diagnostics carry the
    /// captured stderr tail with the destination redacted FIRST (D-T1-11).
    fn report_exit_failure(
        &self,
        status: &ExitStatus,
        stderr: &CapturedPipe,
    ) -> Result<(), StreamingError> {
        if status.success() {
            return Ok(());
        }
        Err(StreamingError::ExitFailure {
            exit_code: status.code(),
            diagnostics: exit_diagnostics(stderr, self.redaction_target.as_deref()),
        })
    }
}

impl FrameSink for FfmpegProcess {
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), StreamingError> {
        let Some(stdin) = self.stdin.as_mut() else {
            return Err(StreamingError::FrameWrite {
                source: io::Error::other("the FFmpeg process has already been finalized"),
            });
        };
        stdin
            .write_all(frame)
            .map_err(|source| StreamingError::FrameWrite { source })
    }

    fn finish(mut self) -> Result<CompletedOutput, StreamingError> {
        self.stdin = None;
        let _captured_stdout = join_drain(self.stdout_drain.take())?;
        let captured_stderr = join_drain(self.stderr_drain.take())?;
        let status = self
            .child
            .wait()
            .map_err(|source| StreamingError::WaitFailed { source })?;
        self.report_exit_failure(&status, &captured_stderr)?;
        let Some(output_path) = self.local_output.take() else {
            return Ok(CompletedOutput::Stream);
        };
        verify_local_output(output_path).map(CompletedOutput::Recording)
    }
}

impl Drop for FfmpegProcess {
    /// Kills a still-running child so an un-awaited `FfmpegProcess` cannot
    /// leak, then waits to reap it. Errors on an already-exited process are
    /// ignored.
    fn drop(&mut self) {
        let _kill_result = self.child.kill();
        let _wait_result = self.child.wait();
    }
}

#[cfg(all(test, unix))]
#[path = "ffmpeg_tests.rs"]
mod ffmpeg_tests;

#[cfg(all(test, unix))]
#[path = "ffmpeg_stream_tests.rs"]
mod ffmpeg_stream_tests;

#[cfg(all(test, unix))]
#[path = "ffmpeg_real_tests.rs"]
mod ffmpeg_real_tests;

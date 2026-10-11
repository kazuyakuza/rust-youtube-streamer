//! Starts FFmpeg without a shell, keeps refusing pipe deadlocks by draining
//! stdout and stderr on dedicated threads, waits for clean stdin-EOF
//! finalization, and maps exit codes/diagnostic tails to typed errors.
//! Drain threads are joined BEFORE `child.wait()` so a fast-exiting FFmpeg
//! can never fill its pipe while we wait; there is no timeout in Task 1
//! (hang handling belongs to Task 2's pacing/shutdown design); the child is
//! killed on Drop so an abandoned process cannot leak into the operator's
//! machine. The argument vector is deliberately never echoed into logs or
//! error text; diagnostics carry the captured stderr tail only.

use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::thread::{self, JoinHandle};

use crate::config::model::AppConfig;
use crate::streaming::args;
use crate::streaming::error::{trailing_bytes, StreamingError};
use crate::streaming::sink::{CompletedVideo, FrameSink};

const DIAGNOSTIC_CAPTURE_LIMIT_BYTES: usize = 4096;

/// Supervised FFmpeg child process: complete frames go to its stdin, both
/// output pipes are continuously drained, and `finish` reports only a
/// verified finalized recording.
pub struct FfmpegProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    output_path: PathBuf,
    stdout_drain: Option<JoinHandle<CapturedPipe>>,
    stderr_drain: Option<JoinHandle<CapturedPipe>>,
}

/// What a drain thread captured from one output pipe.
struct CapturedPipe {
    collected_bytes: Vec<u8>,
    final_error: Option<io::Error>, // a read failure ends reading; bytes stay
}

impl FfmpegProcess {
    /// Builds arguments (validated config in, locked vector out), spawns
    /// the child with piped stdin/stdout/stderr and no shell, then hands
    /// both output pipes to dedicated drain threads.
    pub fn start(config: &AppConfig, output_path: PathBuf) -> Result<Self, StreamingError> {
        let arguments = args::build_arguments(config, &output_path)?;
        let mut command = Command::new(&config.ffmpeg.executable);
        prepare_piped_command(&mut command, &arguments);
        let mut child = command.spawn().map_err(|source| StreamingError::Spawn {
            executable: config.ffmpeg.executable.clone(),
            source,
        })?;
        let stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(missing_stdin_error());
            }
        };
        let stdout_drain = spawn_drain(child.stdout.take());
        let stderr_drain = spawn_drain(child.stderr.take());
        Ok(Self {
            child,
            stdin: Some(stdin),
            output_path,
            stdout_drain: Some(stdout_drain),
            stderr_drain: Some(stderr_drain),
        })
    }
}

/// The piped stdin handle is expected after a successful spawn, but panic
/// is not acceptable: a missing handle becomes a typed frame-write error.
fn missing_stdin_error() -> StreamingError {
    StreamingError::FrameWrite {
        source: io::Error::other("stdin unexpectedly unavailable"),
    }
}

fn prepare_piped_command(command: &mut Command, arguments: &[String]) {
    command
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
}

fn spawn_drain<R: Read + Send + 'static>(pipe: Option<R>) -> JoinHandle<CapturedPipe> {
    thread::spawn(move || capture_pipe(pipe))
}

fn capture_pipe<R: Read>(pipe: Option<R>) -> CapturedPipe {
    let Some(mut pipe) = pipe else {
        return CapturedPipe {
            collected_bytes: Vec::new(),
            final_error: Some(io::Error::other(
                "an output pipe was unexpectedly unavailable",
            )),
        };
    };
    let mut collected_bytes = Vec::new();
    let final_error = pipe.read_to_end(&mut collected_bytes).err();
    CapturedPipe {
        collected_bytes,
        final_error,
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

    fn finish(mut self) -> Result<CompletedVideo, StreamingError> {
        self.stdin = None;
        let _captured_stdout = join_drain(self.stdout_drain.take())?;
        let captured_stderr = join_drain(self.stderr_drain.take())?;
        let status = self
            .child
            .wait()
            .map_err(|source| StreamingError::WaitFailed { source })?;
        report_exit_failure(&status, &captured_stderr)?;
        let output_path = self.output_path.clone();
        if !is_output_finalized(&output_path) {
            return Err(StreamingError::OutputFinalize { path: output_path });
        }
        Ok(CompletedVideo::new(output_path))
    }
}

/// Joins one drain thread. A missing handle or a panicked thread both map
/// to the typed wait failure — never an unwrap.
fn join_drain(drain: Option<JoinHandle<CapturedPipe>>) -> Result<CapturedPipe, StreamingError> {
    let Some(handle) = drain else {
        return Err(StreamingError::WaitFailed {
            source: io::Error::other("a drain thread handle was unexpectedly unavailable"),
        });
    };
    handle.join().map_err(|_| StreamingError::WaitFailed {
        source: io::Error::other("a drain thread panicked"),
    })
}

fn report_exit_failure(status: &ExitStatus, stderr: &CapturedPipe) -> Result<(), StreamingError> {
    if status.success() {
        return Ok(());
    }
    let mut diagnostics = trailing_bytes(&stderr.collected_bytes, DIAGNOSTIC_CAPTURE_LIMIT_BYTES);
    append_read_failure_note(&mut diagnostics, stderr.final_error.as_ref());
    Err(StreamingError::ExitFailure {
        exit_code: status.code(),
        diagnostics,
    })
}

fn append_read_failure_note(diagnostics: &mut String, read_failure: Option<&io::Error>) {
    if let Some(error) = read_failure {
        diagnostics.push_str(&format!(";\n(read failure: {error})"));
    }
}

/// Finalization check with the conservative direction: a probe ERROR
/// (permissions etc.) counts as NOT finalized, so success is never
/// reported on doubt.
fn is_output_finalized(output_path: &Path) -> bool {
    matches!(output_path.try_exists(), Ok(true)) && output_file_is_non_empty(output_path)
}

fn output_file_is_non_empty(output_path: &Path) -> bool {
    output_path
        .metadata()
        .is_ok_and(|metadata| metadata.len() > 0)
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

//! Shared spawn plumbing for both FFmpeg spawn paths (the Task-3 two-args
//! extraction): one no-shell `Command` with piped stdin/stdout/stderr and
//! drain threads attached before the child handle is returned. Secret
//! safety: `SpawnPlan.redaction_target` IS the secret — no `Debug`/`Display`
//! may ever be derived on `SpawnPlan`.

use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::config::model::AppConfig;
use crate::streaming::error::StreamingError;
use crate::streaming::ffmpeg::FfmpegProcess;
use crate::streaming::pipe_drain::spawn_drain;

/// The shared spawn inputs (≤2-args rule via parameter object): the locked
/// vector plus exactly one target shape — a local file to verify, or a
/// destination URL to redact. No `Debug`/`Display` may ever be derived:
/// `redaction_target` IS the secret.
pub(super) struct SpawnPlan {
    pub(super) arguments: Vec<String>,
    pub(super) local_output: Option<PathBuf>,
    pub(super) redaction_target: Option<String>,
}

/// The single spawn path shared by the local and streaming starts: piped
/// stdin/stdout/stderr, no shell, drain threads attached before returning.
pub(super) fn spawn_with(
    config: &AppConfig,
    plan: SpawnPlan,
) -> Result<FfmpegProcess, StreamingError> {
    let SpawnPlan {
        arguments,
        local_output,
        redaction_target,
    } = plan;
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
    Ok(FfmpegProcess {
        child,
        stdin: Some(stdin),
        local_output,
        redaction_target,
        stdout_drain: Some(stdout_drain),
        stderr_drain: Some(stderr_drain),
    })
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

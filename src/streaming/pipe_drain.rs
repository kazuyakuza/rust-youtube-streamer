//! The pipe-drain machinery shared by both FFmpeg spawn paths (plan
//! D-T3-5 F-S3 split, D-T3-14 line cap): one drain thread per output
//! pipe, joined BEFORE `child.wait()` so a fast-exiting FFmpeg can never
//! fill its pipe while we wait. Also the mandated redaction pass
//! (D-T1-11/D-T1-13): FFmpeg echoes the output URL in its stderr
//! (`Output #0, flv, to 'rtmps://…/<key>'`); every occurrence is replaced
//! BEFORE any `ExitFailure` payload is built.

use std::io::{self, Read};
use std::thread::{self, JoinHandle};

use crate::streaming::error::{trailing_bytes, StreamingError};
use crate::streaming::ffmpeg::REDACTED_TEXT;

/// How many trailing diagnostic bytes survive the capture cut.
pub(super) const DIAGNOSTIC_CAPTURE_LIMIT_BYTES: usize = 4096;

/// What a drain thread captured from one output pipe.
pub(super) struct CapturedPipe {
    pub(super) collected_bytes: Vec<u8>,
    pub(super) final_error: Option<io::Error>, // a read failure ends reading; bytes stay
}

pub(super) fn spawn_drain<R: Read + Send + 'static>(pipe: Option<R>) -> JoinHandle<CapturedPipe> {
    thread::spawn(move || capture_pipe(pipe))
}

/// Joins one drain thread. A missing handle or a panicked thread both map
/// to the typed wait failure — never an unwrap.
pub(super) fn join_drain(
    drain: Option<JoinHandle<CapturedPipe>>,
) -> Result<CapturedPipe, StreamingError> {
    let Some(handle) = drain else {
        return Err(StreamingError::WaitFailed {
            source: io::Error::other("a drain thread handle was unexpectedly unavailable"),
        });
    };
    handle.join().map_err(|_| StreamingError::WaitFailed {
        source: io::Error::other("a drain thread panicked"),
    })
}

/// The trailing-diagnostics extraction with the destination redaction
/// applied. The read-failure note is app-generated text appended before
/// this pass and can never carry the URL.
pub(super) fn exit_diagnostics(stderr: &CapturedPipe, redaction_target: Option<&str>) -> String {
    let mut diagnostics = trailing_bytes(&stderr.collected_bytes, DIAGNOSTIC_CAPTURE_LIMIT_BYTES);
    append_read_failure_note(&mut diagnostics, stderr.final_error.as_ref());
    redact_destination(&mut diagnostics, redaction_target);
    diagnostics
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

fn append_read_failure_note(diagnostics: &mut String, read_failure: Option<&io::Error>) {
    if let Some(error) = read_failure {
        diagnostics.push_str(&format!(";\n(read failure: {error})"));
    }
}

fn redact_destination(diagnostics: &mut String, redaction_target: Option<&str>) {
    if let Some(target) = redaction_target {
        *diagnostics = diagnostics.replace(target, REDACTED_TEXT);
    }
}

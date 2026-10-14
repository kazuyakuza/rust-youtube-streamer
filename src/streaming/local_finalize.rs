//! The local-output finalization check of the recording path (plan D-T3-5
//! F-S3 split, D-T3-14 line cap): the streaming finish never runs it —
//! RTMPS leaves no file behind — so it lives beside the local `start`
//! path's success contract.

use std::path::{Path, PathBuf};

use crate::streaming::error::StreamingError;
use crate::streaming::sink::CompletedVideo;

/// Finalization check with the conservative direction: a probe ERROR
/// (permissions etc.) counts as NOT finalized, so success is never
/// reported on doubt.
pub(super) fn verify_local_output(output_path: PathBuf) -> Result<CompletedVideo, StreamingError> {
    if !is_output_finalized(&output_path) {
        return Err(StreamingError::OutputFinalize { path: output_path });
    }
    Ok(CompletedVideo::new(output_path))
}

fn is_output_finalized(output_path: &Path) -> bool {
    matches!(output_path.try_exists(), Ok(true)) && output_file_is_non_empty(output_path)
}

fn output_file_is_non_empty(output_path: &Path) -> bool {
    output_path
        .metadata()
        .is_ok_and(|metadata| metadata.len() > 0)
}

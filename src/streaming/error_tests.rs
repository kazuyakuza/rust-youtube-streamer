//! Display-contract and error-source tests for [`StreamingError`]: every
//! variant must render its exact actionable message, and only the variants
//! that wrap an I/O cause must chain it through `source()`.

use std::io;
use std::path::PathBuf;

use super::*;

fn permission_denied() -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, "permission denied")
}

#[test]
fn format_errors_without_io_source() {
    assert_eq!(
        StreamingError::UnsupportedPixelFormat {
            value: "yuv420p".to_string()
        }
        .to_string(),
        "streaming pipeline requires pixel format 'rgb24' in configuration (got 'yuv420p')"
    );
    assert_eq!(
        StreamingError::ExecutableLocation {
            reason: "no executable".to_string()
        }
        .to_string(),
        "unable to resolve the application executable location: no executable"
    );
    assert_eq!(
        StreamingError::OutputDirectoryCreate {
            path: PathBuf::from("/opt/app/videos"),
            source: permission_denied(),
        }
        .to_string(),
        "unable to create video output directory '/opt/app/videos': permission denied"
    );
    assert_eq!(
        StreamingError::OutputNameExhausted {
            directory: PathBuf::from("/opt/app/videos"),
            attempts: 1000,
        }
        .to_string(),
        "video file naming gave up in directory '/opt/app/videos' after 1000 collision suffixes; the recording was not started"
    );
}

#[test]
fn format_errors_with_io_source() {
    assert_eq!(
        StreamingError::Spawn {
            executable: "ffmpeg".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "missing binary"),
        }
        .to_string(),
        "unable to start the FFmpeg executable 'ffmpeg': missing binary; check that the configured path is correct and FFmpeg is installed and runnable"
    );
    assert_eq!(
        StreamingError::Spawn {
            executable: "/usr/bin/ffmpeg".to_string(),
            source: permission_denied(),
        }
        .to_string(),
        "unable to start the FFmpeg executable '/usr/bin/ffmpeg': permission denied"
    );
    assert_eq!(
        StreamingError::FrameWrite {
            source: io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"),
        }
        .to_string(),
        "writing a frame to the FFmpeg process failed: broken pipe; the FFmpeg process exited while receiving frames"
    );
    assert_eq!(
        StreamingError::FrameWrite {
            source: permission_denied(),
        }
        .to_string(),
        "writing a frame to the FFmpeg process failed: permission denied"
    );
    assert_eq!(
        StreamingError::WaitFailed {
            source: io::Error::new(io::ErrorKind::Interrupted, "interrupted"),
        }
        .to_string(),
        "waiting for the FFmpeg process failed: interrupted"
    );
}

#[test]
fn format_exit_and_finalize_errors() {
    assert_eq!(
        StreamingError::ExitFailure {
            exit_code: Some(1),
            diagnostics: "Unknown encoder 'libx264'".to_string(),
        }
        .to_string(),
        "FFmpeg exited with failure status 1; captured diagnostics follow: Unknown encoder 'libx264'"
    );
    assert_eq!(
        StreamingError::ExitFailure {
            exit_code: None,
            diagnostics: "killed".to_string(),
        }
        .to_string(),
        "FFmpeg terminated by a signal; captured diagnostics follow: killed"
    );
    assert_eq!(
        StreamingError::OutputFinalize {
            path: PathBuf::from("/opt/app/videos/demo.mp4"),
        }
        .to_string(),
        "FFmpeg reported success but the output file '/opt/app/videos/demo.mp4' is missing or empty; the video was not finalized"
    );
}

#[test]
fn source_returns_io_error_where_defined() {
    let output_directory_create = StreamingError::OutputDirectoryCreate {
        path: PathBuf::from("/opt/app/videos"),
        source: permission_denied(),
    };
    let spawn = StreamingError::Spawn {
        executable: "ffmpeg".to_string(),
        source: permission_denied(),
    };
    let frame_write = StreamingError::FrameWrite {
        source: permission_denied(),
    };
    let wait_failed = StreamingError::WaitFailed {
        source: permission_denied(),
    };
    let unsupported_pixel_format = StreamingError::UnsupportedPixelFormat {
        value: "gray".to_string(),
    };
    let executable_location = StreamingError::ExecutableLocation {
        reason: "no executable".to_string(),
    };
    let output_name_exhausted = StreamingError::OutputNameExhausted {
        directory: PathBuf::from("/opt/app/videos"),
        attempts: 1000,
    };
    let exit_failure = StreamingError::ExitFailure {
        exit_code: Some(2),
        diagnostics: "boom".to_string(),
    };
    let output_finalize = StreamingError::OutputFinalize {
        path: PathBuf::from("/opt/app/videos/demo.mp4"),
    };

    assert!(output_directory_create.source().is_some());
    assert!(spawn.source().is_some());
    assert!(frame_write.source().is_some());
    assert!(wait_failed.source().is_some());
    assert!(unsupported_pixel_format.source().is_none());
    assert!(executable_location.source().is_none());
    assert!(output_name_exhausted.source().is_none());
    assert!(exit_failure.source().is_none());
    assert!(output_finalize.source().is_none());
}

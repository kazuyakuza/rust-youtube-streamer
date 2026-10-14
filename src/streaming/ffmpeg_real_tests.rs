//! Real-FFmpeg smoke: records a tiny black clip through the locked
//! argument vector inside the dev-checks image and probes the MP4 with
//! ffprobe (dimensions, frame rate, duration). An absent image tool is a
//! suite failure, never a silent skip.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use crate::config::valid_config;

use super::*;

const SMOKE_FRAME_BYTES: usize = 64 * 36 * 3;
const SMOKE_FRAME_COUNT: usize = 10;
const MIN_PLAUSIBLE_DURATION_SECONDS: f64 = 1.5;
const MAX_PLAUSIBLE_DURATION_SECONDS: f64 = 2.5;

/// The real-FFmpeg smoke is part of the dev-checks suite: the pinned image
/// installs ffmpeg (Dockerfile). An absent ffmpeg binary is a suite
/// failure, never a silent skip (no invented / absent-as-green evidence).
fn require_image_tool(label: &str, path: &Path) -> PathBuf {
    assert!(
        path.exists(),
        "{label} is missing at {}: the pinned dev-checks image must install ffmpeg (Dockerfile decision T5-D1)",
        path.display()
    );
    path.to_path_buf()
}

fn probe_value<'a>(probe_stdout: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}=");
    probe_stdout
        .lines()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .map(str::trim)
}

fn probe_recording(ffprobe_path: &Path, output_path: &Path) -> Output {
    Command::new(ffprobe_path)
        .args([
            "-v",
            "error",
            "-select_streams",
            "v:0",
            "-show_entries",
            "stream=width,height,r_frame_rate:format=duration",
            "-of",
            "default=noprint_wrappers=1",
        ])
        .arg(output_path)
        .output()
        .expect("ffprobe must run inside the dev-checks image")
}

#[test]
fn real_ffmpeg_records_and_probes_plausible_video() {
    let ffmpeg_path = require_image_tool("ffmpeg", Path::new("/usr/bin/ffmpeg"));
    let ffprobe_path = require_image_tool("ffprobe", Path::new("/usr/bin/ffprobe"));
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let output_path = directory.path().join("real-smoke.mp4");

    let mut config = valid_config();
    config.video.width = 64;
    config.video.height = 36;
    config.video.fps = 5;
    config.ffmpeg.executable = ffmpeg_path.to_string_lossy().into_owned();

    let mut process = FfmpegProcess::start(&config, output_path.clone())
        .expect("the image FFmpeg must spawn through the locked vector");
    for _ in 0..SMOKE_FRAME_COUNT {
        process
            .write_frame(&[0u8; SMOKE_FRAME_BYTES])
            .expect("black frames must stream through stdin");
    }
    let CompletedOutput::Recording(completed) = process
        .finish()
        .expect("a natural stdin EOF must finalize the real recording")
    else {
        panic!("the local recording path must complete as Recording");
    };
    assert_eq!(completed.path(), output_path.as_path());
    let recorded_bytes = fs::metadata(&output_path)
        .expect("the recording must exist")
        .len();
    assert!(recorded_bytes > 0);

    let probe = probe_recording(&ffprobe_path, &output_path);
    assert!(
        probe.status.success(),
        "ffprobe must probe the recording: {}",
        String::from_utf8_lossy(&probe.stderr)
    );

    let probe_stdout = String::from_utf8_lossy(&probe.stdout);
    assert_eq!(probe_value(&probe_stdout, "width"), Some("64"));
    assert_eq!(probe_value(&probe_stdout, "height"), Some("36"));
    assert_eq!(probe_value(&probe_stdout, "r_frame_rate"), Some("5/1"));
    let duration: f64 = probe_value(&probe_stdout, "duration")
        .expect("ffprobe must report the container duration")
        .parse()
        .expect("the reported duration must be numeric");
    assert!(
        (MIN_PLAUSIBLE_DURATION_SECONDS..=MAX_PLAUSIBLE_DURATION_SECONDS).contains(&duration),
        "expected ~2.0 s (10 frames at 5 fps), got {duration}"
    );
}

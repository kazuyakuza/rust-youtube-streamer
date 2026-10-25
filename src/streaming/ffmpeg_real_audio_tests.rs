//! Real-FFmpeg silent-audio mechanism pin (plan 20261024 §D9-C): the
//! production with-audio streaming shape against a LOCAL FLV target — no
//! network, ever. An absent image tool is a suite failure, never a skip.

use std::fs;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Output};

use super::ffmpeg_real_tests::{probe_value, require_image_tool};

const SMOKE_FRAME_BYTES: usize = 64 * 36 * 3;
const SMOKE_FRAME_COUNT: usize = 10;
const REAL_AUDIO_BOUND_WAIT_ATTEMPTS: u64 = 15;
const REAL_AUDIO_BOUND_WAIT_SLEEP_MS: u64 = 1000;

/// The production with-audio streaming shape with a LOCAL FLV target instead
/// of an RTMPS destination — no network, ever. Pins the `-shortest`
/// mechanism: the infinite anullsrc silence must be bounded at stdin EOF.
#[test]
fn real_ffmpeg_audio_stream_bounds_the_infinite_silence_and_carries_aac_audio() {
    let ffmpeg_path = require_image_tool("ffmpeg", Path::new("/usr/bin/ffmpeg"));
    let ffprobe_path = require_image_tool("ffprobe", Path::new("/usr/bin/ffprobe"));
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let output_path = directory.path().join("real-audio-smoke.flv");

    let mut child = spawn_audio_smoke(&ffmpeg_path, &output_path);
    write_audio_smoke_frames(&mut child);
    let status = wait_audio_smoke_bounded(&mut child);
    assert!(
        status.success(),
        "the bounded audio smoke must exit cleanly: {status}"
    );
    let smoke_bytes = fs::metadata(&output_path)
        .expect("the audio smoke FLV must exist")
        .len();
    assert!(smoke_bytes > 0);

    let probe = probe_audio_smoke(&ffprobe_path, &output_path);
    assert!(
        probe.status.success(),
        "ffprobe must probe the audio smoke FLV: {}",
        String::from_utf8_lossy(&probe.stderr)
    );
    assert_smoke_streams_carry_h264_and_aac(&String::from_utf8_lossy(&probe.stdout));
}

fn spawn_audio_smoke(ffmpeg_path: &Path, output_path: &Path) -> Child {
    let mut command = Command::new(ffmpeg_path);
    command
        .args([
            "-nostdin",
            "-n",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            "64x36",
            "-framerate",
            "5",
            "-i",
            "pipe:0",
            "-f",
            "lavfi",
            "-i",
            "anullsrc=channel_layout=stereo:sample_rate=48000",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-b:v",
            "6000k",
            "-g",
            "10",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-shortest",
            "-f",
            "flv",
        ])
        .arg(output_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    // D10 collateral immunity: keep this suite child out of the test process group.
    command.process_group(0);
    command
        .spawn()
        .expect("the image FFmpeg must spawn the audio smoke vector")
}

fn write_audio_smoke_frames(child: &mut Child) {
    let stdin = child
        .stdin
        .as_mut()
        .expect("the audio smoke child must expose stdin");
    for _ in 0..SMOKE_FRAME_COUNT {
        stdin
            .write_all(&[0u8; SMOKE_FRAME_BYTES])
            .expect("black frames must stream through stdin");
    }
}

fn wait_audio_smoke_bounded(child: &mut Child) -> ExitStatus {
    drop(child.stdin.take());
    let mut attempts = 0;
    loop {
        let status = child
            .try_wait()
            .expect("joining the audio smoke must not fail with io");
        if let Some(status) = status {
            return status;
        }
        attempts += 1;
        if attempts >= REAL_AUDIO_BOUND_WAIT_ATTEMPTS {
            let _ = child.kill();
            let _ = child.wait();
            panic!(
                "the anullsrc audio stream was not bounded by -shortest at stdin EOF; mechanism 1 failed (D5)"
            );
        }
        std::thread::sleep(std::time::Duration::from_millis(
            REAL_AUDIO_BOUND_WAIT_SLEEP_MS,
        ));
    }
}

fn probe_audio_smoke(ffprobe_path: &Path, output_path: &Path) -> Output {
    let mut command = Command::new(ffprobe_path);
    command
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type,codec_name,sample_rate,channels",
            "-of",
            "default=noprint_wrappers=1",
        ])
        .arg(output_path);
    // D10 collateral immunity: keep this suite child out of the test process group.
    command.process_group(0);
    command
        .output()
        .expect("ffprobe must run inside the dev-checks image")
}

fn assert_smoke_streams_carry_h264_and_aac(probe_stdout: &str) {
    let codec_names: Vec<&str> = probe_stdout
        .lines()
        .filter_map(|line| line.strip_prefix("codec_name="))
        .collect();
    assert!(
        codec_names.contains(&"h264"),
        "the video stream must be h264: {probe_stdout}"
    );
    assert!(
        codec_names.contains(&"aac"),
        "the audio stream must be aac: {probe_stdout}"
    );
    assert_eq!(probe_value(probe_stdout, "sample_rate"), Some("48000"));
    assert_eq!(probe_value(probe_stdout, "channels"), Some("2"));
}

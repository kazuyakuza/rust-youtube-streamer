//! Streaming-stage process test (plan §8 TE, `#[cfg(all(test, unix))]`):
//! the PRODUCTION `prepared_stream_pipeline` construction passes the
//! destination as the FINAL ffmpeg argv token, with `-f flv` immediately
//! before it, no `-movflags`, and the 2×-fps GOP — captured by a
//! temp-dir POSIX fake-FFmpeg script that never consults a real ffmpeg.

use std::fs;
use std::os::unix::fs::PermissionsExt;

use crate::app::pipeline::{prepared_stream_pipeline, StopFlag, StreamParts};
use crate::app::run::runtime_test_fixtures::DESTINATION_URL_FIXTURE;
use crate::app::run::source::ChatFreeSource;
use crate::config::valid_config;
use crate::streaming::{CompletedOutput, RtmpsDestination};

/// Writes the received argv one-per-line beside the script (`$0.argv`),
/// drains stdin, exits 0.
const ARGV_CAPTURE_BODY: &str = r#"printf '%s\n' "$@" > "$0.argv"
cat > /dev/null
exit 0"#;

const SCRIPT_NAME: &str = "fake-ffmpeg-argv.sh";

#[test]
fn stream_pipeline_passes_the_destination_to_the_ffmpeg_argv() {
    let directory = tempfile::tempdir().expect("a TempDir must be creatable");
    let script_path = directory.path().join(SCRIPT_NAME);
    fs::write(&script_path, format!("#!/bin/sh\n{ARGV_CAPTURE_BODY}\n"))
        .expect("the TempDir must accept the fake script");
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
        .expect("the fake script must become executable");
    let mut config = valid_config();
    config.ffmpeg.executable = script_path.to_string_lossy().into_owned();
    config.chat.log_file = directory
        .path()
        .join("chat.log")
        .to_string_lossy()
        .into_owned();
    let stop_flag = StopFlag::new();
    stop_flag.request_stop();

    let pipeline = prepared_stream_pipeline(
        &config,
        StreamParts {
            source: Box::new(ChatFreeSource),
            stop_flag,
            destination: RtmpsDestination::new(DESTINATION_URL_FIXTURE.to_string()),
        },
    )
    .expect("the production streaming pipeline must construct");

    let completed = pipeline
        .run()
        .expect("the pre-stopped zero-frame run must complete");
    assert!(matches!(completed, CompletedOutput::Stream(_)));

    let capture_path = directory.path().join(format!("{SCRIPT_NAME}.argv"));
    let argv: Vec<String> = fs::read_to_string(capture_path)
        .expect("the fake must capture its argv")
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(
        argv.last().map(String::as_str),
        Some(DESTINATION_URL_FIXTURE),
        "the destination URL is the final argument"
    );
    let length = argv.len();
    assert_eq!(argv[length - 3], "-f");
    assert_eq!(argv[length - 2], "flv", "the FLV muxer precedes the URL");
    assert!(
        !argv.iter().any(|argument| argument == "-movflags"),
        "no MP4-only faststart rewrite for the stream target"
    );
    assert!(
        argv.windows(2)
            .any(|pair| pair[0] == "-g" && pair[1] == "60"),
        "the GOP is 2 × the configured 30 fps"
    );
}

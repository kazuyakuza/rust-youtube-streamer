//! Streaming-vector tests: the locked with/without-audio argument vectors
//! must be produced verbatim from a validated configuration, and every
//! audio-off shape must hold the identical video-only wire form.

use crate::config::model::AudioConfig;
use crate::config::valid_config;

use super::*;

const DESTINATION_FIXTURE_URL: &str = "rtmps://ingest.example.test/ingest/stream-key-fixture";

#[test]
fn stream_arguments_match_the_locked_output_shape() {
    let mut config = valid_config();
    config.video.width = 1280;
    config.video.height = 720;
    config.video.fps = 25;
    // The explicit None pins the pre-0.11 video-only wire form deliberately.
    config.ffmpeg.audio = None;
    let destination = RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string());

    let arguments = build_stream_arguments(&config, &destination)
        .expect("a valid rgb24 configuration must build the stream vector");

    assert_eq!(
        arguments,
        vec![
            "-nostdin",
            "-n",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            "1280x720",
            "-framerate",
            "25",
            "-i",
            "pipe:0",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-b:v",
            "6000k",
            "-g",
            "50",
            "-pix_fmt",
            "yuv420p",
            "-f",
            "flv",
            DESTINATION_FIXTURE_URL,
        ]
    );
    assert!(!arguments.iter().any(|argument| argument == "-movflags"));
    let length = arguments.len();
    assert_eq!(arguments[length - 3], "-f");
    assert_eq!(arguments[length - 2], "flv");
}

#[test]
fn stream_arguments_with_audio_match_the_locked_extended_shape() {
    let mut config = valid_config();
    config.video.width = 1280;
    config.video.height = 720;
    config.video.fps = 25;
    let destination = RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string());

    let arguments = build_stream_arguments(&config, &destination)
        .expect("a valid rgb24 configuration must build the with-audio stream vector");

    assert_eq!(
        arguments,
        vec![
            "-nostdin",
            "-n",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            "1280x720",
            "-framerate",
            "25",
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
            "50",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-b:a",
            "128k",
            "-shortest",
            "-f",
            "flv",
            DESTINATION_FIXTURE_URL,
        ]
    );
    assert_eq!(
        arguments[11..16],
        [
            "pipe:0",
            "-f",
            "lavfi",
            "-i",
            "anullsrc=channel_layout=stereo:sample_rate=48000",
        ]
        .map(String::from),
        "the audio input pair rides immediately after the video input pair"
    );
    assert!(!arguments.iter().any(|argument| argument == "-movflags"));
}

#[test]
fn audio_absent_false_and_unset_enabled_hold_the_identical_video_only_vector() {
    let mut without_section = valid_config();
    without_section.ffmpeg.audio = None;
    let mut without_flag = valid_config();
    without_flag.ffmpeg.audio = Some(AudioConfig { enabled: None });
    let mut explicitly_disabled = valid_config();
    explicitly_disabled.ffmpeg.audio = Some(AudioConfig {
        enabled: Some(false),
    });

    let section_absent = build_stream_arguments(
        &without_section,
        &RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string()),
    )
    .expect("section-absent audio must build the video-only vector");
    let flag_absent = build_stream_arguments(
        &without_flag,
        &RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string()),
    )
    .expect("flag-absent audio must build the video-only vector");
    let disabled = build_stream_arguments(
        &explicitly_disabled,
        &RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string()),
    )
    .expect("disabled audio must build the video-only vector");

    assert_eq!(section_absent, flag_absent);
    assert_eq!(flag_absent, disabled);
}

#[test]
fn stream_gop_is_two_frames_per_second() {
    let destination = RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string());
    let mut config = valid_config();
    config.video.fps = 30;

    let arguments =
        build_stream_arguments(&config, &destination).expect("30 fps must build the vector");
    assert_eq!(gop_of(&arguments), Some("60".to_string()));

    config.video.fps = 60;
    let arguments =
        build_stream_arguments(&config, &destination).expect("60 fps must build the vector");
    assert_eq!(gop_of(&arguments), Some("120".to_string()));
}

fn gop_of(arguments: &[String]) -> Option<String> {
    let position = arguments.iter().position(|argument| argument == "-g")?;
    arguments.get(position + 1).cloned()
}

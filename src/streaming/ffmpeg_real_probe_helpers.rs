//! Shared real-FFmpeg probe helpers (stream-health Task 2 extraction, no
//! `#[test]` here): the measured-bitrate and duration floor assertions plus
//! their recorded tolerance constants, consumed by the real-audio pin so
//! each roster file stays under the line cap.

use super::ffmpeg_real_tests::probe_value;

/// Measured-vs-declared tolerance recorded from the plan §1.3 probes: the
/// CBR filler makes the measured average ≈ the declared sum, with mux
/// overhead and startup skew on short clips (10.4 s probe: 5779767 of
/// 6128000 ≈ 94.3%). Both floors are 90% of the declared values; the total
/// floor subsumes the 128000 audio contribution.
const VIDEO_CBR_STREAM_BITRATE_FLOOR: u64 = 5_400_000;
const MEASURED_TOTAL_BITRATE_FLOOR: u64 = 5_515_200;
const MIN_MEASURED_DURATION_SECONDS: f64 = 8.0;
const MAX_MEASURED_DURATION_SECONDS: f64 = 13.0;

fn probe_last_value<'a>(probe_stdout: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}=");
    probe_stdout
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .map(str::trim)
}

pub(super) fn assert_smoke_streams_carry_h264_and_mp3_at_the_measured_floors(probe_stdout: &str) {
    let codec_names: Vec<&str> = probe_stdout
        .lines()
        .filter_map(|line| line.strip_prefix("codec_name="))
        .collect();
    assert!(
        codec_names.contains(&"h264"),
        "the video stream must be h264: {probe_stdout}"
    );
    assert!(
        codec_names.contains(&"mp3"),
        "the audio stream must be mp3: {probe_stdout}"
    );
    assert_eq!(probe_value(probe_stdout, "sample_rate"), Some("48000"));
    assert_eq!(probe_value(probe_stdout, "channels"), Some("2"));
    let video_bitrate: u64 = probe_value(probe_stdout, "bit_rate")
        .expect("ffprobe must report the stream bit_rate")
        .parse()
        .expect("the stream bit_rate must be numeric");
    assert!(
        video_bitrate >= VIDEO_CBR_STREAM_BITRATE_FLOOR,
        "the CBR-filled video stream must reach the encoder-declared floor: {video_bitrate}"
    );
    let measured_total: u64 = probe_last_value(probe_stdout, "bit_rate")
        .expect("ffprobe must report the container bit_rate")
        .parse()
        .expect("the container bit_rate must be numeric");
    assert!(
        measured_total >= MEASURED_TOTAL_BITRATE_FLOOR,
        "the measured container bitrate must reach the floor: {measured_total}"
    );
    let measured_duration: f64 = probe_last_value(probe_stdout, "duration")
        .expect("ffprobe must report the container duration")
        .parse()
        .expect("the container duration must be numeric");
    assert!(
        (MIN_MEASURED_DURATION_SECONDS..=MAX_MEASURED_DURATION_SECONDS)
            .contains(&measured_duration),
        "expected ~10.4 s of filler measurement, got {measured_duration}"
    );
}

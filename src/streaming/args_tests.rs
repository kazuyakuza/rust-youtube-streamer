//! Argument-vector tests: the locked local recording vector must be
//! produced verbatim from a validated configuration, and the defensive
//! pixel-format guard must refuse any input that is not `rgb24`.

use std::path::PathBuf;

use crate::config::valid_config;

use super::*;

#[test]
fn valid_config_builds_exact_locked_vector() {
    let mut config = valid_config();
    config.video.width = 1280;
    config.video.height = 720;
    config.video.fps = 25;
    let output_path = PathBuf::from("/opt/app/videos/demo-20200913-122640.mp4");

    let arguments = build_arguments(&config, &output_path)
        .expect("a valid rgb24 configuration must build the argument vector");

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
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "/opt/app/videos/demo-20200913-122640.mp4",
        ]
    );
}

#[test]
fn non_rgb24_pixel_format_is_rejected_defensively() {
    let mut config = valid_config();
    config.video.pixel_format = "yuv420p".to_string();
    let output_path = PathBuf::from("/opt/app/videos/demo.mp4");

    let error = build_arguments(&config, &output_path)
        .expect_err("a non-rgb24 pixel format must be rejected");

    assert_eq!(
        error.to_string(),
        "streaming pipeline requires pixel format 'rgb24' in configuration (got 'yuv420p')"
    );
    assert!(matches!(
        error,
        StreamingError::UnsupportedPixelFormat { value } if value == "yuv420p"
    ));
}

#[test]
fn output_pixel_format_is_always_yuv420p() {
    let config = valid_config();
    let output_path = PathBuf::from("/opt/app/videos/demo.mp4");

    let arguments =
        build_arguments(&config, &output_path).expect("a valid configuration must build");

    let pix_fmt_positions: Vec<usize> = arguments
        .iter()
        .enumerate()
        .filter(|(_, argument)| argument.as_str() == "-pix_fmt")
        .map(|(index, _)| index)
        .collect();
    assert_eq!(pix_fmt_positions.len(), 2);
    assert_eq!(arguments[pix_fmt_positions[0] + 1], "rgb24");
    assert_eq!(arguments[pix_fmt_positions[1] + 1], "yuv420p");
}

#[test]
fn output_path_is_appended_last() {
    let config = valid_config();
    let output_path = PathBuf::from("/opt/app/videos/demo-20200913-122640_2.mp4");

    let arguments =
        build_arguments(&config, &output_path).expect("a valid configuration must build");

    assert_eq!(
        arguments.last().map(String::as_str),
        Some("/opt/app/videos/demo-20200913-122640_2.mp4")
    );
}

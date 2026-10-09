use crate::config::error::{ConfigError, ConfigIssue};
use crate::config::model::{AppConfig, RendererConfig, VideoConfig};

const REQUIRED_PIXEL_FORMAT: &str = "rgb24";
const SUPPORTED_PRIVACY_STATUSES: [&str; 3] = ["private", "public", "unlisted"];
const BLANK_VALUE_PROBLEM: &str = "must not be empty or whitespace-only";

pub(super) fn validate(config: &AppConfig) -> Result<(), ConfigError> {
    let mut issues = Vec::new();
    validate_video(config, &mut issues);
    validate_renderer(config, &mut issues);
    validate_required_strings(config, &mut issues);
    validate_privacy_status(config, &mut issues);
    if issues.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::Validation { issues })
    }
}

fn validate_video(config: &AppConfig, issues: &mut Vec<ConfigIssue>) {
    let video = &config.video;
    if video.width == 0 {
        issues.push(positive_value_issue("video.width", video.width));
    }
    if video.height == 0 {
        issues.push(positive_value_issue("video.height", video.height));
    }
    if video.fps == 0 {
        issues.push(positive_value_issue("video.fps", video.fps));
    }
    if video.pixel_format != REQUIRED_PIXEL_FORMAT {
        issues.push(ConfigIssue::new(
            "video.pixel_format".to_string(),
            format!(
                "unsupported pixel format '{}'; the MVP requires '{}'",
                video.pixel_format, REQUIRED_PIXEL_FORMAT
            ),
        ));
    }
}

fn validate_renderer(config: &AppConfig, issues: &mut Vec<ConfigIssue>) {
    let renderer = &config.renderer;
    if renderer.font_size == 0 {
        issues.push(positive_value_issue("renderer.font_size", renderer.font_size));
    }
    if renderer.line_height == 0 {
        issues.push(positive_value_issue(
            "renderer.line_height",
            renderer.line_height,
        ));
    }
    if is_unusable_vertical_layout(&config.video, renderer) {
        issues.push(vertical_layout_issue(&config.video, renderer));
    }
}

fn validate_required_strings(config: &AppConfig, issues: &mut Vec<ConfigIssue>) {
    let broadcast = &config.youtube.broadcast;
    let renderer = &config.renderer;
    let ffmpeg = &config.ffmpeg;
    push_blank_issue_if_needed("youtube.broadcast.title", &broadcast.title, issues);
    push_blank_issue_if_needed(
        "youtube.broadcast.description",
        &broadcast.description,
        issues,
    );
    push_blank_issue_if_needed("renderer.font", &renderer.font, issues);
    push_blank_issue_if_needed("renderer.text_color", &renderer.text_color, issues);
    push_blank_issue_if_needed(
        "renderer.background_color",
        &renderer.background_color,
        issues,
    );
    push_blank_issue_if_needed("chat.log_file", &config.chat.log_file, issues);
    push_blank_issue_if_needed("ffmpeg.executable", &ffmpeg.executable, issues);
    push_blank_issue_if_needed("ffmpeg.video_codec", &ffmpeg.video_codec, issues);
    push_blank_issue_if_needed("ffmpeg.preset", &ffmpeg.preset, issues);
    push_blank_issue_if_needed("ffmpeg.bitrate", &ffmpeg.bitrate, issues);
}

fn validate_privacy_status(config: &AppConfig, issues: &mut Vec<ConfigIssue>) {
    let privacy_status = &config.youtube.broadcast.privacy_status;
    if is_unsupported_privacy_status(privacy_status) {
        issues.push(ConfigIssue::new(
            "youtube.broadcast.privacy_status".to_string(),
            format!(
                "unsupported privacy status '{}'; expected one of: {}",
                privacy_status,
                SUPPORTED_PRIVACY_STATUSES.join(", ")
            ),
        ));
    }
}

fn positive_value_issue(field: &str, value: u32) -> ConfigIssue {
    ConfigIssue::new(field.to_string(), format!("must be positive (got {value})"))
}

fn is_unusable_vertical_layout(video: &VideoConfig, renderer: &RendererConfig) -> bool {
    // u64 arithmetic keeps u32 margin/line-height sums overflow-free.
    let occupied_height = u64::from(renderer.top_margin)
        + u64::from(renderer.bottom_margin)
        + u64::from(renderer.line_height);
    occupied_height > u64::from(video.height)
}

fn vertical_layout_issue(video: &VideoConfig, renderer: &RendererConfig) -> ConfigIssue {
    ConfigIssue::new(
        "renderer.vertical_layout".to_string(),
        format!(
            "vertical layout leaves no room for at least one text line (height {}, top_margin {}, bottom_margin {}, line_height {})",
            video.height, renderer.top_margin, renderer.bottom_margin, renderer.line_height
        ),
    )
}

fn push_blank_issue_if_needed(field: &str, value: &str, issues: &mut Vec<ConfigIssue>) {
    if is_blank(value) {
        issues.push(ConfigIssue::new(field.to_string(), BLANK_VALUE_PROBLEM.to_string()));
    }
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn is_unsupported_privacy_status(value: &str) -> bool {
    !SUPPORTED_PRIVACY_STATUSES.contains(&value)
}

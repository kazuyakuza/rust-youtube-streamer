//! The pure config→CDN mapping for the O1 insert body (D-T1-6).
//!
//! Grounding: `cdn.ingestionType` valid values are `dash|hls|rtmp` with
//! "rtmp (which includes RTMPS)"; the `cdn.resolution` label set is
//! `240p|360p|480p|720p|1080p|1440p|2160p|variable` and `cdn.frameRate`
//! `variable|30fps|60fps`, immutable after insert, per
//! https://developers.google.com/youtube/v3/live/docs/liveStreams.
//!
//! Staging: consumed now — Task 2's orchestrator reads the `resolution` /
//! `frame_rate` accessors and the transport serializes the O1 insert-body
//! `cdn` block through them. They live inside the `live_models` module,
//! which keeps a whole-module staging allow that retires when Task 3's
//! `run` wiring roots the chain.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.3, D-T1-6, F1); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 1.

use serde::Serialize;

use crate::config::model::VideoConfig;

/// Locked industry-standard tuples behind the documented label values;
/// any tuple outside the table maps to the documented `variable` label.
const RESOLUTION_TABLE: [(u32, u32, &str); 7] = [
    (426, 240, "240p"),
    (640, 360, "360p"),
    (640, 480, "480p"),
    (1280, 720, "720p"),
    (1920, 1080, "1080p"),
    (2560, 1440, "1440p"),
    (3840, 2160, "2160p"),
];
const VARIABLE_LABEL: &str = "variable";
const THIRTY_FPS: u32 = 30;
const SIXTY_FPS: u32 = 60;
const THIRTY_FPS_LABEL: &str = "30fps";
const SIXTY_FPS_LABEL: &str = "60fps";

/// The `cdn` block of the O1 body (D-T1-6): ingestion type is fixed
/// `"rtmp"` (officially the value that also covers RTMPS); any off-table
/// resolution or frame rate forces BOTH labels to `variable` (official
/// pairing rule: variable resolution requires variable frame rate).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamCreationCdn {
    pub(super) ingestion_type: String,
    pub(super) resolution: String,
    pub(super) frame_rate: String,
}

impl StreamCreationCdn {
    pub(crate) const RTMP_INGESTION_TYPE: &'static str = "rtmp";

    pub(crate) fn resolution(&self) -> &str {
        &self.resolution
    }

    pub(crate) fn frame_rate(&self) -> &str {
        &self.frame_rate
    }

    /// Pure config→CDN mapping over the typed `VideoConfig` (one
    /// argument — max-2-params rule).
    pub(crate) fn from_video_config(video: &VideoConfig) -> Self {
        let resolution = Self::resolution_label(video.width, video.height);
        let frame_rate = Self::frame_rate_label(video.fps);
        Self::from_labels(&resolution, &frame_rate)
    }

    pub(super) fn from_labels(resolution: &str, frame_rate: &str) -> Self {
        let off_table = resolution == VARIABLE_LABEL || frame_rate == VARIABLE_LABEL;
        let (resolution, frame_rate) = if off_table {
            (VARIABLE_LABEL, VARIABLE_LABEL)
        } else {
            (resolution, frame_rate)
        };
        Self {
            ingestion_type: Self::RTMP_INGESTION_TYPE.to_string(),
            resolution: resolution.to_string(),
            frame_rate: frame_rate.to_string(),
        }
    }

    fn resolution_label(width: u32, height: u32) -> String {
        RESOLUTION_TABLE
            .iter()
            .find(|(table_width, table_height, _)| *table_width == width && *table_height == height)
            .map(|(_, _, label)| (*label).to_string())
            .unwrap_or_else(|| VARIABLE_LABEL.to_string())
    }

    fn frame_rate_label(fps: u32) -> String {
        match fps {
            THIRTY_FPS => THIRTY_FPS_LABEL.to_string(),
            SIXTY_FPS => SIXTY_FPS_LABEL.to_string(),
            _ => VARIABLE_LABEL.to_string(),
        }
    }
}

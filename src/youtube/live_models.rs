//! Typed YouTube Live API resource models (plan F1, D-T1-2): serde views
//! of the documented `liveBroadcasts`/`liveStreams` response fields plus
//! the typed insert-request bodies. Responses IGNORE unknown fields (the
//! deliberate opposite of the config module's `deny_unknown_fields` —
//! Google adds resource fields freely); missing parts stay `None` here
//! and surface as typed failures at the consumer boundary (Task 2).
//! Request bodies lock the exact documented JSON shapes offline before
//! any transport exists. No chat/renderer/streaming dependency; the only
//! config dependency is `VideoConfig` for the pure CDN mapping (D-T1-6).

use serde::{Deserialize, Serialize};

use crate::config::model::VideoConfig;

/// A `liveBroadcasts.list` page (O5); `items` defaults to empty.
#[derive(Deserialize)]
pub(crate) struct LiveBroadcastList {
    #[serde(default)]
    items: Vec<LiveBroadcast>,
}

/// One broadcast resource view (O2/O3/O5); documented fields only.
#[derive(Deserialize)]
pub(crate) struct LiveBroadcast {
    id: Option<String>,
    snippet: Option<BroadcastSnippet>,
    status: Option<BroadcastStatus>,
    #[serde(rename = "contentDetails")]
    content_details: Option<BroadcastContentDetails>,
}

#[derive(Deserialize)]
pub(crate) struct BroadcastSnippet {
    #[serde(rename = "scheduledStartTime")]
    scheduled_start_time: Option<String>,
    title: Option<String>,
    description: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BroadcastStatus {
    #[serde(rename = "lifeCycleStatus")]
    life_cycle_status: Option<String>,
    #[serde(rename = "privacyStatus")]
    privacy_status: Option<String>,
    #[serde(rename = "recordingStatus")]
    recording_status: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct BroadcastContentDetails {
    #[serde(rename = "boundStreamId")]
    bound_stream_id: Option<String>,
}

/// A `liveStreams.list` page (O4); `items` defaults to empty.
#[derive(Deserialize)]
pub(crate) struct LiveStreamList {
    #[serde(default)]
    items: Vec<LiveStream>,
}

/// One live-stream resource view (O1/O4); documented fields only.
#[derive(Deserialize)]
pub(crate) struct LiveStream {
    id: Option<String>,
    status: Option<StreamStatus>,
    cdn: Option<StreamCdn>,
}

#[derive(Deserialize)]
pub(crate) struct StreamCdn {
    #[serde(rename = "ingestionType")]
    ingestion_type: Option<String>,
    #[serde(rename = "ingestionInfo")]
    ingestion_info: Option<IngestionInfo>,
}

/// The documented ingestion-address family (never real keys in fixtures).
#[derive(Deserialize)]
pub(crate) struct IngestionInfo {
    #[serde(rename = "streamName")]
    stream_name: Option<String>,
    #[serde(rename = "ingestionAddress")]
    ingestion_address: Option<String>,
    #[serde(rename = "backupIngestionAddress")]
    backup_ingestion_address: Option<String>,
    #[serde(rename = "rtmpsIngestionAddress")]
    rtmps_ingestion_address: Option<String>,
    #[serde(rename = "rtmpsBackupIngestionAddress")]
    rtmps_backup_ingestion_address: Option<String>,
}

#[derive(Deserialize)]
pub(crate) struct StreamStatus {
    #[serde(rename = "streamStatus")]
    stream_status: Option<String>,
    #[serde(rename = "healthStatus")]
    health_status: Option<StreamHealth>,
}

#[derive(Deserialize)]
pub(crate) struct StreamHealth {
    #[serde(rename = "status")]
    overall: Option<String>,
}

// Request side (serialize; camelCase renames — transport-agnostic command types).

/// O1 (`liveStreams.insert`) command: validated title + CDN labels.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateStreamCommand {
    pub(crate) title: String,
    pub(crate) resolution: String,
    pub(crate) frame_rate: String,
}

/// O2 (`liveBroadcasts.insert`) command: validated broadcast metadata.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateBroadcastCommand {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) privacy_status: String,
    pub(crate) scheduled_start_time: String,
}

/// O3 (`liveBroadcasts.bind`) parameters: query-string pairs, no body
/// (official: "Do not provide a request body when calling this method").
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BindCommand {
    pub(crate) broadcast_id: String,
    pub(crate) stream_id: String,
}

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
/// `"rtmp"` (officially the value that also covers RTMPS); resolution
/// and frame rate map from validated `config.video`, and any off-table
/// value forces BOTH labels to `variable` (official pairing rule:
/// variable resolution requires variable frame rate).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamCreationCdn {
    ingestion_type: String,
    resolution: String,
    frame_rate: String,
}

impl StreamCreationCdn {
    pub(crate) const RTMP_INGESTION_TYPE: &'static str = "rtmp";

    /// Pure config→CDN mapping over the typed `VideoConfig` (one
    /// argument — max-2-params rule).
    pub(crate) fn from_video_config(video: &VideoConfig) -> Self {
        let resolution = Self::resolution_label(video.width, video.height);
        let frame_rate = Self::frame_rate_label(video.fps);
        Self::from_labels(&resolution, &frame_rate)
    }

    fn from_labels(resolution: &str, frame_rate: &str) -> Self {
        let off_table = resolution == VARIABLE_LABEL || frame_rate == VARIABLE_LABEL;
        let resolution = if off_table {
            VARIABLE_LABEL
        } else {
            resolution
        };
        let frame_rate = if off_table {
            VARIABLE_LABEL
        } else {
            frame_rate
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

/// The exact O2 insert body (D-T1-7): autoStart/autoStop enabled, the
/// monitor stream disabled, required snippet + privacy status.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BroadcastRequestBody {
    snippet: BroadcastCreationSnippet,
    content_details: BroadcastCreationDetails,
    status: BroadcastCreationStatus,
}

impl BroadcastRequestBody {
    pub(crate) fn from_command(command: &CreateBroadcastCommand) -> Self {
        Self {
            snippet: BroadcastCreationSnippet {
                title: command.title.clone(),
                description: command.description.clone(),
                scheduled_start_time: command.scheduled_start_time.clone(),
            },
            content_details: BroadcastCreationDetails::auto_started(),
            status: BroadcastCreationStatus {
                privacy_status: command.privacy_status.clone(),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BroadcastCreationSnippet {
    title: String,
    description: String,
    scheduled_start_time: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BroadcastCreationDetails {
    enable_auto_start: bool,
    enable_auto_stop: bool,
    monitor_stream: MonitorStream,
}

impl BroadcastCreationDetails {
    /// D-T1-7: autoStart + autoStop true, monitor stream false — the
    /// runtime never calls `liveBroadcasts.transition`.
    fn auto_started() -> Self {
        Self {
            enable_auto_start: true,
            enable_auto_stop: true,
            monitor_stream: MonitorStream {
                enable_monitor_stream: false,
            },
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MonitorStream {
    enable_monitor_stream: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BroadcastCreationStatus {
    privacy_status: String,
}

/// The exact O1 insert body: the stream snippet title + mapped CDN.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StreamRequestBody {
    snippet: StreamCreationSnippet,
    cdn: StreamCreationCdn,
}

impl StreamRequestBody {
    pub(crate) fn from_command(command: &CreateStreamCommand) -> Self {
        Self {
            snippet: StreamCreationSnippet {
                title: command.title.clone(),
            },
            cdn: StreamCreationCdn::from_labels(&command.resolution, &command.frame_rate),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StreamCreationSnippet {
    title: String,
}

#[cfg(test)]
#[path = "live_models_tests.rs"]
mod live_models_tests;

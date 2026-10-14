//! The exact typed insert-request bodies (D-T1-7): the JSON shapes the
//! O1/O2 operations document, locked offline before any transport exists.

use serde::Serialize;

use super::commands::{CreateBroadcastCommand, CreateStreamCommand};
use super::stream_cdn::StreamCreationCdn;

/// The exact O2 insert body (D-T1-7): autoStart/autoStop enabled, the
/// monitor stream disabled, required snippet + privacy status — the
/// runtime never calls `liveBroadcasts.transition`.
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
            content_details: BroadcastCreationDetails {
                enable_auto_start: true,
                enable_auto_stop: true,
                monitor_stream: MonitorStream {
                    enable_monitor_stream: false,
                },
            },
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

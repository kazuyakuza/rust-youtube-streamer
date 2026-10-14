//! The exact typed insert-request bodies (D-T1-7): the JSON shapes the
//! O1/O2 operations document, locked offline before any transport exists.
//!
//! Grounding: `snippet.{title,description,scheduledStartTime}` and
//! `status.privacyStatus` are the insert fields of
//! https://developers.google.com/youtube/v3/live/docs/liveBroadcasts;
//! the `contentDetails.enableAutoStart`/`enableAutoStop` +
//! `monitorStream.enableMonitorStream=false` shape follows the
//! no-testing-stage guidance of
//! https://developers.google.com/youtube/v3/live/life-of-a-broadcast
//! ("consider setting enableAutoStart and enableAutoStop to true"), which
//! removes both manual `liveBroadcasts.transition` calls; the O1 body
//! fields are those of
//! https://developers.google.com/youtube/v3/live/docs/liveStreams.
//!
//! Staging: consumed now — Task 2's concrete transport serializes these as
//! the O1/O2 request bodies (via `api_request`). They live inside the
//! `live_models` module, which keeps a whole-module staging allow that
//! retires when Task 3's `run` wiring roots the chain.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.1/§1.3, D-T1-7, F1); TODO `.agent/todos/20261013/20261013-todo-1.md`
//! Task 1.

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

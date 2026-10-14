//! The broadcast-and-ingestion orchestrator (plan F-T2-2; D-T2-8; the
//! TODO Task-2 bullet "create a live stream and a broadcast from the
//! validated config, bind them, inspect status until ready"): one entry
//! type [`LiveBroadcastPreparer`] consumed by Task 3. `from_config`
//! builds both typed commands from the ALREADY-validated `AppConfig`
//! (D-T2-13: no local re-validation — the config loader is the single
//! funnel; the stream title reuses the validated broadcast title, D-T1-6)
//! and pins `scheduledStartTime` to `now + 60 s` (D-T1-5). `prepare`
//! runs D-T1-8 steps 3–6 exactly once each — O1 insert stream, O2 insert
//! broadcast, O3 bind (asserting `boundStreamId`), the bounded readiness
//! wait, then the destination extraction — with NO request duplication
//! around insert/bind (brief §12.2's duplicate-broadcast risk).
//! Redaction contract: `PreparedBroadcast` has no `Display`; its derived
//! `Debug` is secret-safe because the only secret-typed field is the
//! redacting `RtmpsIngestion` wrapper (D-T2-12).
//!
//! Plan: `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
//! (F-T2-2, D-T2-8, D-T2-13); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 2.

use crate::config::model::AppConfig;

use super::api_transport::LiveApiTransport;
use super::live_error::LiveApiError;
use super::live_models::{
    BindCommand, CreateBroadcastCommand, CreateStreamCommand, LiveBroadcast, LiveStream,
    StreamCreationCdn,
};
use super::readiness_wait::ReadinessWait;
use super::rtmps_ingestion::RtmpsIngestion;
use super::scheduled_start_time::scheduled_start_time_from_epoch;
use super::wait_cancellation::WaitCanceller;

/// The Task-3-consumed orchestrator: validated config → commands.
pub(crate) struct LiveBroadcastPreparer {
    stream_command: CreateStreamCommand,
    broadcast_command: CreateBroadcastCommand,
}

/// The prepared result: the non-secret operator-diagnostics ids and the
/// redacting destination wrapper (D-T2-12's one reveal point is
/// `RtmpsIngestion::into_url`, Task-3 app glue only). The derived
/// `Debug` is secret-safe: the only secret-typed field redacts itself.
#[derive(Debug)]
pub(crate) struct PreparedBroadcast {
    broadcast_id: String,
    stream_id: String,
    ingestion: RtmpsIngestion,
}

impl PreparedBroadcast {
    pub(crate) fn broadcast_id(&self) -> &str {
        &self.broadcast_id
    }

    /// The bound stream id, consumed by Task 3's post-start O4
    /// confirmation wait.
    pub(crate) fn stream_id(&self) -> &str {
        &self.stream_id
    }

    pub(crate) fn into_ingestion(self) -> RtmpsIngestion {
        self.ingestion
    }
}

impl LiveBroadcastPreparer {
    /// Two args by design (max-2 rule): config + the injected clock
    /// moment; the clock never enters `youtube` from a global.
    pub(crate) fn from_config(config: &AppConfig, now_epoch_secs: u64) -> Self {
        let broadcast = &config.youtube.broadcast;
        let cdn = StreamCreationCdn::from_video_config(&config.video);
        Self {
            stream_command: CreateStreamCommand {
                title: broadcast.title.clone(),
                resolution: cdn.resolution().to_string(),
                frame_rate: cdn.frame_rate().to_string(),
            },
            broadcast_command: CreateBroadcastCommand {
                title: broadcast.title.clone(),
                description: broadcast.description.clone(),
                privacy_status: broadcast.privacy_status.clone(),
                scheduled_start_time: scheduled_start_time_from_epoch(now_epoch_secs),
            },
        }
    }

    /// D-T1-8 steps 3–6, each exactly once, failures typed and immediate.
    pub(crate) fn prepare(
        &self,
        transport: &dyn LiveApiTransport,
        cancellation: &WaitCanceller,
    ) -> Result<PreparedBroadcast, LiveApiError> {
        let stream = transport.create_stream(&self.stream_command)?;
        let stream_id = inserted_stream_id(&stream)?;
        let broadcast = transport.create_broadcast(&self.broadcast_command)?;
        let broadcast_id = inserted_broadcast_id(&broadcast)?;
        let bound = transport.bind(&BindCommand {
            broadcast_id: broadcast_id.clone(),
            stream_id: stream_id.clone(),
        })?;
        bound_stream_asserted(&bound, &stream_id)?;
        let confirmed =
            ReadinessWait::new(transport, &stream_id, &broadcast_id, cancellation).run()?;
        let ingestion = confirmed_ingestion(&confirmed)?;
        Ok(PreparedBroadcast {
            broadcast_id,
            stream_id,
            ingestion,
        })
    }
}

fn inserted_stream_id(stream: &LiveStream) -> Result<String, LiveApiError> {
    stream.id.clone().ok_or_else(|| malformed("id"))
}

fn inserted_broadcast_id(broadcast: &LiveBroadcast) -> Result<String, LiveApiError> {
    broadcast.id.clone().ok_or_else(|| malformed("id"))
}

/// D-T2-8 step 3: the bind reply must carry the expected bound stream.
fn bound_stream_asserted(bound: &LiveBroadcast, stream_id: &str) -> Result<(), LiveApiError> {
    let bound_stream_id = bound
        .content_details
        .as_ref()
        .and_then(|details| details.bound_stream_id.as_deref())
        .ok_or_else(|| malformed("contentDetails.boundStreamId"))?;
    if bound_stream_id != stream_id {
        return Err(broadcast_state("boundStreamId mismatch"));
    }
    Ok(())
}

fn confirmed_ingestion(confirmed: &LiveStream) -> Result<RtmpsIngestion, LiveApiError> {
    RtmpsIngestion::from_stream(confirmed)
}

fn malformed(field: &str) -> LiveApiError {
    LiveApiError::MalformedResponse {
        field: field.to_string(),
    }
}

fn broadcast_state(detail: &str) -> LiveApiError {
    LiveApiError::BroadcastState {
        detail: detail.to_string(),
    }
}

#[cfg(test)]
#[path = "live_ingestion_tests.rs"]
mod live_ingestion_tests;

#[cfg(test)]
#[path = "live_ingestion_failure_tests.rs"]
mod live_ingestion_failure_tests;

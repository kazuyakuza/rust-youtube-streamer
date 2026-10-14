//! The broadcast-resource stage of the run runtime (plan D-T3-2 stage
//! 5): ONE preparer call runs O1 insert stream → O2 insert broadcast →
//! O3 bind → the bounded readiness wait → the destination extraction,
//! each step exactly once (no request duplication, brief §12.2). The
//! transport/cancellation collaborators arrive as the `StageRuntime`
//! parameter object (≤2-args rule).

use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::AppConfig;
use crate::youtube::{
    LiveApiError, LiveApiTransport, LiveBroadcastPreparer, PreparedBroadcast, WaitCanceller,
};

/// One preparation stage's injected collaborators.
pub(super) struct StageRuntime<'a> {
    pub(super) transport: &'a (dyn LiveApiTransport + Send + Sync),
    pub(super) cancellation: &'a WaitCanceller,
}

pub(super) fn prepare_broadcast(
    app_config: &AppConfig,
    stage: &StageRuntime<'_>,
) -> Result<PreparedBroadcast, LiveApiError> {
    let preparer = LiveBroadcastPreparer::from_config(app_config, epoch_seconds_now());
    preparer.prepare(stage.transport, stage.cancellation)
}

/// The clock read for the `scheduledStartTime` derivation (D-T1-5). A
/// pre-epoch (or failed) read falls back to 0, which the server rejects
/// as a typed `invalidScheduledStartTime` → `InvalidMetadata` —
/// actionable, never silent.
fn epoch_seconds_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.as_secs())
}

//! The app's SINGLE Ctrl+C handler install (plan D-T3-2 stage 1; split from
//! `run/mod.rs` per the line-cap precedent): it flips BOTH the frame-loop
//! `StopFlag` and a `WaitCanceller` clone, so no youtube type enters the
//! `pipeline` module.

use crate::app::pipeline::{install_stop_request_handler, StopFlag};
use crate::youtube::WaitCanceller;

use super::error::RunError;
use super::seams::RunStops;

pub(super) fn install_stop_pair() -> Result<RunStops, RunError> {
    let stops = RunStops {
        stop_flag: StopFlag::new(),
        canceller: WaitCanceller::new(),
    };
    let handler_flag = stops.stop_flag.clone();
    let handler_canceller = stops.canceller.clone();
    install_stop_request_handler(Box::new(move || {
        handler_flag.request_stop();
        handler_canceller.request_cancel();
    }))
    .map_err(RunError::Interrupt)?;
    Ok(stops)
}

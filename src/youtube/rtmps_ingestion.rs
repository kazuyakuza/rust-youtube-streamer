//! RTMPS destination extraction (plan F-T2-5, D-T2-12; D-T1-11 locked):
//! the youtube-side redacting wrapper around the assembled destination —
//! EXACTLY `rtmpsIngestionAddress/streamName`, the documented
//! concatenation form (no scheme validation, no port insertion, no other
//! URL surgery) — and NEVER an undocumented fallback to the plain
//! `ingestionAddress` (rewriting an rtmp:// scheme would be undocumented
//! surgery). `Debug` prints only the redacted marker, there is no
//! `Display` ever, and `into_url` is the ONE reveal point that crosses
//! into Task 3's app glue
//! (`RtmpsDestination::new(prepared_ingestion.into_url())`); the
//! streaming-side `RtmpsDestination` wrapper stays Task 3's file — two
//! wrappers exist because neither module may depend on the other.
//!
//! Grounding: https://developers.google.com/youtube/v3/live/guides/rtmps-ingestion
//! (destination is `rtmpsIngestionAddress`/`streamName`).
//! Plan: `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
//! (F-T2-5, D-T2-12); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 2.

use std::fmt;

use super::live_error::LiveApiError;
use super::live_models::LiveStream;

const CDN_FIELD: &str = "cdn";
const INGESTION_INFO_FIELD: &str = "cdn.ingestionInfo";
const RTMPS_ADDRESS_FIELD: &str = "cdn.ingestionInfo.rtmpsIngestionAddress";
const STREAM_NAME_FIELD: &str = "cdn.ingestionInfo.streamName";

/// The assembled RTMPS destination; the url is private and unformattable.
pub(crate) struct RtmpsIngestion {
    url: String,
}

impl RtmpsIngestion {
    /// Chain extraction (cdn → ingestionInfo → the two required fields →
    /// the single concatenation); a missing field is `DestinationMissing`
    /// with the exact API resource-path spelling.
    pub(crate) fn from_stream(stream: &LiveStream) -> Result<Self, LiveApiError> {
        let cdn = stream.cdn.as_ref().ok_or_else(|| missing(CDN_FIELD))?;
        let info = cdn
            .ingestion_info
            .as_ref()
            .ok_or_else(|| missing(INGESTION_INFO_FIELD))?;
        let address = info
            .rtmps_ingestion_address
            .as_deref()
            .ok_or_else(|| missing(RTMPS_ADDRESS_FIELD))?;
        let name = info
            .stream_name
            .as_deref()
            .ok_or_else(|| missing(STREAM_NAME_FIELD))?;
        Ok(Self {
            url: format!("{address}/{name}"),
        })
    }

    /// The ONE reveal point. Retire the allow when Task 3's app glue
    /// consumes it (`RtmpsDestination::new(prepared_ingestion.into_url())`).
    #[allow(dead_code)]
    pub(crate) fn into_url(self) -> String {
        self.url
    }
}

impl fmt::Debug for RtmpsIngestion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RtmpsIngestion(<redacted>)")
    }
}

fn missing(field: &str) -> LiveApiError {
    LiveApiError::DestinationMissing {
        field: field.to_string(),
    }
}

#[cfg(test)]
#[path = "rtmps_ingestion_tests.rs"]
mod rtmps_ingestion_tests;

//! The streaming-owned secret wrapper for the production RTMPS destination
//! (plan D-T3-5 F-S1, D-T1-10 (i)): the assembled ingestion URL crosses the
//! app layer opaque and is revealed only to the streaming-internal argument
//! builder and spawn path. Same shape as the youtube-side `RtmpsIngestion`
//! wrapper; two wrappers exist because neither module may depend on the
//! other. `Debug` prints only a fixed redaction marker, there is no
//! `Display` ever, and `url` is the only secret-typed field.

use std::fmt;

/// The RTMPS destination FFmpeg streams toward; the url is private and
/// unformattable.
pub(crate) struct RtmpsDestination {
    url: String,
}

impl RtmpsDestination {
    /// Wraps the assembled destination URL at the production reveal point.
    /// Retire the allow when the Task-3 run wiring constructs it.
    #[allow(dead_code)]
    pub(crate) fn new(url: String) -> Self {
        Self { url }
    }

    /// The assembled URL, consumed ONLY by the `args`/`ffmpeg` siblings.
    pub(super) fn as_url(&self) -> &str {
        &self.url
    }
}

impl fmt::Debug for RtmpsDestination {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RtmpsDestination(<redacted>)")
    }
}

#[cfg(test)]
#[path = "rtmps_destination_tests.rs"]
mod rtmps_destination_tests;

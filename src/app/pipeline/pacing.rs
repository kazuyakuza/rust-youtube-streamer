//! Frame pacing: validated-FPS interval derivation, the pure anchored
//! incremental [`PacingSchedule`] that holds the loop to the configured FPS
//! without unbounded drift, and the setup-time [`PipelinePacing`] parameter
//! object (frame interval plus the exact RGB24 byte count). Anchored
//! incremental drift model: the target advances one interval per emitted
//! frame and is floored to "now" whenever the loop falls a whole interval
//! behind, so catch-up is bounded to one immediate frame and the loop can
//! never burst or accumulate drift (TODO Task 2; plan decision S6).

use crate::app::pipeline::error::FramePipelineError;
use crate::config::model::VideoConfig;

pub(crate) const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// Frame interval for a validated config FPS: fixed-point truncation to whole
/// nanoseconds. `fps == 0` cannot come from the validated config loader; the
/// guard keeps the helper total (no panics) and degrades to one frame per
/// second instead of dividing by zero.
pub(crate) fn interval_nanos(fps: u32) -> u64 {
    if fps == 0 {
        return NANOS_PER_SECOND;
    }
    NANOS_PER_SECOND / u64::from(fps)
}

/// Anchored incremental schedule (pure; unit-testable without a clock).
pub(crate) struct PacingSchedule {
    interval_nanos: u64,
    target_nanos: u64,
}

impl PacingSchedule {
    /// Anchors the schedule: the first frame is emitted immediately, the
    /// first sleep target sits one interval ahead of `start_nanos`.
    pub(crate) fn started(interval_nanos: u64, start_nanos: u64) -> Self {
        Self {
            interval_nanos,
            target_nanos: start_nanos,
        }
    }

    /// Called once AFTER each emitted frame. Advances the target by one
    /// interval, floors it to `now_nanos` when the loop is behind (dropping
    /// the whole backlog), and returns the gap the clock must sleep: at
    /// most one immediate catch-up frame per wake, no burst, no drift
    /// accumulation. Wrap-around-proof via `saturating_add` (u64 ns ≈ 584 y).
    pub(crate) fn next_gap(&mut self, now_nanos: u64) -> u64 {
        self.target_nanos = self.target_nanos.saturating_add(self.interval_nanos);
        if self.target_nanos <= now_nanos {
            self.target_nanos = now_nanos;
        }
        self.target_nanos - now_nanos
    }
}

/// Derived setup parameters for one pipeline: the frame interval from the
/// validated FPS and the expected RGB24 byte count from the validated video
/// geometry.
pub(crate) struct PipelinePacing {
    interval_nanos: u64,
    frame_byte_count: usize,
}

impl PipelinePacing {
    /// Derives the frame interval from the validated FPS and the expected
    /// RGB24 byte count `width * height * 3` from validated video geometry.
    /// Overflow (checked u32 math) is a typed pipeline error; the config
    /// validator makes it unreachable through `load_from_path`.
    pub(crate) fn from_config(video: &VideoConfig) -> Result<Self, FramePipelineError> {
        let width = video.width;
        let height = video.height;
        let byte_count = rgb24_byte_count(width, height)
            .ok_or(FramePipelineError::FrameByteOverrun { width, height })?;
        Ok(Self::raw(interval_nanos(video.fps), byte_count))
    }

    /// Direct constructor for tests and thin pipelines: no validation.
    pub(crate) fn raw(interval_nanos: u64, frame_byte_count: usize) -> Self {
        Self {
            interval_nanos,
            frame_byte_count,
        }
    }

    pub(crate) fn interval_nanos(&self) -> u64 {
        self.interval_nanos
    }

    pub(crate) fn frame_byte_count(&self) -> usize {
        self.frame_byte_count
    }
}

/// Checked `width * height * 3` for validated u32 geometry.
fn rgb24_byte_count(width: u32, height: u32) -> Option<usize> {
    usize::checked_mul(width as usize, height as usize).and_then(|pixels| pixels.checked_mul(3))
}

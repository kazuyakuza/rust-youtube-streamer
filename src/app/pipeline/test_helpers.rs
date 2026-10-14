use std::cell::{Cell, RefCell};
use std::io;
use std::path::PathBuf;
use std::rc::Rc;

use crate::app::pipeline::clock::FrameClock;
use crate::app::pipeline::interrupt::StopFlag;
use crate::renderer::{Frame, RenderError, RenderInput, Renderer, Rgb};
use crate::streaming::{CompletedOutput, CompletedVideo, FrameSink, StreamingError};

pub(crate) const TEST_INTERVAL_NANOS: u64 = 40;
pub(crate) const TEST_FRAME_BYTES: usize = 12; // 2 * 2 * 3

/// Shared per-run sink recordings: writes, finish call, output path.
#[derive(Default)]
pub(crate) struct SinkEvents {
    pub(crate) writes: Vec<Vec<u8>>,
    pub(crate) finish_called: bool,
    pub(crate) finish_path: Option<PathBuf>,
}

/// Deterministic clock: elapsed advances per recorded sleep.
pub(crate) struct FakeClock {
    elapsed: Cell<u64>,
    sleeps: Rc<RefCell<Vec<u64>>>,
}

impl FakeClock {
    pub(crate) fn fresh() -> (Self, Rc<RefCell<Vec<u64>>>) {
        let sleeps = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                elapsed: Cell::new(0),
                sleeps: Rc::clone(&sleeps),
            },
            sleeps,
        )
    }
}

impl FrameClock for FakeClock {
    fn now_nanos(&self) -> u64 {
        self.elapsed.get()
    }

    fn sleep_nanos(&self, nanos: u64) {
        self.sleeps.borrow_mut().push(nanos);
        self.elapsed.set(self.elapsed.get() + nanos);
    }
}

/// Renderer stub: records input text; optional one-shot failure.
pub(crate) struct StubRenderer {
    inputs: Rc<RefCell<Vec<String>>>,
    failure: Option<RenderError>,
}

impl StubRenderer {
    pub(crate) fn fresh() -> (Self, Rc<RefCell<Vec<String>>>) {
        let inputs = Rc::new(RefCell::new(Vec::new()));
        (
            Self {
                inputs: Rc::clone(&inputs),
                failure: None,
            },
            inputs,
        )
    }

    pub(crate) fn failing(failure: RenderError) -> Self {
        Self {
            inputs: Rc::new(RefCell::new(Vec::new())),
            failure: Some(failure),
        }
    }
}

impl Renderer for StubRenderer {
    fn render(&mut self, input: &RenderInput) -> Result<Frame, RenderError> {
        if let Some(failure) = self.failure.take() {
            return Err(failure);
        }
        self.inputs.borrow_mut().push(input.text.clone());
        Frame::filled(2, 2, Rgb::black())
    }
}

/// Recording sink: logs frames; stops the loop after N writes; fakes finish.
pub(crate) struct RecordingSink {
    events: Rc<RefCell<SinkEvents>>,
    output_path: PathBuf,
    stop_after_writes: Option<(usize, StopFlag)>,
}

impl RecordingSink {
    pub(crate) fn fresh() -> (Self, Rc<RefCell<SinkEvents>>) {
        let events = Rc::new(RefCell::new(SinkEvents::default()));
        let sink = Self {
            events: Rc::clone(&events),
            output_path: PathBuf::from("videos/test-recording.mp4"),
            stop_after_writes: None,
        };
        (sink, events)
    }

    pub(crate) fn stop_after_writes(mut self, threshold: usize, stop_flag: StopFlag) -> Self {
        self.stop_after_writes = Some((threshold, stop_flag));
        self
    }

    fn request_stop_when_due(&self) {
        if let Some((threshold, flag)) = &self.stop_after_writes {
            let write_count = self.events.borrow().writes.len();
            if write_count == *threshold {
                flag.request_stop();
            }
        }
    }
}

impl FrameSink for RecordingSink {
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), StreamingError> {
        self.events.borrow_mut().writes.push(frame.to_vec());
        self.request_stop_when_due();
        Ok(())
    }

    fn finish(self) -> Result<CompletedOutput, StreamingError> {
        let output_path = self.output_path.clone();
        let mut log = self.events.borrow_mut();
        log.finish_called = true;
        log.finish_path = Some(output_path.clone());
        Ok(CompletedOutput::Recording(CompletedVideo::new(output_path)))
    }
}

/// Sink fake failing the first write; shares `SinkEvents` observability.
pub(crate) struct FailingWriteSink {
    events: Rc<RefCell<SinkEvents>>,
}

impl FailingWriteSink {
    pub(crate) fn fresh() -> (Self, Rc<RefCell<SinkEvents>>) {
        let events = Rc::new(RefCell::new(SinkEvents::default()));
        (
            Self {
                events: Rc::clone(&events),
            },
            events,
        )
    }
}

impl FrameSink for FailingWriteSink {
    fn write_frame(&mut self, frame: &[u8]) -> Result<(), StreamingError> {
        self.events.borrow_mut().writes.push(frame.to_vec());
        Err(StreamingError::FrameWrite {
            source: io::Error::new(io::ErrorKind::BrokenPipe, "pipe closed by the child"),
        })
    }

    fn finish(self) -> Result<CompletedOutput, StreamingError> {
        self.events.borrow_mut().finish_called = true;
        Ok(CompletedOutput::Recording(CompletedVideo::new(
            PathBuf::from("videos/not-reached.mp4"),
        )))
    }
}

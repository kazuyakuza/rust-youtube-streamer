//! Application-level frame pipeline: wires chat → renderer → streaming at
//! the application layer (brief §2.2/§4), the one place where those three
//! modules meet. Boundary rules: `pipeline` may use `crate::chat`,
//! `crate::renderer`, `crate::streaming`, `crate::config::model` and `std`;
//! `streaming` never learns about chat messages or rendering. Pacing is a
//! single synchronous std thread (no async runtime; brief §3.2), Ctrl+C is a
//! shared stop flag documented in `interrupt`, and scope authority is TODO
//! Task 2 in `.agent/todos/20261011/20261011-todo-1-DONE.md` plus plan
//! `.kilo/plans/20261011-phase03-task2-frame-pipeline.md`. Reachable from
//! `main` through the `demo` mode (TODO Task 3).
mod clock;
mod error;
mod interrupt;
mod pacing;
mod runner;
mod source;

pub(crate) use error::FramePipelineError;
pub(crate) use interrupt::{
    install_interrupt_handler, install_stop_request_handler, InterruptHandlerError, StopFlag,
};
pub(crate) use runner::{
    prepared_pipeline, prepared_stream_pipeline, PipelineConstruction, StreamParts,
};
pub(crate) use source::MessageSource;

#[cfg(test)]
pub(crate) use pacing::PipelinePacing;
#[cfg(test)]
pub(crate) use runner::{FramePipeline, PipelineParts};
#[cfg(test)]
pub(crate) use test_helpers::{
    FakeClock, RecordingSink, StubRenderer, TEST_FRAME_BYTES, TEST_INTERVAL_NANOS,
};

#[cfg(test)]
mod clock_tests;
#[cfg(test)]
mod error_tests;
#[cfg(test)]
mod interrupt_tests;
#[cfg(test)]
mod pacing_tests;
#[cfg(test)]
mod runner_tests;
#[cfg(test)]
mod test_helpers;

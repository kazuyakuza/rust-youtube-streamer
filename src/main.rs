//! `rust-youtube-streamer-service` process entry point. Delegates to the
//! application layer for command-line parsing, logging initialization,
//! configuration loading, and mode dispatch.

mod app;
#[allow(dead_code)]
mod chat;
mod config;

#[allow(dead_code)]
mod renderer;

// Deliberate while the frame pipeline awaits Task 2 wiring; the allow
// retires when the streaming module gets its first external consumer.
#[allow(dead_code)]
mod streaming;

use std::process::ExitCode;

fn main() -> ExitCode {
    app::run(&std::env::args_os().skip(1).collect::<Vec<_>>())
}

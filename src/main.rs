//! `rust-youtube-streamer-service` process entry point. Delegates to the
//! application layer for command-line parsing, logging initialization,
//! configuration loading, and mode dispatch.

mod app;
mod config;

use std::process::ExitCode;

fn main() -> ExitCode {
    app::run(&std::env::args_os().skip(1).collect::<Vec<_>>())
}

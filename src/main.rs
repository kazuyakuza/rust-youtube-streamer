//! Phase 00 build baseline: minimal executable proving the Cargo package builds.
//! Real application startup arrives with later feature phases.

// Temporary until Task 2's app layer consumes the config module.
// unused_imports covers the unreferenced pub use re-exports of this binary crate.
#[allow(dead_code, unused_imports)]
mod config;

fn main() {
    println!("rust-youtube-streamer-service");
}

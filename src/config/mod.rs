mod error;
mod loader;
mod model;
mod validation;

#[cfg(test)]
mod loader_tests;
#[cfg(test)]
mod validation_tests;

pub use error::{ConfigError, ConfigIssue};
pub use loader::load_from_path;
pub use model::{
    AppConfig, BroadcastConfig, ChatConfig, FfmpegConfig, RendererConfig, VideoConfig,
    YouTubeConfig,
};

mod error;
mod loader;
mod model;
mod validation;

#[cfg(test)]
mod test_fixtures;

pub use error::{ConfigError, ConfigIssue};
pub use loader::load_from_path;
pub use model::{
    AppConfig, BroadcastConfig, ChatConfig, FfmpegConfig, RendererConfig, VideoConfig,
    YouTubeConfig,
};

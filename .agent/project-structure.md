# Project Structure

# Folders in src/

- module folders contain only `.gitkeep` placeholders; feature modules are added by their respective phases
- app/ - application orchestration: startup, task coordination, error handling, shutdown
- config/ - loading, validation and exposure of the application configuration
- youtube/ - YouTube API integration: OAuth auth, broadcast lifecycle, streams, chat transport
- chat/ - chat message model and bounded visible-message queue (ChatStore)
- renderer/ - converts RenderInput to raw RGB24 video frames; no YouTube/FFmpeg dependency
- streaming/ - FFmpeg process supervision and raw-frame input pipeline

# Root files

- Cargo.toml - Rust package manifest: `rust-youtube-streamer-service` v0.1.0, edition 2021, no dependencies
- Cargo.lock - tracked lockfile for the executable package (sole root-level generated artifact)
- src/main.rs - minimal executable entry point proving the package builds (no features yet)
- Dockerfile - pinned official `rust:1.82` image; installs rustfmt and clippy components for build checks
- docker-compose.yml - `rust` service for Docker-based cargo checks; bind-mounts the project and redirects build artifacts to the `rust-streamer-target` named volume

# Other folders

- .agent/ - agent context: project-info/, todos/, rules/workflow indexes and the structure map
- .kilo/ - Kilo Code integration: agents/, rules/, commands/ and plans/
- .opencode/ - opencode integration: agents/, commands/ and opencode.json
- config/ - runtime JSON configuration files
- credentials/ - OAuth credentials and token storage location (no files committed here)
- docs/ - Documentation files
- scripts/ - developer check utilities; `dev-checks.sh` runs format/check/test/clippy checks in the Compose `rust` service and writes UTC-timestamped logs to the gitignored `logs/checks/` directory
- fonts/ - font files used by the renderer
- logs/ - application/chat log output directory (contents ignored; only .gitkeep placeholder versioned)

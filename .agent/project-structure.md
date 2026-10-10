# Project Structure

# Folders in src/

- app/ - application orchestration: CLI parsing and dispatch, startup pipeline, logging initialization, error boundaries
  - `cli.rs` - std-library CLI parser: `auth` | `run` | `--config <path>` (before/after the command) | `--help`; `DEFAULT_CONFIG_PATH` constant
  - `cli_tests.rs` - parser unit tests (12 cases: valid/invalid forms, default path)
  - `error.rs` - `StartupError { Usage, Configuration, LogInit }` with exit-code mapping (2/1/1) and Display contracts
  - `logging.rs` - tracing-subscriber init seam: `RUST_LOG` honored, default filter `info`, invalid value → one stderr warning + default
  - `mod.rs` - startup funnel: parse → init logging → load+validate config → dispatch; 3 whitelisted-field info events; `report_failure` single stderr line
  - `modes.rs` - `auth`/`run` placeholder handlers (config validated first; report not-implemented; exit 3)
- config/ - loading, validation and exposure of the application configuration
  - `mod.rs` - public API: `load_from_path(&Path) -> Result<AppConfig, ConfigError>`; module docs
  - `model.rs` - strongly typed structs mirroring `config/config.example.json` (serde derive, `deny_unknown_fields`)
  - `loader.rs` - read → parse → validate pipeline
  - `validation.rs` - 18 validation checks grouped as named predicates (5 positive dimensions/FPS/size values, exact `rgb24`, 1 vertical layout ≥ 1 line, 10 non-blank strings/paths incl. FFmpeg fields, 1 privacy enum)
  - `error.rs` - `ConfigError { Io, Deserialization, Validation }` + aggregated `ConfigIssue` (typed, actionable, no panics)
  - `loader_tests.rs` / `validation_tests.rs` - 16 unit tests (temp files via `tempfile`; no real developer config)
  - `test_fixtures.rs` - shared test fixture builder (`#[cfg(test)]`)
- youtube/ - YouTube API integration: OAuth auth, broadcast lifecycle, streams, chat transport (placeholder; future phase)
- chat/ - chat message model and bounded visible-message queue (ChatStore) (placeholder; future phase)
- renderer/ - converts RenderInput to raw RGB24 video frames; no YouTube/FFmpeg dependency (placeholder; future phase)
- streaming/ - FFmpeg process supervision and raw-frame input pipeline (placeholder; future phase)

# Root files

- Cargo.toml - Rust package manifest: `rust-youtube-streamer-service` v0.3.0, edition 2021; deps: serde (derive), serde_json, tracing, tracing-subscriber (env-filter); dev-dep: tempfile
- Cargo.lock - tracked lockfile for the executable package (sole root-level generated artifact)
- src/main.rs - entry point delegating to the `app` layer (`mod app; mod config;`, no lint suppressions)
- Dockerfile - pinned official `rust:1.82` image; installs rustfmt and clippy components for build checks
- docker-compose.yml - `rust` service for Docker-based cargo checks; bind-mounts the project and redirects build artifacts to the `rust-streamer-target` named volume

# Other folders

- .agent/ - agent context: project-info/, todos/, rules/workflow indexes and the structure map
- .kilo/ - Kilo Code integration: agents/, rules/, commands/ and plans/ (per-task implementation plans tracked)
- .opencode/ - opencode integration: agents/, commands/ and opencode.json
- config/ - runtime JSON configuration files; `config.example.json` is tracked; real `config.json` is gitignored
- credentials/ - OAuth credentials and token storage location (no files committed here)
- docs/ - Documentation files: `configuration.md` (configuration setup + CLI usage guide), `build.md` (Linux release build guide: prerequisites, standard invocation, output, failure semantics, limitations), plus agent how-to guides
- scripts/ - developer utilities; `dev-checks.sh` runs format/check/test/clippy checks in the Compose `rust` service and writes UTC-timestamped logs to the gitignored `logs/checks/` directory; `build-linux.sh` builds the release with the tracked lockfile in the same service and copies the final Linux executable to gitignored `dist/`
- fonts/ - font files used by the renderer
- logs/ - application/chat log output directory (contents ignored; only .gitkeep placeholder versioned)
- dist/ - gitignored release-build output; `build-linux.sh` writes only the final Linux executable `rust-youtube-streamer-service` here; the explicitly approved exception to the no-root-generated-files rule (contents never committed)

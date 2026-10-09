# Plan — Task 1: Implement the Configuration Module

- Date: 2026-10-09
- Status: PENDING USER APPROVAL
- TODO source: `.agent/todos/20261009/20261009-todo-3.md` → `### 1. Implement the Configuration Module`
- Parent plan: `.kilo/plans/20261009-phase01-configuration-app-skeleton.md`
- Source of truth: `.agent/project-info/brief.md` (§4, §5.1–5.3, §15.1)
- Implementer profile: JUNIOR under 50% restriction — every structural/architectural decision is encoded below; the implementer writes final code within these decisions only.

## 0. Verified Repository Facts (re-checked by architector)

- Branch: `feat/phase01-config-app-skeleton` (HEAD `c62e937`, `chore: bump version to 0.2.0`). Git branch creation was Step 2 and is already done; Task 1 does NOT create/switch branches.
- `Cargo.toml`: package `rust-youtube-streamer-service` v0.2.0, edition 2021, **zero dependencies**; `Cargo.lock` tracked.
- `src/config/` contains only `.gitkeep`; `src/main.rs` is the Phase 00 placeholder (4 lines + doc comments).
- `.gitignore` already ignores `/config/config.json` and `/target/`.
- All cargo commands must run through the Alpine VM MCP (`alpine-vm_vm_status` first, then `alpine-vm_vm_run_command`); the Windows host has no Rust/Cargo. Canonical pattern: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo <subcmd>`.
- Final verification command (full suite): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` (runs `fmt --check`, `check --locked`, `test --locked`, `clippy --locked -- -D warnings`; exit 0 = all pass). Exit code is authoritative; missing cargo `Finished` lines in MCP output is not a failure.
- Container must not write artifacts in the project root (named-volume `CARGO_TARGET_DIR`; `Cargo.lock` is the sole root exception).

## 1. Architectural Decisions (all binding)

### 1.1 Module file layout under `src/config/`

| File | Responsibility | Keep ≤ 200 lines |
|---|---|---|
| `src/config/mod.rs` | Module root: declares private child modules; re-exports the public API (`load_from_path`, `AppConfig` and section structs, `ConfigError`, `ConfigIssue`). No logic. | ~20 lines |
| `src/config/model.rs` | Typed configuration structs only (serde derives + attributes). No logic, no IO. | ~90 lines |
| `src/config/loader.rs` | File reading + JSON deserialization, then delegates to validation. Owns `load_from_path`. | ~60 lines |
| `src/config/validation.rs` | All validation rules returning typed, aggregated issues. Pure functions on `&AppConfig` — no IO, no panics. | ~150 lines |
| `src/config/error.rs` | `ConfigError` enum + `ConfigIssue` + hand-written `Display`/`std::error::Error` impls. | ~100 lines |

- Delete `src/config/.gitkeep` when the first real file is added (placeholder rule from the structure map).
- `src/main.rs`: add ONLY `#[allow(dead_code)]` + `mod config;` (with one short comment stating the attribute is temporary until Task 2's app layer consumes the module). Rationale: this is a binary crate, so an unreferenced module's `pub` items still trigger `dead_code`, and `cargo clippy -- -D warnings` would fail. The allow attribute is scoped to the module declaration, does not check in commented-out code, and MUST be removed in Task 2 when the app layer calls `load_from_path`. No other `main.rs` change (CLI behavior belongs to Task 2).
- `mod.rs` module declarations are plain `mod error; mod loader; mod model; mod validation;` (private children, re-exported API — prefer-private rule).

### 1.2 Typed-struct schema (mirrors brief §5.2 exact JSON property names)

In `model.rs`, all structs derive `Debug, Clone, PartialEq, Deserialize`; every container struct carries `#[serde(deny_unknown_fields)]`; every field is required (no `#[serde(default)]` anywhere).

`#[serde(deny_unknown_fields)]` belongs on each STRUCT definition (it applies to the JSON map that struct deserializes from, NOT the parent container's fields). Exact shape:

```rust
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AppConfig {
    pub youtube: YouTubeConfig,
    pub video: VideoConfig,
    pub renderer: RendererConfig,
    pub chat: ChatConfig,
    pub ffmpeg: FfmpegConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YouTubeConfig { pub broadcast: BroadcastConfig }

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BroadcastConfig { pub title: String, pub description: String, pub privacy_status: String }

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VideoConfig { pub width: u32, pub height: u32, pub fps: u32, pub pixel_format: String }

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RendererConfig {
    pub font: String, pub font_size: u32, pub line_height: u32,
    pub left_margin: u32, pub top_margin: u32, pub right_margin: u32, pub bottom_margin: u32,
    pub text_color: String, pub background_color: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChatConfig { pub log_file: String }

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FfmpegConfig { pub executable: String, pub video_codec: String, pub preset: String, pub bitrate: String }
```

Decisions encoded:
- **Property names** are exactly the JSON names from brief §5.2 (`privacy_status`, `pixel_format`, `log_file`, `video_codec`, …). No rename attributes needed.
- **Numeric types are unsigned (`u32`)** for width, height, fps, font_size, line_height, and all four margins. Justification: negative values are nonsense for these fields; serde automatically rejects a JSON negative (e.g. `-20`) with a deserialization error naming the expected type — the plan does NOT reimplement sign checks in validation for negatives. Validation then enforces the semantic positivity bounds (`> 0`, `≥ 0` is automatic for u32).
- **Privacy status is `String` (not a Rust enum consumed by serde).** Justification: TODO requires rejecting unsupported values via the validation pass so errors list the accepted set; a String + validation equals a custom enum minus serde complexity. (A `#[derive(Deserialize)]` enum would also work, but the plan picks String + validation for uniformity with other rule-based errors. Do NOT do both.)
- **No custom `Deserialize` impls. Derive-only.** Justification: every semantic rule from the TODO is expressible in a post-deserialization validation pass; hand-rolled deserialize would duplicate field parsing for zero benefit. Structural strictness (deny unknown fields, missing fields, negative numbers) comes free from serde derive.
- **`deny_unknown_fields` on every struct: YES.** Justification: a typo like `privcy_status` or `max_messages` must be rejected, never silently ignored (brief §5.3 spirit: reject unsupported values rather than silently substituting). The config surface is small and controlled; strictness is the safer default.
- **No `max_messages` field** (TODO forbids; visible-line capacity is derived later from height/margins/line_height — validation only guarantees ≥ 1 line fits).
- Field visibility: fields are `pub` — these structs are the module's public data contract consumed by the app layer in Task 2; the prefer-private rule applies to internal logic, and plain data DTOs are the standard exception. `AppConfig` and all section structs are re-exported from `mod.rs`.

### 1.3 Dependencies (exact `Cargo.toml` additions)

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
tempfile = "3"
```

Rationale (encode in completion report):
- `serde` + `serde_json`: required by the TODO for typed JSON deserialization. Widely maintained, ecosystem-standard.
- **Error reporting: NO `thiserror`.** Hand-written `Display` + `Error` impls in `error.rs`. Justification: the error is one internal enum in one file (~60 impl lines); `thiserror` adds a proc-macro dependency for marginal gain, and brief §3.5 says avoid a dependency when the standard library suffices.
- **Tests: use the `tempfile` dev-dependency**, NOT `std::env::temp_dir` + hand-rolled unique names. Justification: collision-free path generation and automatic cleanup on drop are exactly what `tempfile` guarantees; hand-rolling pid/thread counters is brittle low-value code. As a dev-dependency it does not enter release builds.

`Cargo.lock` refresh is part of the implementation loop (Step 4.2.1 of this plan).

### 1.4 Loading API

In `loader.rs`:

```rust
pub fn load_from_path(path: &Path) -> Result<AppConfig, ConfigError>
```

Pipeline (three internal steps, each a short function):
1. `read_file(path) -> Result<String, ConfigError>` — `std::fs::read_to_string`; on failure wraps the io error as `ConfigError::Io { path, source }`.
2. `parse(path, raw) -> Result<AppConfig, ConfigError>` — `serde_json::from_str`; failure → `ConfigError::Deserialization { path, message }` (the serde_json message string, which includes line/column context, is stored in `message`).
3. `validation::validate(&config)` — failure → `ConfigError::Validation { issues }`.

- **No default path in this module.** `config/config.json` default-path constant belongs to the app layer (Task 2). The config module must not hardcode or reference any default path (the `--config` option and default are Task 2 scope).
- Path is taken as `&Path` (not `&str`), no CWD assumptions.

### 1.5 Error types (`error.rs`)

```rust
pub enum ConfigError {
    Io { path: PathBuf, source: io::Error },
    Deserialization { path: PathBuf, message: String },
    Validation { issues: Vec<ConfigIssue> },
}

pub struct ConfigIssue { field: String, problem: String }   // fields private
```

- Hand-written impls: `impl Display for ConfigError` / `for ConfigIssue`, `impl std::error::Error for ConfigError` with `source()` returning the io `source` for the Io variant.
- `ConfigIssue` fields private; the `Display` output is the testing contract.
- **No panics anywhere** for expected file/JSON/validation errors; no `unwrap`/`expect` outside genuine internal invariants (there are none planned — do not use them).
- Display contracts (exact prefixes the tests assert against):
  - Io: `unable to read configuration file '<path>': <io error>`
  - Deserialization: `failed to parse configuration file '<path>': <serde message>`
  - Validation: `invalid configuration (<n> issue(s)): <issue>; <issue>; …`
  - Issue: `<field path>: <problem>` e.g. `video.width: must be positive (got 0)`.
- **Validation collects ALL issues and reports them together** (one actionable pass), not first-error-abort. Rationale: better startup diagnostics.

### 1.6 Validation rules (`validation.rs`) — explicit contract

`pub(super) fn validate(config: &AppConfig) -> Result<(), ConfigError>`; internally splits into per-section helpers (keeps functions ≤ 50 lines, ≤ 2 params — helper takes `(&Section, &mut Vec<ConfigIssue>)` or appends via a small issue-collector; choose plain two-param helpers appending into `&mut Vec<ConfigIssue>`).

Field-path strings use dot notation + JSON names: `youtube.broadcast.title`, `video.width`, `renderer.top_margin`, `chat.log_file`, `ffmpeg.executable`, …

1. `video.width > 0` → problem `must be positive (got N)`
2. `video.height > 0` → same
3. `video.fps > 0` → same
4. `video.pixel_format == "rgb24"` (exact, case-sensitive) → `unsupported pixel format '<value>'; the MVP requires 'rgb24'`
5. `renderer.font_size > 0` → `must be positive (got N)`
6. `renderer.line_height > 0` → `must be positive (got N)`
7. Vertical layout (single named predicate, e.g. `is_unusable_vertical_layout(video, &renderer)` on the struct refs): `u64`-arithmetic comparison `top_margin + bottom_margin + line_height > height` (u64 avoids u32 overflow) → `vertical layout leaves no room for at least one text line (height N, top_margin N, bottom_margin N, line_height N)`. This encodes `height - top_margin - bottom_margin >= line_height`.
8. Margins ≥ 0 is automatic via `u32` (document in code-free naming only; no explicit check).
9. Non-empty & not-whitespace-only strings: `youtube.broadcast.title`, `youtube.broadcast.description`, `renderer.font`, `renderer.text_color`, `renderer.background_color`, `chat.log_file`, `ffmpeg.executable`, `ffmpeg.video_codec`, `ffmpeg.preset`, `ffmpeg.bitrate` → `must not be empty or whitespace-only`. One named helper (`is_blank(value: &str)`).
10. `youtube.broadcast.privacy_status` ∈ {`private`, `public`, `unlisted`} exact, case-sensitive → `unsupported privacy status '<value>'; expected one of: private, public, unlisted`
11. FFmpeg `video_codec` / `preset` / `bitrate`: covered by rule 9 (present is structural — serde forbids missing; non-empty is rule 9). No format validation beyond non-empty (TODO requires only presence/non-emptiness).
12. **NO font-file or FFmpeg-executable existence checks** (TODO explicitly defers existence validation to the consuming components; config validates path syntax via non-emptiness only).
13. No hex-format validation for colors (not demanded by the TODO; only rule 9 non-emptiness applies).

### 1.7 `config/config.example.json` — exact content

Create `config/example` file with EXACTLY the JSON from brief §5.2 (privacy `unlisted`, safe non-secret values). Verbatim:

```json
{
  "youtube": {
    "broadcast": {
      "title": "Rust YouTube Streamer Prototype",
      "description": "YouTube Live streaming prototype",
      "privacy_status": "unlisted"
    }
  },
  "video": {
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "pixel_format": "rgb24"
  },
  "renderer": {
    "font": "fonts/console.ttf",
    "font_size": 32,
    "line_height": 40,
    "left_margin": 20,
    "top_margin": 20,
    "right_margin": 20,
    "bottom_margin": 20,
    "text_color": "#FFFFFF",
    "background_color": "#000000"
  },
  "chat": {
    "log_file": "logs/chat.log"
  },
  "ffmpeg": {
    "executable": "ffmpeg",
    "video_codec": "libx264",
    "preset": "veryfast",
    "bitrate": "6000k"
  }
}
```

- Do **NOT** create `config/config.json` (gitignored; would be flagged by the Gitignore Compliance Rule if staged).
- Keep `config/.gitkeep`.

### 1.8 Test architecture — `tests/config_tests.rs` (integration test file)

**PICK: a single `tests/` integration file** over in-module `#[cfg(test)]`. Justification: the module's real contract is the whole pipeline (file → parse → validate) exposed by `load_from_path`; behavior-level tests against the public API directly mirror how the app layer (Task 2) consumes it, keep `src/` files well below the 200-line limit, and avoid re-testing serde internals through unit holes.

Shared test helper (top of `tests/config_tests.rs`): `TempDir::new()` from `tempfile`, write a JSON string to `dir.path().join("config.json")`, call `crate::config::load_from_path` (import as `use rust_youtube_streamer_service::config::load_from_path;` — NOTE: this requires the integration test to access the binary crate. Because it is a binary-only crate, integration tests CANNOT import it. **Resolution decision:** config tests are therefore in-module `#[cfg(test)]` — this overrides the initial pick. Split as:
- `src/config/loader.rs`: `#[cfg(test)] mod tests;` style — actually place tests in **`src/config/loader_tests.rs`** referenced from `loader.rs` via `#[cfg(test)] mod loader_tests;` with `use super::*;`. File holds the file-pipeline tests (valid, missing file, unreadable, malformed JSON, missing field, unknown field, negative number).
- `src/config/validation.rs`: holds `#[cfg(test)] mod validation_tests;` in **`src/config/validation_tests.rs`** calling `validation::validate(&AppConfig { .. })` directly with hand-built structs (no JSON needed — faster and targets rules precisely).

Justification encoded: binary crate cannot host `tests/` imports of `main`-only crates; in-module `cfg(test)` compiles them with `cargo test`. Both test files use `tempfile` only where file IO is involved (loader tests); validation tests build structs directly and need no temp files.

Enumerated test cases (EVERY one required by the TODO):

| # | Test name (suggested) | Target file | Setup | Expected |
|---|---|---|---|---|
| 1 | `loads_valid_example_config` | loader_tests | embedded copy of §5.2 example JSON in TempDir | Ok; assert every struct field equals the example value (PartialEq) |
| 2 | `missing_file_reports_io_error` | loader_tests | TempPath that was never written (`TempDir` + non-existent filename) | `ConfigError::Io`; Display contains the full path; no panic |
| 3 | `unreadable_file_reports_io_error` | loader_tests | write a JSON file, then pass the path of a **directory** created under TempDir (e.g. `dir.path().join("some-dir")` after `fs::create_dir`) → reading a directory as a file yields an io error and is reliable even when tests run as root in the container (chmod-based unreadability is NOT reliable as root — do not use chmod) | `ConfigError::Io`; Display contains the path and the io error reason |
| 4 | `malformed_json_reports_deserialization_error` | loader_tests | file contains `{"youtube": {` (or `{ not json`) | `ConfigError::Deserialization`; Display contains the path |
| 5 | `missing_required_field_reports_error` | loader_tests | example JSON minus the `video` section (and second case: minus `ffmpeg.bitrate`) | `ConfigError::Deserialization`; Display contains `missing field` |
| 6 | `unknown_field_rejected` | loader_tests | example JSON + `"max_messages": 10` inside renderer | `ConfigError::Deserialization` (deny_unknown_fields) |
| 7 | `negative_number_rejected` | loader_tests | `"width": -5` | `ConfigError::Deserialization`; Display contains `width` and expected type (from serde message…) — assert only that the variant is Deserialization and message mentions `width` if provided by serde; do NOT hardcode serde's exact wording; minimum assertion is variant match |
| 8 | `zero_width_rejected` | validation_tests | hand-built valid struct with `width = 0` | `ConfigError::Validation` with an issue naming `video.width` |
| 9 | `zero_height_or_fps_rejected` | validation_tests | same pattern (`height = 0`; `fps = 0` as separate cases) | issue naming `video.height` / `video.fps` |
| 10 | `unsupported_pixel_format_rejected` | validation_tests | `pixel_format = "rgba"` | issue naming `video.pixel_format` and `rgb24` |
| 11 | `zero_font_size_or_line_height_rejected` | validation_tests | `font_size = 0`; separate case `line_height = 0` | issue naming `renderer.font_size` / `renderer.line_height` |
| 12 | `vertical_layout_without_one_line_rejected` | validation_tests | `height = 100, top_margin = 40, bottom_margin = 40, line_height = 40` (100-80=20 < 40) | issue naming the vertical-layout problem and quoting height/line_height numbers |
| 13 | `vertical_layout_boundary_accepts_one_line` | validation_tests | `height = 100, top = 30, bottom = 30, line_height = 40` (exactly 1 line: 40 == 40) | Ok — boundary accepted |
| 14 | `unsupported_privacy_status_rejected` | validation_tests | `privacy_status = "internal"` (also test a case-mismatch like `"Public"` → rejected) | issue naming `youtube.broadcast.privacy_status` and the allowed list |
| 15 | `empty_required_value_rejected` | validation_tests | blank variants: `title = "  "`, `description = ""`, `font = "   "`, `log_file = ""`, `executable = ""`, `video_codec = ""`, `preset = " "`, `bitrate = ""` (loop or parametrized-style separate asserts; keep each its own small assert line) | issue naming each respective field |
| 16 | `multiple_issues_reported_together` | validation_tests | width 0 AND fps 0 AND privacy `internal` simultaneously | `ConfigError::Validation` whose Display contains ALL three field paths (proves aggregate reporting) |

Rules for tests: no `unwrap` on load results other than the happy path; match error variants via `match`/`if let`; assert message content via `Display` string `contains` on the FIELD PATH only (never on serde's exact phrasing beyond `missing field` where stable); every temp file lives in a `TempDir` from the `tempfile` crate; no test reads any developer's real `config/` files; no network, no secrets, no real font/FFmpeg paths relied upon.

### 1.9 Modular-monolith boundary

- `src/config/**` may import ONLY: `std::*`, `serde`, `serde_json` (and `tempfile` in `#[cfg(test)]` test files).
- NO dependency (module, type, or re-export) on `youtube`, `renderer`, `chat`, `streaming`, or `app`. Violation = implementation error to be fixed immediately.
- Config exposes data + `load_from_path`; it performs NO logging (logging stack is Task 3) and NO process launching.

## 2. Implementation Steps (ordered, atomic)

### Step 4.2.1 — Dependency wiring first (so the rest compiles green)

1. Call `alpine-vm_vm_status`; retry/stop per workflow rules if VM is down.
2. Edit `Cargo.toml`: add `[dependencies]` with `serde` (features `["derive"]`) and `serde_json`, plus `[dev-dependencies]` section with `tempfile = "3"` (exact text in §1.3).
3. Refresh the lockfile through the container (normal, non-locked run):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check`
   Exit 0 → `Cargo.lock` now pins the three crates. Verify `Cargo.lock` shows serde/serde_json/tempfile entries (`Select-String` locally on the host is fine — no VM needed for reading).
4. Then run `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked` → must be exit 0 (proves lock discipline).
5. Nothing committed yet (implementer commits in bulk per Step 4.2.3; the plan deliberately commits code+deps together — no intermediate commit needed. If the implementer prefers a fine-grained commit at this point, allowed message: `chore: add serde, serde_json and tempfile dependencies`).

### Step 4.2.2 — Code writing order (build bottom-up: errors → model → validation → loader → mod → main)

Write in this exact order — each step only compiles once its dependencies exist:

1. **`src/config/error.rs`** — `ConfigError` enum, `ConfigIssue` struct (private fields), `Display` impls (contracts in §1.5), `impl std::error::Error for ConfigError` with `source()`.
2. **`src/config/model.rs`** — all seven structs exactly per §1.2, with derives (`Debug, Clone, PartialEq, Deserialize`) and `#[serde(deny_unknown_fields)]` on every section struct + `AppConfig` (top-level struct gets `#[serde(deny_unknown_fields)]` only at its own level).
3. **`src/config/validation.rs`** — `validate` + per-section helpers (`validate_video`, `validate_renderer`, `validate_youtube`, `validate_chat_and_ffmpeg` — naming may adjust but responsibilities must stay section-scoped) + named predicates `is_blank(const…)` / `is_unusable_vertical_layout` per the Single-Section Boolean rule. Every rule from §1.6 in the exact order listed (deterministic ordering matters for the aggregate test 16).
4. **`src/config/loader.rs`** — `load_from_path` + the 2 internal steps (§1.4).
5. **`src/config/mod.rs`** — private `mod` declarations + public `pub use` re-exports (§1.1).
6. **`src/config/loader_tests.rs` + `src/config/validation_tests.rs`** — the 16 enumerated tests (§1.8), wired via `#[cfg(test)] mod loader_tests;` / `#[cfg(test)] mod validation_tests;` inside `loader.rs` / `validation.rs` respectively.
7. **`src/main.rs`** — add `#[allow(dead_code)]\nmod config;` with the temporary-attribute comment (§1.1; nothing else changes).
8. **`config/config.example.json`** — exact content §1.7.
9. **Delete `src/config/.gitkeep`** and confirm `git status` shows it as removed (staging happens in the commit step).

Junior guardrails during writing:
- No commented-out code (No Commented Code rule). Comments allowed only where genuinely explanatory (e.g., the temporary `allow(dead_code)` rationale, u64-arithmetic rationale).
- Descriptive names: `ConfigIssue`, `validate`, `is_blank`, `is_unusable_vertical_layout`, `load_from_path` — do not abbreviate.
- Functions ≤ 50 lines, nesting ≤ 2 depths, ≤ 2 args (helper pattern: section ref + `&mut Vec<ConfigIssue>`).
- Terminal width/newlines: files use real newlines; no literal `\n` sequences.

### Step 4.2.3 — Commits (exact, ordered)

Commit A after steps 1–5 + 7–9 compile:
```
git add Cargo.toml Cargo.lock src/config src/main.rs config/config.example.json
git rm src/config/.gitkeep
```
Converted to practice: use `git add` on the listed paths, `git rm` for the placeholder; verify no gitignored file is staged (`git status`; `/config/config.json` must NOT appear). Commit message (exact):
`feat: add typed configuration module with validation`

Commit B after the test files (steps 6):
```
git add src/config/loader_tests.rs src/config/validation_tests.rs src/config/loader.rs src/config/validation.rs
```
(only the two `#[cfg(test)] mod` lines already committed in A would be re-added implicitly if included — acceptable; message exact):
`test: cover configuration loading and validation`

Practical sequencing simplification (allowed): if implementing code+tests in one sitting, both commits are still required in this order (deps+code first, tests second) — matches the suggested messages.

### Step 4.2.4 — Build/test loop through the Alpine VM

Run after each writing step that matters (at minimum after steps 4, 5, 6, 8 of 4.2.2):

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check
```

Then run the targeted test module:

```
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test config
```

On failure: fix per plan rules only; no scope/structure changes without returning to the architector.

### Step 4.2.5 — Final verification (full suite)

`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → required exit 0 with four PASS lines (`fmt-check`, `check`, `test`, `clippy`) and `ALL CHECKS PASSED`. Record the log path printed by the script (under `logs/checks/`).

Additional checks by the implementer before finishing:
- `git status` clean except staged work; no `.gitignore`-matching file staged (`/config/config.json` absent, `/target/` absent, `logs/*` absent except `.gitkeep`).
- `config/config.json` must NOT exist on disk (`Test-Path config\config.json` → False expected locally).
- `config/config.example.json` is tracked (`git ls-files config`).

## 3. Review & Docs Handoff (for later cycle steps — NOT executed by this plan's implementer)

- 4.3 code-reviewer + code-simplifier work on the same files; expected review focus: rule coverage completeness (§1.6 all 13 points), no `unwrap` leaks, line limits, mod.rs re-export shape.
- 4.4 docs-specialist adds code-level docs only (module doc comments in `mod.rs`); README / `/docs` / project-structure updates belong to Task 4 per the TODO.

## 4. Explicitly OUT of Scope for Task 1

- No CLI parsing, no `auth`/`run` commands, no `--config` option, no default-path constant, no exit codes (all Task 2).
- No logging stack (`tracing` etc.) and no log initialization (Task 3).
- No README/docs/`.agent` metadata updates (Task 4).
- No `config/config.json` creation.
- No font-file / FFmpeg-executable existence checks.
- No color hex-format validation; no bitrate format validation.
- No YouTube API, chat store, renderer, FFmpeg pipelines, services, installers, or CI changes.
- No `main.rs` behavior change beyond the module declaration (§1.1).

## 5. Acceptance Checklist for this Task's plan (verification by 4.5b)

- [ ] All 7 structs present with deny_unknown_fields; types match §1.2; no `max_messages`.
- [ ] `load_from_path(&Path)` public; zero default-path logic in config/.
- [ ] All 13 validation rules implemented; aggregate reporting; messages follow §1.5 contracts.
- [ ] 16 test cases present and passing via the Docker workflow.
- [ ] `config/config.example.json` content == §1.7 verbatim; `config/config.json` nonexistent.
- [ ] Cargo.toml additions exactly §1.3; Cargo.lock updated; `--locked` variants pass.
- [ ] Two commits with §4.2.3 messages; dev-checks full suite exit 0.
- [ ] No forbidden dependency (`thiserror` implicitly rejected), no boundary violation, no out-of-scope feature.

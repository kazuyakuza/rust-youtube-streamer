# Simplification Plan — Task 1: Configuration Module (4.3)

- Date: 2026-10-09
- Reviewer: code-simplifier
- Scope reviewed: `src/config/{mod,loader,model,validation,error,loader_tests,validation_tests}.rs`, `config/config.example.json`, `src/main.rs` (module decl only), `Cargo.toml`
- Commits under review: `a612009` (`feat: add typed configuration module with validation`), `09ca897` (`test: cover configuration loading and validation`)
- Authoritative baseline (MUST NOT be violated): `.kilo/plans/20261009-task1-config-module.md`
- Implementer profile: JUNIOR, 50% restriction. Steps are atomic and fully specified; no judgment calls required.

## 0. Baseline state (verified this session)

- Alpine VM reachable; full suite green: `fmt-check` / `check` / `test` / `clippy` all `exit=0`, `ALL CHECKS PASSED`.
- Tests observed: 16 (`config::loader_tests::*` = 7, `config::validation_tests::*` = 9). All pass.
- No source file exceeds the 200-line hard cap. No commented-out code. All functions ≤ 50 lines. Nesting depth ≤ 2 everywhere.
- Behavior/contract frozen by baseline that MUST stay identical:
  - `load_from_path(&Path) -> Result<AppConfig, ConfigError>` signature and its 3-step pipeline.
  - `ConfigError` variants + exact `Display` prefixes (`unable to read configuration file`, `failed to parse configuration file`, `invalid configuration (<n> issue(s))`, `<field>: <problem>`).
  - `AppConfig`/section struct shapes, field names, types, `deny_unknown_fields`.
  - Validation rule set, message strings, and deterministic aggregation order (test `multiple_issues_reported_together` depends on order).
  - The 16 test scenarios (same count, same assertions).

## 1. Findings summary

| # | Location | Issue | Class | Action |
|---|----------|-------|-------|--------|
| F1 | `validation.rs` `push_blank_issue_if_needed` | 3 parameters → violates binding "max 2 params" rule | Rule defect | MUST (S1) |
| F2 | `loader_tests.rs` / `validation_tests.rs` / `mod.rs` | The full `AppConfig` valid-fixture literal is duplicated verbatim across two test files (`example_app_config` ≡ `valid_config`) | DRY / test scaffolding | RECOMMENDED (S2) |
| F3 | `validation_tests.rs` `empty_required_value_rejected` | 8 near-identical build→mutate→assert blocks; a tiny helper collapses them | DRY / test scaffolding | RECOMMENDED (S3) |

No other real wins found. See §4 for items deliberately left untouched (taste/churn, plan-mandated, or build-correct).

## 2. Steps

Order: **S1 (MUST)** then optionally **S2**, then **S3** (S3 assumes S2 is applied; if S2 is skipped, keep local `valid_config` and S3 still works). Run the verification in §3 after each step.

### S1 — MUST — Remove the 3-param helper in `validation.rs`

Replaces the rule-violating `push_blank_issue_if_needed(field, value, issues)` (3 params) with a pure 2-param predicate `blank_issue(field, value) -> Option<ConfigIssue>`, pushed via `Vec::extend`. This is behavior-preserving: same field paths, same `BLANK_VALUE_PROBLEM` string, same ordering. `ConfigIssue::new` signature is NOT changed.

File: `src/config/validation.rs`

Edit 1 — replace the helper (current lines ~122-129).

Before:
```rust
fn push_blank_issue_if_needed(field: &str, value: &str, issues: &mut Vec<ConfigIssue>) {
    if is_blank(value) {
        issues.push(ConfigIssue::new(
            field.to_string(),
            BLANK_VALUE_PROBLEM.to_string(),
        ));
    }
}
```

After:
```rust
fn blank_issue(field: &str, value: &str) -> Option<ConfigIssue> {
    if is_blank(value) {
        Some(ConfigIssue::new(
            field.to_string(),
            BLANK_VALUE_PROBLEM.to_string(),
        ))
    } else {
        None
    }
}
```

Edit 2 — rewrite the body of `validate_required_strings` so every `push_blank_issue_if_needed(a, b, issues);` becomes `issues.extend(blank_issue(a, b));`, keeping the EXACT same field paths, same value expressions, and same order. Current `&broadcast`/`&renderer`/`&ffmpeg` local bindings may be dropped if each call now references the fields directly, but keeping them is also acceptable. Final body (order preserved):

```rust
fn validate_required_strings(config: &AppConfig, issues: &mut Vec<ConfigIssue>) {
    let broadcast = &config.youtube.broadcast;
    let renderer = &config.renderer;
    let ffmpeg = &config.ffmpeg;
    issues.extend(blank_issue("youtube.broadcast.title", &broadcast.title));
    issues.extend(blank_issue("youtube.broadcast.description", &broadcast.description));
    issues.extend(blank_issue("renderer.font", &renderer.font));
    issues.extend(blank_issue("renderer.text_color", &renderer.text_color));
    issues.extend(blank_issue("renderer.background_color", &renderer.background_color));
    issues.extend(blank_issue("chat.log_file", &config.chat.log_file));
    issues.extend(blank_issue("ffmpeg.executable", &ffmpeg.executable));
    issues.extend(blank_issue("ffmpeg.video_codec", &ffmpeg.video_codec));
    issues.extend(blank_issue("ffmpeg.preset", &ffmpeg.preset));
    issues.extend(blank_issue("ffmpeg.bitrate", &ffmpeg.bitrate));
}
```

Fallback (only if clippy rejects `extend` on an `Option` in this toolchain, which is unexpected): use `if let Some(issue) = blank_issue(<path>, <value>) { issues.push(issue); }` per line. Ordering is still preserved.

Constraints check: `blank_issue` = 2 params, nesting depth 1; `validate_required_strings` = 2 params, no `if` in body (flat); `is_blank` remains used (no dead code); no message/variant changes; deterministic order unchanged.

### S2 — RECOMMENDED — Deduplicate the shared valid-config fixture (test-only)

The identical `AppConfig` literal exists twice: `loader_tests::example_app_config()` and `validation_tests::valid_config()`. Move it into one `#[cfg(test)]` fixture module. No production code, public API, dependency, or scenario-count change. `example_json()` / `example_json_value()` STAY in `loader_tests.rs` (JSON-pipeline only).

Step S2.1 — create file `src/config/test_fixtures.rs` with the current `validation_tests::valid_config()` body moved verbatim:
```rust
use super::*;

pub(super) fn valid_config() -> AppConfig {
    AppConfig {
        youtube: YouTubeConfig {
            broadcast: BroadcastConfig {
                title: "Rust YouTube Streamer Prototype".to_string(),
                description: "YouTube Live streaming prototype".to_string(),
                privacy_status: "unlisted".to_string(),
            },
        },
        video: VideoConfig {
            width: 1920,
            height: 1080,
            fps: 30,
            pixel_format: "rgb24".to_string(),
        },
        renderer: RendererConfig {
            font: "fonts/console.ttf".to_string(),
            font_size: 32,
            line_height: 40,
            left_margin: 20,
            top_margin: 20,
            right_margin: 20,
            bottom_margin: 20,
            text_color: "#FFFFFF".to_string(),
            background_color: "#000000".to_string(),
        },
        chat: ChatConfig {
            log_file: "logs/chat.log".to_string(),
        },
        ffmpeg: FfmpegConfig {
            executable: "ffmpeg".to_string(),
            video_codec: "libx264".to_string(),
            preset: "veryfast".to_string(),
            bitrate: "6000k".to_string(),
        },
    }
}
```
Note: `use super::*;` pulls the section structs from the `config` module's re-exports (already `pub use model::{...}` in `mod.rs`), so no `use crate::config::model` import is needed here.

Step S2.2 — in `src/config/mod.rs`, add the fixture module next to the existing test modules (keep it `#[cfg(test)]`):
```rust
#[cfg(test)]
mod test_fixtures;
```

Step S2.3 — in `src/config/validation_tests.rs`:
- Delete the local `fn valid_config() -> AppConfig { ... }` (lines ~157-193).
- Delete now-unused import line `use crate::config::model::{BroadcastConfig, ChatConfig, FfmpegConfig, YouTubeConfig};` (line 1).
- Add `use super::test_fixtures::valid_config;`.
- Keep `use crate::config::validation::validate;` and `use super::*;` (the glob still supplies `AppConfig` and `ConfigError` used by `expect_validation_error`).

Step S2.4 — in `src/config/loader_tests.rs`:
- Delete the local `fn example_app_config() -> AppConfig { ... }` (lines ~157-193).
- Delete now-unused import block `use crate::config::model::{BroadcastConfig, ChatConfig, FfmpegConfig, RendererConfig, VideoConfig, YouTubeConfig};` (lines ~3-5).
- Add `use super::test_fixtures::valid_config;`.
- In `loads_valid_example_config`, change `assert_eq!(loaded, example_app_config());` to `assert_eq!(loaded, valid_config());` (identical data → same assertion).
- Keep `use std::path::PathBuf;`, `use super::*;`, and the `example_json()`/`example_json_value()` helpers untouched.

Visibility: `mod test_fixtures;` is private to `config`; child test modules reach it as `super::test_fixtures`. `pub(super) fn valid_config` is visible throughout the `config` subtree. No new public surface escapes `config`.

### S3 — RECOMMENDED — Collapse the 8 blank-value blocks in `empty_required_value_rejected` (test-only)

`validation_tests::empty_required_value_rejected` repeats `let mut c = valid_config(); c.<field> = <blank>; assert!(expect_validation_error(&c).to_string().contains("<path>"))` eight times. Replace with one small helper. Same single `#[test]`, same eight assertions, same field paths — scenario count stays 16.

Step S3.1 — add this helper to `src/config/validation_tests.rs` (near the other helpers):
```rust
fn assert_blank_value_rejected(field: &str, make_blank: impl FnOnce(&mut AppConfig)) {
    let mut config = valid_config();
    make_blank(&mut config);
    let message = expect_validation_error(&config).to_string();
    assert!(message.contains(field));
}
```

Step S3.2 — rewrite the body of `empty_required_value_rejected` to eight calls (field path + closure setting the same blank value as before), preserving order:
```rust
#[test]
fn empty_required_value_rejected() {
    assert_blank_value_rejected("youtube.broadcast.title", |c| {
        c.youtube.broadcast.title = "  ".to_string()
    });
    assert_blank_value_rejected("youtube.broadcast.description", |c| {
        c.youtube.broadcast.description = "".to_string()
    });
    assert_blank_value_rejected("renderer.font", |c| c.renderer.font = "   ".to_string());
    assert_blank_value_rejected("chat.log_file", |c| c.chat.log_file = "".to_string());
    assert_blank_value_rejected("ffmpeg.executable", |c| {
        c.ffmpeg.executable = "".to_string()
    });
    assert_blank_value_rejected("ffmpeg.video_codec", |c| {
        c.ffmpeg.video_codec = "".to_string()
    });
    assert_blank_value_rejected("ffmpeg.preset", |c| c.ffmpeg.preset = " ".to_string());
    assert_blank_value_rejected("ffmpeg.bitrate", |c| c.ffmpeg.bitrate = "".to_string());
}
```
Constraints: helper = 2 params; closure bodies are single assignments; nesting depth ≤ 1; blank values/paths identical to the current test.

## 3. Verification (Alpine VM Docker workflow)

Always `alpine-vm_vm_status` first, then run via MCP `alpine-vm_vm_run_command`.

- Targeted after S1 / S2 / S3 (fast loop):
  - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt`
    (formatting is allowed to reflow wrapped lines; then:)
  - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings`
  - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked`
- Final gate (full suite, exit 0 required, expect four PASS lines + `ALL CHECKS PASSED`):
  - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`

Acceptance for this step:
- Test count remains 16; all green (S1 and S3 must not drop any assertion; S2 must not add/remove a `#[test]`).
- `validation.rs`: zero functions with > 2 params (F1 resolved).
- No new `#[allow(...)]`, no new dependency, no `src/` file > 200 lines.
- `Cargo.lock` unchanged in scope (S1-S3 add no crates; `tempfile` already present).

## 4. Considered and REJECTED (do NOT change — recorded so the implementer does not "helpfully" churn)

- `ConfigIssue::new(String, String)` → `(&str, impl Into<String>)`: would trim a few `.to_string()` calls but is pure ergonomics, touches a public-ish constructor + every call site, and is not rule-driven. Skipped to keep S1 self-contained.
- `model.rs` repeated `#[derive(...)]` + `#[serde(deny_unknown_fields)]`: idiomatically required; macro extraction would reduce clarity and contradicts the plan's verbatim struct shape. Leave.
- Single-use helpers `vertical_layout_issue`, `is_unsupported_privacy_status`, `write_validation`: self-documenting, keep (removing would inline long `format!`/chains and raise nesting).
- Names `is_unusable_vertical_layout`, `validate`, `is_blank`, `load_from_path`: plan-mandated. Leave.
- `src/main.rs` `#[allow(dead_code, unused_imports)]` differs from the plan text (`#[allow(dead_code)]`): the extra `unused_imports` is required to keep `clippy -D warnings` green for the currently unconsumed re-exports in a binary crate. Build-correctness, not a simplification target. Leave as-is.
- `config/config.example.json`, `loader.rs`, `error.rs`, `Cargo.toml`: reviewed, already minimal/clean; no changes.
- Test-module wiring location (`#[cfg(test)] mod loader_tests/validation_tests;` declared in `mod.rs` rather than inside `loader.rs`/`validation.rs` as the plan sketched): functionally equivalent and already green; relocating adds no value. Leave.

## 5. Out-of-scope for this step (report to caller, not planned here)

- None rise to architecture. All recommended changes are local, behavior-preserving refactors within `src/config` (S1 production; S2/S3 test-only). No TODO re-scope / architector routing needed.

## 6. This step's status

- DONE: review + this plan written. Baseline re-verified green.
- NOT DONE (by design): no source file modified; nothing committed. Implementation of S1-S3 is the next cycle step.

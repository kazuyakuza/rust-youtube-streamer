# Task 1 Config Module — Review Fix Plan

Date: 2026-10-09
Source review: commits `a612009` + `09ca897` on `feat/phase01-config-app-skeleton`

## Review Summary

All 16 enumerated tests pass and `scripts/dev-checks.sh` exits 0 (fmt/check/test/clippy).
The implementation correctly covers the schema, validation rules, error contracts, and example config.
Three small fixes are required to remove deviations from the binding plan/rules.

## Findings

### Acceptable / plan-compliant

1. **`#[derive(Debug)]` on `ConfigError` / `ConfigIssue`** — exactly matches §1.5; not a deviation.
2. **`tempfile` pinned to `3.14.0`** — acceptable toolchain-forced deviation. The pinned `rust:1.82` container cannot build `tempfile 3.27` (requires edition 2024); the `--precise` pin keeps the dependency workflow intact.

### Fix required

1. **`src/main.rs`: remove extra `unused_imports` allow.**
   - Plan §1.1 requires **only** `#[allow(dead_code)]` + `mod config;`. The current `#[allow(dead_code, unused_imports)]` adds an unplanned lint override.
   - `dead_code` alone is sufficient because the warnings stem from unused `pub` re-exports in a binary crate, not from unused `use` imports.

2. **Move `#[cfg(test)] mod` declarations to the files that own the tests.**
   - Plan §1.8 specifies `loader.rs` declares `#[cfg(test)] mod loader_tests;` and `validation.rs` declares `#[cfg(test)] mod validation_tests;`.
   - Current code declares both in `src/config/mod.rs`, which is a structural deviation.

3. **Eliminate the 3-argument helper in `src/config/validation.rs`.**
   - `push_blank_issue_if_needed(field, value, issues)` violates `max-arguments-per-method.md` (max 2 params).
   - Replace it with an inline array-of-tuples loop inside `validate_required_strings`, keeping `is_blank(value: &str)` as the only named predicate as planned.

## Fix Steps

### Step 1 — `src/main.rs`

Change:

```rust
// Temporary until Task 2's app layer consumes the config module.
// unused_imports covers the unreferenced pub use re-exports of this binary crate.
#[allow(dead_code, unused_imports)]
mod config;
```

To:

```rust
// Temporary until Task 2's app layer consumes the config module.
#[allow(dead_code)]
mod config;
```

### Step 2 — `src/config/loader.rs`

Add at the end of the file:

```rust
#[cfg(test)]
mod loader_tests;
```

### Step 3 — `src/config/validation.rs`

Add at the end of the file:

```rust
#[cfg(test)]
mod validation_tests;
```

### Step 4 — `src/config/mod.rs`

Remove the two test-module declarations:

```rust
#[cfg(test)]
mod loader_tests;
#[cfg(test)]
mod validation_tests;
```

Keep the private `mod` declarations and `pub use` re-exports unchanged.

### Step 5 — `src/config/validation.rs`: replace 3-arg helper

Replace:

```rust
fn validate_required_strings(config: &AppConfig, issues: &mut Vec<ConfigIssue>) {
    let broadcast = &config.youtube.broadcast;
    let renderer = &config.renderer;
    let ffmpeg = &config.ffmpeg;
    push_blank_issue_if_needed("youtube.broadcast.title", &broadcast.title, issues);
    push_blank_issue_if_needed(
        "youtube.broadcast.description",
        &broadcast.description,
        issues,
    );
    push_blank_issue_if_needed("renderer.font", &renderer.font, issues);
    push_blank_issue_if_needed("renderer.text_color", &renderer.text_color, issues);
    push_blank_issue_if_needed(
        "renderer.background_color",
        &renderer.background_color,
        issues,
    );
    push_blank_issue_if_needed("chat.log_file", &config.chat.log_file, issues);
    push_blank_issue_if_needed("ffmpeg.executable", &ffmpeg.executable, issues);
    push_blank_issue_if_needed("ffmpeg.video_codec", &ffmpeg.video_codec, issues);
    push_blank_issue_if_needed("ffmpeg.preset", &ffmpeg.preset, issues);
    push_blank_issue_if_needed("ffmpeg.bitrate", &ffmpeg.bitrate, issues);
}

fn push_blank_issue_if_needed(field: &str, value: &str, issues: &mut Vec<ConfigIssue>) {
    if is_blank(value) {
        issues.push(ConfigIssue::new(
            field.to_string(),
            BLANK_VALUE_PROBLEM.to_string(),
        ));
    }
}
```

With:

```rust
fn validate_required_strings(config: &AppConfig, issues: &mut Vec<ConfigIssue>) {
    let broadcast = &config.youtube.broadcast;
    let renderer = &config.renderer;
    let ffmpeg = &config.ffmpeg;
    let required_strings = [
        ("youtube.broadcast.title", broadcast.title.as_str()),
        ("youtube.broadcast.description", broadcast.description.as_str()),
        ("renderer.font", renderer.font.as_str()),
        ("renderer.text_color", renderer.text_color.as_str()),
        ("renderer.background_color", renderer.background_color.as_str()),
        ("chat.log_file", config.chat.log_file.as_str()),
        ("ffmpeg.executable", ffmpeg.executable.as_str()),
        ("ffmpeg.video_codec", ffmpeg.video_codec.as_str()),
        ("ffmpeg.preset", ffmpeg.preset.as_str()),
        ("ffmpeg.bitrate", ffmpeg.bitrate.as_str()),
    ];
    for (field, value) in required_strings {
        if is_blank(value) {
            issues.push(ConfigIssue::new(
                field.to_string(),
                BLANK_VALUE_PROBLEM.to_string(),
            ));
        }
    }
}
```

This keeps `validate_required_strings` under 50 lines, nesting at 2 levels, and every function at ≤ 2 parameters.

## Verification Commands

Run through the Alpine VM Docker workflow:

```bash
# 1. Targeted config tests
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test config

# 2. Full quality gate (must exit 0 with four PASS lines)
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```

Expected: all 16 config tests pass, `fmt --check`, `check --locked`, `test --locked`, and `clippy --locked -- -D warnings` all report PASS.

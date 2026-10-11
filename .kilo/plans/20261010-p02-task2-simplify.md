# Simplification Plan — Phase 02 Task 2 Renderer (Critical Workflow 4.3)

Applies the accepted simplifications from the 4.3 code-simplification review of the
Task 2 renderer (`src/renderer/**`), on branch `feat/phase02-renderer-chat-store`
(commits `c22c9f9`, `43fb2f3`, `b8d8119`, `4e8d8f8`; gate log
`logs/checks/20261011T035012Z.log`, 81 tests, exit 0).

Executor: junior implementer. Every edit below is fully specified — execute exactly
as written, one edit at a time. No judgment calls, no extra "improvements", no
reformatting of untouched code. This plan changes NO behavior.

## Authority and MUST-NOT-CHANGE list (hard, non-negotiable)

- TODO authority: `.agent/todos/20261010/20261010-todo-2.md` §2/§4 — pixel math
  determinism, error contracts, and every test expectation ARE behavior.
- Plan authority: `.kilo/plans/20261010-p02-task2-text-renderer.md` R1–R6 + A6–A9.
- After all five edits the code must satisfy:
  1. Every `RenderError` `Display` string byte-identical to R4 contracts, including
     the FrameSize `{expected}` overflow sentinel `usize::MAX` behavior.
  2. Blend/layout math exactly per R3/A8: `blend_channel` round-half-up linear
     interpolation; baseline `top_margin + scale_font.ascent() + i * line_height`;
     pen start `left_margin`; advance-before-skip; notdef `glyph_id.0 == 0` skip;
     `origin.floor()` placement.
  3. Clip rules: per-pixel physical frame-boundary clipping via `is_inside_frame`
     re-check; index formula `(y * width + x) * 3`; negative-origin wrap-through-u32
     must still be rejected by the bounds check.
  4. `rgb24_byte_count` / `has_non_positive_dimension` semantics unchanged — only
     their visibility changes; `lacks_visible_line` u64 math unchanged.
  5. All 19 R5 test names, assertions and helper semantics unchanged; test count
     stays exactly 81 across the crate.
  6. No new files, no deleted files, no `mod.rs` declaration changes, no new
     dependencies, no new `#[allow]`, no derive added to `TextRenderer`.
  7. Method bodies stay ≤50 lines, nesting ≤2 (trivially true for these edits).
- If any verification step fails or any final snippet does not apply verbatim,
  STOP and report the exact error — do not improvise a fix.

## Edit 1 — `src/renderer/frame.rs`: expose shared size predicates

Two single-keyword visibility changes; bodies untouched.

Change line 84 from:

```rust
fn has_non_positive_dimension(width: u32, height: u32) -> bool {
```

to:

```rust
pub(super) fn has_non_positive_dimension(width: u32, height: u32) -> bool {
```

Change line 88 from:

```rust
fn rgb24_byte_count(width: u32, height: u32) -> Option<usize> {
```

to:

```rust
pub(super) fn rgb24_byte_count(width: u32, height: u32) -> Option<usize> {
```

Nothing else in `frame.rs` changes.

## Edit 2 — `src/renderer/error.rs`: reuse `rgb24_byte_count` (deduplicate)

In the import block, after line 9 (`use std::path::PathBuf;`), add one blank line
and the crate import, so the import section becomes:

```rust
use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::renderer::frame::rgb24_byte_count;
```

Replace the `FrameSize` display arm (current lines 44–50) with:

```rust
            RenderError::FrameSize { width, height } => {
                let expected = rgb24_byte_count(*width, *height).unwrap_or(usize::MAX);
                write!(
                    formatter,
                    "frame size of width {width} and height {height} does not fit a usable RGB24 buffer of {expected} bytes"
                )
            }
```

Delete the trailing helper and its separating blank line (current lines 85–91):

```rust
fn expected_buffer_bytes(width: u32, height: u32) -> usize {
    (width as usize)
        .checked_mul(height as usize)
        .and_then(|pixels| pixels.checked_mul(3))
        .unwrap_or(usize::MAX)
}
```

The file must end after the `impl Error for RenderError { ... }` closing brace
(single trailing newline).

## Edit 3 — `src/renderer/settings.rs`: reuse the frame geometry predicate

In the crate import block (after `use crate::renderer::error::RenderError;`), add:

```rust
use crate::renderer::frame::has_non_positive_dimension;
```

so lines 9–11 become:

```rust
use crate::config::model::{RendererConfig, VideoConfig};
use crate::renderer::error::RenderError;
use crate::renderer::frame::has_non_positive_dimension;
```

In `RendererSettings::from_config`, change the guard (current line 74) from:

```rust
        if has_non_positive_frame(video) {
```

to:

```rust
        if has_non_positive_dimension(video.width, video.height) {
```

Delete the local helper and its separating blank line (current lines 103–106):

```rust
fn has_non_positive_frame(video: &VideoConfig) -> bool {
    video.width == 0 || video.height == 0
}
```

The `Err(RenderError::Geometry { width: video.width, height: video.height })`
arm body stays exactly as-is.

## Edit 4 — `src/renderer/settings.rs`: idiomatic `ok_or_else`

Replace `parse_hex_field` (current lines 114–122) with exactly:

```rust
fn parse_hex_field(field: &'static str, value: &str) -> Result<Rgb, RenderError> {
    parse_six_hex(value).ok_or_else(|| RenderError::InvalidColor {
        field,
        value: value.to_string(),
    })
}
```

(Formatting matches the `text.rs:44–47` `map_err` precedent; rustfmt-stable.)
`parse_six_hex`, `has_six_ascii_hex_digits`, the `TEXT_COLOR_FIELD`/
`BACKGROUND_COLOR_FIELD` constants and all other items are untouched.

## Edit 5 — `src/renderer/text_tests.rs`: idiomatic `expect` helper

Replace `default_renderer` (current lines 51–56) with exactly:

```rust
fn default_renderer() -> text::TextRenderer {
    text::TextRenderer::new(settings_with_font(FONT_PATH)).expect("vendored font must load")
}
```

`new_renderer_or_error` and every other helper/test stay byte-identical
(`expect_err` needs `Debug` on `TextRenderer`; intentionally NOT converted).

## Verification (Alpine VM MCP, single commands, exit codes authoritative)

Run in order after all five edits; stop on any non-zero exit and report the exact
output (never fix beyond this plan):

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --all -- --check`
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked`
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
   — must print `ALL CHECKS PASSED`, exit 0, and still report **81 tests** with
   identical renderer test names.

## Expected diff footprint

- `frame.rs`: +2 `pub(super)` keywords, line count unchanged (100).
- `error.rs`: −6 net lines (≈91 → ≈85), one import added, one helper deleted.
- `settings.rs`: −5 net lines (≈138 → ≈133), one import added, one helper deleted,
  one body collapsed.
- `text_tests.rs`: −3 net lines (≈253 → ≈250).
- No other file may appear in `git status` (especially not `fonts/console.ttf`,
  `Cargo.*`, `src/chat/**`, `src/config/**`, logs).

## Explicitly out of scope (do NOT act on these)

- `text_tests.rs` code-line count (≈214 excl. blanks/comments/imports) exceeds the
  200-line file cap. The remedy is a test-file split — a structural change beyond
  this plan's authority. Caller decision required (planner amendment or separate
  TODO). Do not split, move tests, or touch `mod.rs` in this task.
- Keeping: `source()`'s explicit four `None` arms (chat precedent), the `.ok()?`
  guards in `parse_six_hex` (no-panic safety), per-line `as_scaled`, `LineJob`,
  `new_renderer_or_error`'s match form, transposed test ink helpers, per-file test
  config builders.

## Coordination note (Planner, 2026-10-10) — merged execution with review-fix plan

Planner ruling on the two out-of-scope/overlap items raised by the reviews:

1. **`text_tests.rs` cap fix**: the caller chose the *planner amendment* route — the
   review-fix plan's Fix 1 (`src/renderer/text_test_helpers.rs` under `#[cfg(test)]`,
   verbatim relocation of the shared constants/helpers) is EXECUTED IN THIS COMBINED
   TASK (not a separate TODO). Executor adjustments forced by the split: import
   `text_test_helpers` from `text_tests.rs` via the correct full crate path
   (`use crate::renderer::text_test_helpers::*;` — the plan's bare path is
   module-relative and would not resolve); relocated items keep their bodies
   byte-identical, with visibility adjusted minimally (prefer `pub(super)`) to make
   the compiler green; `mod.rs` gains exactly the one `#[cfg(test)] mod
   text_test_helpers;` declaration.
2. **`expected_buffer_bytes` duplication**: the caller accepts THIS plan's simplification
   (Edits 1–2 dedup via `pub(super) frame::rgb24_byte_count`) and supersedes the
   review-fix plan's "accepted duplication" stance — both files are siblings inside
   `crate::renderer` on a pub(super) surface, so the intra-module dependency is fine.
3. **`default_renderer` (Edit 5)** applies to the helper's NEW location after the
   split (rewritten body `.expect("vendored font must load")` inside
   `text_test_helpers.rs`); `new_renderer_or_error` stays a match (unchanged verdict).
4. **`blend_channel` 3-arg fix (review-fix Fix 2)** executes with a body-preservation
   rule: only rewrap the parameters into the `ChannelBlend` struct; the computation
   body must remain semantically byte-identical to the current implementation
   (round-half-up floor interpolation; NO new clamp/rounding steps unless already
   present). If the review-fix snippet's body differs from current code, current code
   wins and the implementer reports the delta.
5. Commit split (meaningful messages, after the gate is green):
   - `refactor(renderer): share frame size helpers across renderer modules` —
     frame.rs (2× `pub(super)` only), error.rs, settings.rs.
   - `fix(renderer): split renderer test helpers and bound blend args` —
     mod.rs (one cfg(test) decl), text_test_helpers.rs (new), text_tests.rs, frame.rs
     (ChannelBlend part; if a file spans both commits, it belongs to the commit whose
     changes land last in that file — verify final tree state is the authoritative
     combined result).
   - Gate before commits: `sh scripts/dev-checks.sh` exit 0, four PASS lines, exactly
     81 tests with identical renderer test names. Empty capture ⇒ exit code authoritative.

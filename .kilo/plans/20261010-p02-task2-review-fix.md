# Fix Plan — Task 2 RGB24 Text Renderer Code Review

Review of commits `c22c9f9`, `43fb2f3`, `b8d8119`, `4e8d8f8` on branch `feat/phase02-renderer-chat-store`.

## Required Fixes

### 1. [MEDIUM] `src/renderer/text_tests.rs` exceeds `max-lines-per-file` limit

- **Rule touched**: `.kilo/rules/max-lines-per-file.md`
- **Current size**: 253 total lines / 214 code lines (blank=33, comment=2, import=4). Hard limit is 200 lines for source files in `src/`.
- **Why it matters**: the repository-wide file-size ceiling is unconditional for `src/` files; the test module is the only renderer source file that violates it.
- **Fix** (minor local refactor, no behavior change):
  1. Create `src/renderer/text_test_helpers.rs` under `#[cfg(test)]`.
  2. Move from `src/renderer/text_tests.rs` into the new file:
     - All constants (`WIDTH`, `HEIGHT`, `TOP_MARGIN`, `LINE_HEIGHT`, `LEFT_MARGIN`, `BOTTOM_MARGIN`, `FONT_SIZE`, `FONT_PATH`, `SAMPLE_TEXT`).
     - All helper functions (`base_renderer_config`, `base_video_config`, `settings_with_font`, `default_renderer`, `new_renderer_or_error`, `render_frame`, `pixel_at`, `has_ink_at`, `count_ink_in_row`, `count_ink_in_rows`, `count_ink_in_column`, `count_ink_in_columns`, `first_ink_row`, `first_ink_column`, `find_pixel_in_band`, `is_full_white`, `is_partial_gray`, `is_grayscale`).
  3. In `src/renderer/mod.rs` add:
     ```rust
     #[cfg(test)]
     mod text_test_helpers;
     ```
  4. In `src/renderer/text_tests.rs` replace the moved helpers with:
     ```rust
     use text_test_helpers::*;
     ```
  5. Verify both files are under 200 lines and the full gate still passes.

### 2. [MINOR] `blend_channel` in `src/renderer/frame.rs` has 3 arguments

- **Rule touched**: `.kilo/rules/max-arguments-per-method.md`
- **Location**: `src/renderer/frame.rs:96` (`fn blend_channel(background: u8, coverage: f32, text: u8) -> u8`)
- **Why it matters**: the rule caps methods/functions at 2 parameters; this private helper is not mandated by the plan to take 3 parameters.
- **Fix**:
  1. Introduce a small struct in `src/renderer/frame.rs`:
     ```rust
     struct ChannelBlend {
         background: u8,
         coverage: f32,
         text: u8,
     }
     ```
  2. Change `blend_channel` to:
     ```rust
     fn blend_channel(blend: ChannelBlend) -> u8 {
         let delta = blend.text as f32 - blend.background as f32;
         let blended = blend.background as f32 + (blend.coverage * delta + 0.5).floor();
         blended.clamp(0.0, 255.0) as u8
     }
     ```
  3. Update the three call sites in `Frame::blend_at` to pass a `ChannelBlend` value.
  4. Run `cargo fmt --check` and the dev gate.

## Accepted / Not Requiring Fixes

- `Frame::filled(width, height, color)` has 3 parameters because plan R1 explicitly mandates that signature. This is a plan-mandated deviation from `max-arguments-per-method.md`.
- `expected_buffer_bytes` in `src/renderer/error.rs` duplicates the checked multiplication in `src/renderer/frame.rs::rgb24_byte_count`. Acceptable small duplication because `error.rs` is a base module and should not depend on `frame.rs` internals.
- Negative `i32` draw-callback offsets cast to `u32` in `src/renderer/text.rs` rely on defined wraparound and are then filtered by `Frame::is_inside_frame`. This is accepted per the recorded ruling on u32 draw-callback offset handling.

## Verification

After both fixes:

```bash
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```

Must report `ALL CHECKS PASSED` with 81 tests and exit 0.

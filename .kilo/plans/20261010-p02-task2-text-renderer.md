# Implementation Plan — Phase 02 Task 2: RGB24 Text Renderer

Sub-plan of `.agent/todos/20261010/20261010-todo-2.md` Task 2 (Critical Workflow 4.1b, sub-step 4.1b).
Branch: `feat/phase02-renderer-chat-store` (already checked out). Package v0.5.0, edition 2021, rust 1.82 container toolchain.
Executor: junior implementer — do NOT treat any instruction here as optional or as an invitation to redesign.

## Scope authority and hard boundaries

- Implement ONLY `.agent/todos/20261010/20261010-todo-2.md` **Task 2** (lines 39–50) and nothing else.
- **Do not touch**: `src/chat/**` (Task 1, closed), `src/app/**` (except the single `main.rs` module declaration described below), `src/config/**`, `config/config.example.json`, `docs/**`, `README.md`, `.agent/*` docs (Task 3), CLI/exit codes, `scripts/*`, Dockerfile, docker-compose.yml.
- The v0.5.0 version bump already happened — do not bump again.
- No FFmpeg, YouTube, OAuth, streaming, audio, scrolling, wrapping, font fallback, or truncation behavior.
- Forbidden: `panic`/`unwrap`/`expect` in normal paths, commented-out code, re-exports in `mod.rs`, `#[allow]` anywhere except the single `main.rs` `mod renderer;` annotation defined below.

## Mandatory reading list (before step 1)

1. `.agent/todos/20261010/20261010-todo-2.md` — full file (Task 2 + `Constraints and Out of Scope`).
2. `.agent/project-info/brief.md` §9.1–9.7 (renderer contract), §5 (config), §15.1 Renderer unit-test list.
3. `.agent/project-info/architecture.md` — `renderer/` row + Boundary Rules + Data Flow.
4. `.agent/project-info/context.md` — Recent Changes (2026‑10‑10) + Known Deviations/Limitations.
5. `src/chat/error.rs`, `src/chat/mod.rs`, `src/chat/store.rs` — the exact module/error/test style to mirror.
6. `src/config/model.rs` — `VideoConfig`, `RendererConfig` field names/types (renderer settings derivation source).
7. `src/main.rs` — Task‑1 `#[allow(dead_code)] mod chat;` pattern to replicate.
8. `.gitignore` — commit-hygiene check before every commit (`fonts/console.ttf` is untracked and must stay untracked).
9. `cargo fmt --check` / gate conventions: `scripts/dev-checks.sh` (read only; you run it, you do not modify it).

---

## Decisions encoded (implementer MUST NOT deviate)

### R1 — Trait and public surface

- File `src/renderer/renderer.rs` defines:
  - `pub struct RenderInput { pub text: String }` — exact brief §9.2 shape (complete visible multiline text; `\n`-separated).
  - `pub trait Renderer { fn render(&mut self, input: &RenderInput) -> Result<Frame, RenderError>; }`.
- `Frame` lives in `src/renderer/frame.rs`: `pub struct Frame { width: u32, height: u32, pixels: Vec<u8> }` (fields private).
  - Construction ONLY via `Frame::filled(width: u32, height: u32, color: Rgb) -> Result<Frame, RenderError>`: checks `width and height positive` and `width as usize * height as usize * 3` with `checked_mul` chain; overflow/negative → `RenderError::FrameSize { width, height }`. Fills every byte with the background `Rgb` triple.
  - Public accessors: `width() -> u32`, `height() -> u32`, `pixels() -> &[u8]`.
  - Module-internal paint surface: `pub(super) fn blend_at(&mut self, point: PixelPoint, blend: PixelBlend)` where `PixelPoint { x: u32, y: u32 }` and `PixelBlend { coverage: f32, color: Rgb }` (two struct params to honor the ≤2-args rule). It re-checks bounds against width/height and writes bytes at index `(y as usize * width as usize + x as usize) * 3`.
- `Rgb` value type lives in `src/renderer/settings.rs`: `pub struct Rgb { pub red: u8, pub green: u8, pub blue: u8 }` with `pub fn parse_hex(value: &str) -> Result<Rgb, RenderError>` and `Display` as `#RRGGBB` uppercase.
- `TextRenderer::new(settings: RendererSettings) -> Result<TextRenderer, RenderError>` — ONE argument. All validation (geometry, colors, font load) happens at construction. `render` returns errors only for genuine render-time failures (frame allocation overflow).
- `RendererSettings` (see R3) is the sole source of geometry; `TextRenderer` stores `settings: RendererSettings` and reads geometry from it. No cached frame buffer: each `render` call allocates a fresh `Frame` via `Frame::filled`.
- `src/renderer/mod.rs`: module doc block (mirrors `chat/mod.rs` wording, referencing brief §9.1–9.4 and the TODO Task 2 scope authority) + `mod error; mod frame; mod renderer; mod settings; mod text; mod text_tests;` declarations only. NO `pub use` re-exports (binary crate; re-exports return at Phase 03 wiring, per `chat/mod.rs` precedent). Inside the module tree access items directly (`frame::Frame`, `text::TextRenderer`); tests use `super::*`.

### R2 — Font crate and test font (ONE decision each)

**Crate: `ab_glyph = "0.2"`, lockfile pinned to version `0.2.28`.**

Justification (record verbatim in the commit + later Task 3 context.md):

- Actively maintained (releases through 2025; 0.2.28 Jul 2024; author also maintains `owned_ttf_parser` chain; 700k+ downloads/month; Debian-packaged).
- Pure Rust: no platform-installed graphics libraries, no fontconfig, no C deps (deps: `owned_ttf_parser`, `ab_glyph_rasterizer`). Works identically on Windows and Linux.
- License Apache-2.0 — permits dependency use.
- Published Jul 2024, predating edition-2024 crates; compatible with the pinned `rust:1.82` image (verified empirically by the container `cargo check` below — a hard blocker stop condition if it fails).
- Supersedes `rusttype` (maintenance mode, slower) and `fontdue` (slower-moving; API evolution risk).
- API shape fits the plan: `FontVec::try_from_vec`, `glyph_id`, `h_advance`, `outline_glyph(...).draw(|x, y, coverage| ...)` with coverage in `0.0..=1.0`.

**Test font: vendor ONE small SIL OFL 1.1 font, `Exo2-Light.otf`, into the repo with full provenance docs.** The local `fonts/console.ttf` is untracked with undocumented provenance — it must NOT be staged/committed and must NOT be referenced by tests.

- Download URL: `https://github.com/alexheretic/ab-glyph/raw/main/dev/fonts/Exo2-Light.otf` (same upstream project as the chosen crate; single face, ~100–200 KB — smallest defensible single-file asset, no font family bulk).
- Provenance + license documentation commit requirement: create `fonts/README.md` stating: source URL, exact download date (UTC), SHA-256 of the file (`Get-FileHash` on Windows host), full legal name ("Exo 2" Light, designer Natanael Gama), license SIL Open Font License 1.1 (redistribution permitted; embeddable; must ship the OFL text), and the operator-supplied-font note ("`fonts/console.ttf` and the configured real font are operator-supplied at runtime and are not versioned").
- Also save the complete OFL 1.1 license text as `fonts/OFL.txt` (canonical current copy URL: `https://raw.githubusercontent.com/alexheretic/ab-glyph/main/dev/fonts/Exo2-Light.otf.LICENSE` — if that 404s, use any official OFL 1.1 full-text copy obtained from the upstream repo and record the exact fetch URL in `fonts/README.md`).
- Tests reference the vendored font via `concat!(env!("CARGO_MANIFEST_DIR"), "/fonts/Exo2-Light.otf")` in a shared test helper constant inside `text_tests.rs`.
- Planner amendment A6 (factual correction after repository check): `fonts/console.ttf` IS already tracked (pre-phase operator asset committed in the original scaffold; provenance undocumented). Rules: do NOT stage, modify, or delete it in this task; vendored files are separate new adds; `git ls-files fonts/` after this task must list `console.ttf`, `Exo2-Light.otf`, `OFL.txt`, `README.md` only. The README operator-supplied-font note additionally records that `fonts/console.ttf` predates this phase and its provenance is indeterminate — operators should supply their own documented font (full treatment in Task 3 docs scope).
- Operator note (also recorded for Task 3, not implemented here): `config.example.json` keeps `fonts/console.ttf` as the example; the runtime font path exists only as a config contract — existence/parseability is checked by the renderer at construction with typed errors, per Phase 01 validation precedent (no file-existence check in config validation).
- Planner operational amendment A7 (download mechanics): the implementer performs the font + license downloads INSIDE the Compose container on the Alpine VM (curl ships in the rust:1.82 Debian image): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "curl -fsSL -o <path> <url>"` writing directly onto the bind mount; record UTC download date via container `date -u` and file SHA-256 via container `sha256sum <path>` (write these values into `fonts/README.md`). If the primary URL fails, try the fallback OFL copy per R2; if both fail, STOP with the exact errors as a blocker — never fabricate files.

### R3 — Rasterization geometry math (exact)

All math on `u32`/`usize` integers except ab_glyph's own `f32` placement, all deterministic:

- `PxScale = PxScale::from(settings.font_size as f32)`.
- Text splitting: `input.text.split('\n')` — NO trimming, NO filtering (brief: empty input → black frame; a line of only spaces yields no ink naturally).
- Line `i` baseline: `y = settings.top_margin + i * settings.line_height` (ab_glyph glyph `position` point is on the baseline). Computed with saturating u32 math then bounds-checked against the frame height; it can exceed the frame — vertical clipping is a no-ink-beyond-frame skip, NOT an error.
- Line start x: `settings.left_margin`. Per glyph advance: build `Glyph` via `font.glyph_id(ch)`, set scale, `font.outline_glyph(glyph.with_scale_and_position(scale, point(pen_x as f32, baseline_y as f32)))`; after drawing, `pen_x += font.h_advance(raw_glyph)`.
- Missing-glyph policy: if `font.glyph_id(ch)` maps to notdef (`glyph_id.0 == 0`), SKIP the character (advance still added) — ab_glyph maps unknown codepoints to `GlyphId(0)`; the font itself decides the notdef shape rendering otherwise. Documented as a renderer limitation for Task 3.
- Horizontal clip rule (encoded): the draw-area START offset is `left_margin` per line; glyphs are clipped ONLY at the physical frame pixel boundary — pixels with `x >= width` are skipped, so text longer than the frame is clipped at `x = width - 1` (right margin does NOT clip; right_margin is a start-offset config not applied as clip in MVP because brief §9.5 says clip at the frame boundary; margins respected = left/top margins + line spacing; note this deviation-comment in `text.rs` module docs).
- Coverage blending math (deterministic, no gamma): for pixel p with background `Rgb b` and text `Rgb t`, channel value = `background + ((coverage * (text - background)) + 0.5).floor() as u8`, i.e. per-channel round-half-up of linear interpolation. Colors equal ⇒ constant bytes ⇒ pure white stems on pure black background are exactly `(255,255,255)`.
- Paint filter: paint only when `coverage > 0.0` AND `x < width` AND `y < height`; `blend_at` re-checks bounds. Booleans split into named predicates per repo rule: `is_inside_frame(point) -> bool` on Frame, `has_influence(coverage) -> bool` = `coverage > 0.0` in `text.rs`.
- Single-section-boolean rule applies to the `pen_x > 0.0` / layout conditionals too: any compound condition must be extracted into a named helper function (e.g. `fn glyph_paint_targets(pen_x: u32, point: PixelPoint, ...) -> bool` style, chosen by implementer to keep bodies ≤ 50 lines).

### R4 — Error type `src/renderer/error.rs`

Mirror `src/chat/error.rs` exactly in style (hand-written Display, `std::error::Error` with `source()`, variant struct fields, no thiserror, `#[derive(Debug)]`):

```rust
pub enum RenderError {
    Geometry { width: u32, height: u32 },                    // non-positive frame dimensions at construction
    FrameSize { width: u32, height: u32 },                   // width * height * 3 overflow / unusable allocation
    Layout { height: u32, top_margin: u32, bottom_margin: u32, line_height: u32 }, // vertical layout leaves < 1 line
    InvalidColor { field: &'static str, value: String },     // field is "text_color" or "background_color"
    FontOpen { path: PathBuf, source: io::Error },           // missing/unreadable font file
    FontParse { path: PathBuf, source: ab_glyph::InvalidFont }, // file exists but is not a valid font
}
```

- Display contracts (write these exact shapes):
  - Geometry: `"renderer frame dimensions must be positive (width {width}, height {height})"`.
  - FrameSize: `"frame size of width {width} and height {height} does not fit a usable RGB24 buffer of {expected} bytes"`.
  - Layout: values echoed like `ChatError::Capacity` (`"renderer vertical layout must leave room for at least one line (height {height}, top_margin {top_margin}, bottom_margin {bottom_margin}, line_height {line_height})"`).
  - InvalidColor: `"renderer {field} '{value}' is not a valid #RRGGBB hex color (example #FFFFFF; '#' prefix plus six hex digits required)"`.
  - FontOpen: `"unable to open font file '{}': {}"` (path displayed, like LogOpen).
  - FontParse: `"unable to parse font file '{}': {}"` (include path; `source()` chains to the ab_glyph error).
- Keep the file ≤ 200 lines; it will exceed 125 "ideal" — acceptable per error-file precedent.

### R5 — Test scope (renderer portion ONLY of TODO Task 4)

Deterministic assertions, fraction/property-based, NO golden image files, ALL tests inside the renderer module (`#[cfg(test)] mod tests`-style files, like chat's `store_tests.rs`). Helper constants at top of `text_tests.rs`:

```rust
const WIDTH: u32 = 200; const HEIGHT: u32 = 200;      // small frame → fast tests
const TOP_MARGIN: u32 = 20; const LINE_HEIGHT: u32 = 40; /* left/bottom 20 */
const FONT_SIZE: u32 = 24;                            // cap-height ≈ 17px < line band
```

Test files and exact cases (`name` → core assertions):

`src/renderer/frame_tests.rs`
1. `frame_filled_has_exact_byte_count` — `Frame::filled(64, 24, Rgb::black())` length == 64*24*3 (4608) and `pixels().len()` matches; dims returned by accessors.
2. `frame_filled_is_uniform_color` — every consecutive rgb triple equals the fill color (iterate all pixels).
3. `frame_rejects_zero_or_overflowed_sizes` — `Frame::filled(0, 24, …)` and a pathological huge size (e.g. `(u32::MAX, 2, …)`) return `RenderError::Geometry`/`RenderError::FrameSize` respectively (match on variant; no panic).
4. `blend_at_within_bounds_only_paints_inside` — paint at `(width, 0)`/inside points; verify out-of-bounds paint is a no-op and inside paint changes exactly that triple.

`src/renderer/settings_tests.rs`
5. `settings_parse_mixed_case_hex_colors` — `#AbC012` parses to `Rgb {0xAB, 0xC0, 0x12}`; uppercase `#ABC012` equal (case-insensitive parse).
6. `settings_reject_invalid_color_shapes` — `#FFF`, `FFFFFF` (no `#`), `#GGHHII`, `#ABC0123` (7 digits), blank: each returns `InvalidColor` with the correct `field` value carried.
7. `settings_from_config_rejects_invalid_vertical_layout` — derive from a `RendererConfig`+`VideoConfig` where `top+bottom+line_height > height` → `RenderError::Layout` with values echoed (construct config structs in-place as literals — config validation module is NOT invoked).
8. `settings_from_config_rejects_zero_frame_dimensions` — width 0 → `Geometry`.

`src/renderer/text_tests.rs` (font path constant above; expected bands: line i ink confined to rows `[top_margin + i*line_height, top_margin + i*line_height + line_height)` intersected with frame)
9. `missing_font_file_returns_typed_error` — `RendererSettings` font path = nonexistent tempdir path → `TextRenderer::new` returns `FontOpen`, Display contains the path string.
10. `garbage_font_file_returns_parse_error` — temp file with `[u8::A_FAKE_HEADER; 64]`-style garbage bytes → `FontParse`.
11. `invalid_color_at_construction` — `TextRenderer::new` with `"#XYZ123"` → `InvalidColor` (proves colors validated at construction).
12. `empty_text_yields_completely_black_frame` — `render(&RenderInput { text: String::new() })`: length exact, EVERY pixel triple == background (iterate).
13. `multiline_orders_top_to_bottom` — text `"abcdefgh\nijklmnop"`: split ink pixels by image rows into the two bands; assert every ink pixel above-band equals zero for band 1 rows ABOVE the first band and band-1 ink exists inside band 0 (use a counting helper `count_ink_in_rows(frame, start, end)`); and at least one ink pixel in each band → proves order+placement.
14. `margins_take_effect` — first ink column of line 0 ≥ `left_margin` and all line-0 ink rows ≥ `top_margin` (scan helpers; Exo2 'a' has positive left side bearing).
15. `horizontal_clips_at_frame_edge` — text = `"a".repeat(400)` with width 200 (far beyond frame): assert ink exists in the rightmost column region (`count_ink_in_columns(frame, width - 1, width) > 0`) proving boundary clipping, and `render` succeeded (no wrap/truncation error).
16. `vertical_clips_at_frame_bottom` — text = `"a\n".repeat`-built 20 lines with HEIGHT 200 (only 4 bands fit): render succeeds, total ink is confined to rows `< height` (invariant helper over all pixels) and ink exists in the LAST 3 rows of the frame (partial-band clip proof) when font metrics permit; assert no panic and exact byte count.
17. `white_text_on_black_background_with_default_colors` — defaults `#FFFFFF`/`#000000`: assert pixel `(0,0)` is `(0,0,0)`; assert at least one pixel exists equal to exactly `(255,255,255)` inside a glyph's stem region (scan band 0 for a max-coverage pixel: helper `find_full_white_pixel`); assert every pixel satisfies `r == g == b` (grayscale monotone consequence of white-on-black).
18. `coverage_blend_is_neither_background_nor_text` — within band 0 there EXISTS a pixel with `0 < r < 255` (antialiased edge) — deterministic for Exo2 Light at 24 px.
19. `render_is_repeatable` — two consecutive `render` calls with the same input produce identical `pixels()` byte-slices (`assert_eq!`) — no hidden state mutation.

Assertions helpers (`count_ink_in_rows`, `count_ink_in_columns`, `find_pure_text_pixel`) stay ≤ 50-line bodies, private, in the test file.

### R6 — File layout, wiring, commits

Final `src/renderer/` file list (all ≤ 200 code lines; keep bodies ≤ 50, nesting ≤ 2):

| File | Contents |
| --- | --- |
| `mod.rs` | module docs + `mod` declarations (no re-exports) |
| `error.rs` | `RenderError` (R4) |
| `frame.rs` | `Frame`, `PixelPoint`, `PixelBlend`, blend math (R1/R3) |
| `settings.rs` | `Rgb`, `RendererSettings` (+ `from_config(config: &crate::config::model::RendererConfig, video: &crate::config::model::VideoConfig)` typed constructor, validation predicates, hex color parsing) |
| `renderer.rs` | `RenderInput` + `trait Renderer` (R1) |
| `text.rs` | `TextRenderer`, `new`, line advancing + glyph rasterization loop (R3) |
| `frame_tests.rs`, `settings_tests.rs`, `text_tests.rs` | R5 tests |

Cross-module referencing inside the tree: `frame::Frame`, `settings::RendererSettings`, etc. (no re-exports). Use full sibling `crate::config::model::RendererConfig` import path in `settings.rs` (crate-internal, verified pattern).

`src/main.rs` — append, mirroring the Task-1 A5 pattern (attribute and `mod` on split lines, no other change to the file):

```rust
#[allow(dead_code)]
mod renderer;
```

Annotated in place as Task 1 artifact: not wired into `app::run`; retires at Phase 03 app-wiring. Keep `mod chat;` untouched.

Dependency + lockfile steps (exact):
- `Cargo.toml`: add `ab_glyph = "0.2"` under `[dependencies]` (alphabetical order — first line of deps).
- Refresh/verify lock inside the container (Alpine VM MCP commands):
  1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo update -p ab_glyph --precise 0.2.28`
  2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked` → must exit 0; if edition-2024/MSRV errors appear from `owned_ttf_parser`/transitives at 1.82: STOP and return the exact error text as a blocker question — do NOT silently pick another version.

Commit plan (one feature per commit; explicit path `git add`, NEVER `git add -A`; verify `git status` shows no `fonts/console.ttf` or `logs/*` before each commit):
1. `feat(renderer): vendor Exo2-Light font with provenance and add ab_glyph` — `fonts/Exo2-Light.otf`, `fonts/OFL.txt` (or the obtained license copy), `fonts/README.md`, `Cargo.toml`, `Cargo.lock`.
2. `feat(renderer): add typed renderer errors and rgb24 frame type` — `src/renderer/error.rs`, `frame.rs`, `frame_tests.rs`, `src/renderer/mod.rs` with only `error`/`frame` declared, `src/main.rs` `mod renderer;`.
3. `feat(renderer): add settings validation with hex color parsing` — `settings.rs`, tests file, mod.rs decls extended.
4. `feat(renderer): render text as rgb24 frames via ab_glyph` — `renderer.rs` (trait + RenderInput), `text.rs`, `text_tests.rs`, mod.rs final.
(Commits 2–4 must each keep `cargo check --locked` green ideally; acceptable that inter-commit states have unused items flagged by clippy only at the end — final gate governs.)

Verification steps per Critical Workflow (implementer-run, Alpine VM MCP, exit codes authoritative):
- After each commit batch: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked`.
- Final: standard gate `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → all four PASS lines, `ALL CHECKS PASSED`, exit 0. Iterate on fmt/clippy failures (e.g. rustfmt split-attribute formatting, clippy pedantic-level warnings under `-D warnings`) until clean. Authoritative full-gate rerun happens in Task 4; the report is Task 4's deliverable.

Clippy-tightness notes to bake in from the start: `#[allow(dead_code)]` items ARE allowed by that attribute for the module gap only; every `pub` item in submodules that is currently unreferenced by other sibling modules is fine (same as chat); avoid `if x < width && coverage > 0.0` compound conditions (extract predicates); avoid needless `return`.

## High-level approach

Construction-time-validated renderer: `RendererConfig`+`VideoConfig` → `RendererSettings` (geometry + colors parsed/validated, font bytes loaded+parsed once into `FontVec`) → `TextRenderer::new`. `render(&RenderInput)` allocates a fresh background-filled `Frame` with checked arithmetic, lays out each `\n`-separated line at baseline `top_margin + i*line_height` starting at pen `x = left_margin`, rasterizes every glyph via ab_glyph coverage callbacks, blends coverage deterministically into RGB bytes with per-pixel frame-boundary clipping, and returns the invariant-checked `Frame` or a typed `RenderError`. The renderer stays a pure local component: no YouTube/FFmpeg/chat/history/scroll behavior, no silent fallbacks, no panics.

## Step list (ordered; verify after each)

1. Read the mandatory reading list; confirm baseline (`git status` clean, branch `feat/phase02-renderer-chat-store`, 62 tests currently green).
2. Cargo.toml + lockfile pin (R6 steps; blocker stop condition noted).
3. Font vendoring + provenance docs (R2; explicit `git add` hygiene).
4. `frame.rs` + `frame_tests.rs`; `cargo check --locked` in container.
5. `error.rs`; extend mod.rs; re-check.
6. `settings.rs` + `settings_tests.rs`; re-check.
7. `renderer.rs` trait + `text.rs` + `text_tests.rs`; extend mod.rs with all decls; re-check.
8. `main.rs` `mod renderer;` (R6 snippet).
9. Iterate `cargo fmt --check` / clippy in-container until clean; run the full gate once (exit 0).
10. Commits 1–4 with the exact messages above; after each `git status --porcelain` must list ONLY that commit's files.

### Review checklist (self/Planner)
- Every R1–R6 decision present? No judgment left? Scope fences intact (no chat/config/app filler edits)? Tests deterministic (no golden files, no platform dependence)? Errors typed and tested? `Cargo.lock` committed AND `--locked` resolvable? `fonts/console.ttf` never staged? No `#[allow]` besides `main.rs`?

## Verification against Task 2 wording (line 39–50)

- Trait + `TextRenderer` + typed errors, brief §9-matched → R1/R3/R4. ✔
- Input = complete visible text; renderer owns nothing else → R1 `RenderInput`. ✔
- Full frames, `width×height×3` bytes RGB24, checked arithmetic, actionable errors, no panics → R1 `Frame::filled` + `RenderError::FrameSize`. ✔
- Empty → all black; non-empty → black bg + white text, line order, configured font/size/line_height/margins → R5 tests 12/13/14/17. ✔
- Explicit font load, no fallback, typed testable missing/invalid-font errors → R2 + tests 9/10. ✔
- Horizontal clipping at frame boundary; no wrap/resize/truncation; margins respected; never out of buffer → R3 + tests 14/15. ✔
- Invalid geometry rejected without panics → tests 3/7/8. ✔
- Maintained cross-platform font crate justified; Cargo.toml/lock updated reproducibly → R2. ✔
- Font asset only with license+provenance documented → R2 vendoring + `fonts/README.md`/`OFL.txt`. ✔
- Color contract not silently dropped; no schema change; documented deviation for Task 3 → R2/R3 (+ hex parsing tests 5/6/11). ✔

## Amendments (2026-10-10, Planner rulings after 4.2 blocker report)

### A6/A7 — applied (already in the vendoring instructions above)

Accepted as executed: ab_glyph resolved via plain `cargo check` to 0.2.32 then pinned DOWN to 0.2.28 with `cargo update -p ab_glyph --precise 0.2.28` (exact `Downgrading ab_glyph v0.2.32 -> v0.2.28` + owned_ttf_parser 0.24.0, ttf-parser 0.24.1; `cargo check --locked` exit 0 on Rust 1.82). The generic-curl-based OFL canonical URL 404'd (exit 22, curl: (22)); accepted fallback: `fonts/OFL.txt` (Exo 2 family OFL 1.1 text) from `https://raw.githubusercontent.com/google/fonts/main/ofl/exo2/OFL.txt` — recorded in fonts/README.md with its own SHA-256 (`7f95a8eeecc6904ffc81cd10cc8ebc7fcee72a67922924bf45f02dff03b88127`) and UTC date. Font: `fonts/Exo2-Light.otf`, 111052 bytes, SHA-256 `a4bbe688531db7853383649e17b67d7ded1d3a2492f5c3f44f715a6a3919b9ac`, source `https://github.com/alexheretic/ab-glyph/raw/main/dev/fonts/Exo2-Light.otf`, 2026-10-11 UTC.

### A8 — R3 line-box baseline (blocker ruling: option A)

- The literal R3 formula `baseline_i = top_margin + i*line_height` places all line-0 ink ABOVE `top_margin` (glyphs draw upward from the baseline), violating the TODO's "Respect left/top margins" (top authority) and making R5 tests 13/14/16 impossible as written.
- Amended R3 (final): line-box placement — `baseline_i = settings.top_margin + scale_font.ascent() + i * settings.line_height`, using ab_glyph's `scale_font.ascent()` (px-space ascent at the configured scale); pen x still starts at `settings.left_margin`; per-pixel vertical/horizontal clipping at the physical frame boundary stays exactly as R3 encoded; NOT-YET-USED `bottom_margin` keeps its config-only role (documented deviation in `text.rs` docs per R3).
- No R5 test changes (bands [top_margin + i*line_height, +line_height), margin assertion, last-3-rows clip test all valid under line-box placement).
- Everything else in R1–R6 unchanged.

### A9 — module-inception ruling (blocker from resumed 4.2 report; decision B)

- `clippy::module_inception` rejects plan-literal `mod renderer;` inside module `renderer` under `-D warnings`. Chosen fix: in `src/renderer/mod.rs` declare `#[path = "renderer.rs"] mod contract;` (path attribute keeps the R6 file name and every commit-4 file list byte-identical; only the internal module identifier changes, with import-line updates in `text.rs`/`text_tests.rs`). Consumers inside the tree use `contract::Renderer` / `contract::RenderInput`.
- No `#[allow]` introduced (plan boundary intact). mod.rs doc comment notes the path-attribute mapping. All other R1–R6 + A6–A8 items unchanged.

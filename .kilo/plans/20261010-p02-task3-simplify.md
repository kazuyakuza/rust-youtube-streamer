# Plan — Phase 02, Task 3: Docs/Metadata Simplification (sub-step 4.3 outcome)

Scope authority: `.agent/todos/20261010/20261010-todo-2.md` §3 + `## Constraints and Out of Scope`; Task 3 plan `.kilo/plans/20261010-p02-task3-docs-metadata.md` (M4 checklist governs). Review covered commits `30dcdc4`, `051966d`, `70bec50`. Simplification applies ONLY to the NEW Phase 02 content in the files below. Docs-only; no code, no prior-phase history, no report file.

## Fact verification result (no findings — do not touch)

All numeric and factual claims in the new content verified against the tree:
- 16 chat unit tests (`store_tests.rs` 11 + `model_tests.rs` 5), 19 renderer (`frame_tests.rs` 4 + `settings_tests.rs` 4 + `text_tests.rs` 11), 81 total across `src/`.
- `Cargo.toml` v0.5.0, `ab_glyph = "0.2"` locked 0.2.28.
- `src/main.rs` mod list + `#[allow(dead_code)]` on `chat`/`renderer` exactly as documented.
- `ChatError { Capacity, LogOpen, LogAppend }`, `RenderError { Geometry, FrameSize, Layout, InvalidColor, FontOpen, FontParse }` match `error.rs`.
- Log record `format!("{seconds} {author}: {text}\n")` matches the documented `<epoch_seconds> <author>: <text>` single-space claim.
- Baseline `top_margin + ascent + i * line_height`, pen at `left_margin`, uppercase `#{:02X}` `Rgb` `Display` all match code.
- M4 verbatim sentences intact: README line 48; README line 52 FFmpeg sentence; docs/configuration.md lines 99–100 and line 76 final FFmpeg sentence; exit-code lines.
- `context.md` (both new sections + Next Steps item 1), `tech.md` new bullet, README new paragraph: reviewed, consistent, guard-weighted — **NO CHANGES** (any edit there risks the M4 false-claim guards or documented-deviation disclosures).

## Edits (execute in order; each is one exact-string replacement, nothing else)

### Step 1 — `.agent/project-structure.md` line 21 (chat parent bullet): drop dead_code duplicate

The `#[allow(dead_code)]` retirement fact currently appears 3× (lines 21, 27, 41). Line 41 (`src/main.rs` bullet) is the exact location and stays authoritative; lines 21/27 keep only the wiring status.

Replace exactly:

`- chat/ - application-level chat message model and bounded visible-message queue (ChatStore); independent of YouTube API types, network, streaming and FFmpeg (implemented, not yet wired into app::run; the `#[allow(dead_code)]` on `mod chat;` in `src/main.rs` retires at the Phase 03 wiring step)`

with exactly:

`- chat/ - application-level chat message model and bounded visible-message queue (ChatStore); independent of YouTube API types, network, streaming and FFmpeg (implemented, not yet wired into app::run)`

### Step 2 — `.agent/project-structure.md` line 27 (renderer parent bullet): same trim

Replace exactly:

`- renderer/ - converts RenderInput to complete raw RGB24 frames (width * height * 3 bytes, row-major, R/G/B); independent of YouTube API types, chat state, network and FFmpeg (implemented, not yet wired into app::run; the `#[allow(dead_code)]` on `mod renderer;` in `src/main.rs` retires at the Phase 03 wiring step)`

with exactly:

`- renderer/ - converts RenderInput to complete raw RGB24 frames (width * height * 3 bytes, row-major, R/G/B); independent of YouTube API types, chat state, network and FFmpeg (implemented, not yet wired into app::run)`

Line 41 (`src/main.rs`) is NOT touched; it retains the `#[allow(dead_code)]` + Phase 03 retirement claim for both modules.

### Step 3 — `.agent/project-structure.md` line 26: align test-file convention with line 18

Existing convention (line 18): `... unit tests (temp files via \`tempfile\`; ...)`. Bring the new chat test line inside that convention.

Replace exactly:

`  - `model_tests.rs` / `store_tests.rs` - 16 unit tests (display/normalization/metadata; capacity, FIFO eviction, append-only log ordering and I/O-failure semantics) via temp files (`tempfile`)`

with exactly:

`  - `model_tests.rs` / `store_tests.rs` - 16 unit tests (display/normalization/metadata; capacity, FIFO eviction, append-only log ordering and I/O-failure semantics; temp files via `tempfile`)`

### Step 4 — `docs/configuration.md` line 57 (renderer.font bullet): fix append punctuation seam

The sentence appended in Task 3 created a dangling `. — ` seam. Replace exactly:

`Example: `fonts/console.ttf`. — runtime font must be a parseable font file;`

with exactly:

`Example: `fonts/console.ttf`. Runtime font must be a parseable font file;`

Rest of the bullet untouched (zero claim change: only the seam dash is removed and one word capitalized).

### Step 5 — `docs/configuration.md` line 76: drop repeated-load filler word

Replace exactly:

`(no file-existence or parseable-font check at configuration load itself — this follows the Phase 01 validation precedent, and the renderer is wired into the runtime in a later phase)`

with exactly:

`(no file-existence or parseable-font check at configuration load — this follows the Phase 01 validation precedent, and the renderer is wired into the runtime in a later phase)`

## HARD MUST-NOT-CHANGE list (reject any edit touching these)

1. README.md line 48 placeholder paragraph — verbatim, untouched.
2. README.md line 50 (new Phase 02 paragraph + chat-log sentence) — untouched (guard-bearing, plan-verbatim).
3. README.md line 52 "FFmpeg remains a separately provided external prerequisite — it is not installed by this phase and this phase does not launch it." — verbatim.
4. docs/configuration.md lines 99–100 (`not implemented yet` mode descriptions) — verbatim.
5. docs/configuration.md line 76 final sentence "FFmpeg itself is a separately provided external prerequisite — this phase neither installs nor launches it." — verbatim (Step 5 ends before it).
6. Every number: 16, 19, 81, 0.5.0, 0.2/0.2.28, `width * height * 3`, exit codes 0/1/2/3, gate log `20261011T184938Z`, line/margin/baseline formulas.
7. Every documented deviation disclosure: epoch-second log format vs brief §11.2, right/bottom margin config-only, notdef skip-with-advance, OFL fallback URL, `console.ttf` indeterminate provenance, container-only verification.
8. All prior-phase context.md history (Earlier Recent Changes sections, Phase 01 deviations, Notes "brief.md is the source of truth"), architecture.md, brief §11.2, the TODO file.
9. No commit, push, gate run, or TODO checkbox flip inside this plan; it is 2 file edits only.

## Verification after edits

1. `git diff` shows changes ONLY in `.agent/project-structure.md` (3 lines) and `docs/configuration.md` (2 lines); nothing else.
2. Grep sanity: `#[allow(dead_code)]` appears exactly once inside `.agent/project-structure.md` (the line-41 bullet); `16 unit tests` and `19 unit tests` still present; `. — runtime font` no longer matches; `configuration load itself` no longer matches.
3. Confirm MUST-NOT-CHANGE items 1–5 byte-identical to HEAD.

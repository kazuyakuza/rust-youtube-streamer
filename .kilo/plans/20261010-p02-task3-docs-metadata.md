# Plan — Phase 02, Task 3: Documentation and Project Metadata (sub-step 4.1b)

Task authority: `.agent/todos/20261010/20261010-todo-2.md` §3 plus `## Constraints and Out of Scope` (TODO wins over this plan). Branch: `feat/phase02-renderer-chat-store` (continue, no new branch). Baseline: v0.5.0, tree clean, 81 tests, dev-checks log `logs/checks/20261011T184938Z.log` exit 0. Tasks 1–2 are `[DONE]`; chat and renderer modules exist and compile (both are `#[allow(dead_code)] mod`s in `src/main.rs`), but are NOT wired into `app::run`.

## Reading List (executor must read before editing)

1. `.agent/todos/20261010/20261010-todo-2.md` — §3 and `## Constraints and Out of Scope` verbatim.
2. `README.md` (current, v0.5.0 tree).
3. `docs/configuration.md` (current).
4. `.agent/project-structure.md` (current).
5. `.agent/project-info/context.md` (current, full).
6. `.agent/project-info/tech.md`, `.agent/project-info/architecture.md`.
7. `.agent/project-info/brief.md` — rows §8 (chat), §9 (renderer), §11.2 (chat log example).
8. `fonts/README.md` (provenance facts to reference, not copy wholesale).
9. `src/chat/{mod,model,store,error}.rs`, `src/renderer/{mod,renderer,frame,settings,text,error}.rs`, `src/main.rs`, `Cargo.toml` (facts only — no code changes).
10. Confirm on the tree: `docs/authentication.md` does NOT exist (verified 2026-10-11: `docs/` holds `configuration.md`, `build.md`, `build-windows.md`, two how-to guides). Do not invent it.

## High-Level Approach

Docs-only task. Four edited surfaces, three commits, one gate run at the very end:

1. `.agent/project-structure.md` — replace the two placeholder chat/renderer bullets with real per-file maps matching the existing one-line style; refresh `fonts/`, `Cargo.toml`, and `src/main.rs` root-file entries.
2. `.agent/project-info/context.md` — append a dated "Recent Changes" section for Phase 02 Tasks 1–2, a new "Known Deviations and Limitations (Phase 02, Tasks 1–2)" section, and refresh "Immediate Next Steps" item 1. Never touch prior phases' entries.
3. README.md — one paragraph + one bullet in `Current Status`; identical guard sentences to TODO constraints.
4. `docs/configuration.md` — one sentence correction in the Field Reference closing paragraph (font existence check) and a provenance note for `fonts/console.ttf` under the `renderer.font` field.
5. `tech.md` — add one dependency bullet (ab_glyph). `architecture.md` — no change needed (module-map rows already match the implementation; "For current repo state, see `context.md`" already defers).

Forbidden content boundaries (apply everywhere; verbatim false-claim guards):
- Never write that `ffmpeg`, streaming, OAuth, YouTube API calls, broadcast creation, network chat listening, audio, or `app::run` orchestration exist. Chat/renderer are src-level, unwired components only.
- Preserve these sentences verbatim (do not weaken or delete):
  - README.md line 48: "Both `auth` and `run` are placeholders until later phases: after configuration is loaded and validated, each prints a "not implemented yet" error and exits with code 3; neither contacts YouTube, requests credentials, creates broadcast resources, or starts FFmpeg."
  - docs/configuration.md lines 99–100 (the two `not implemented yet` mode descriptions).
- Exit codes 0/1/2/3 semantics unchanged in every edited doc.
- No speculative/future files in project-structure.md (no invented `youtube/` or `streaming/` contents).
- Brief is source of truth; the documented epoch-second log-format deviation from brief §11.2 must be flagged as a deviation, never presented as conformance, and brief §11.2 must not be rewritten.

---

## Step 1 — Update `.agent/project-structure.md`

### 1.1 Replace line 21 (chat bullet)

Delete:
`- chat/ - chat message model and bounded visible-message queue (ChatStore) (placeholder; future phase)`

Insert this block (exact wording; one line per file mirroring existing style):

```
- chat/ - application-level chat message model and bounded visible-message queue (ChatStore); independent of YouTube API types, network, streaming and FFmpeg (implemented, not yet wired into app::run; the `#[allow(dead_code)]` on `mod chat;` in `src/main.rs` retires at the Phase 03 wiring step)
  - `mod.rs` - module docs: log-append-first contract and scope authority (TODO §1); private submodule declarations (re-exports return at the first external consumer)
  - `model.rs` - `ChatMessage { author, text, message_id: Option, timestamp: Option<u64> }`; builder attachers for optional API metadata; `Display` is exactly `author: text` (metadata never visible); newline normalization flattens line breaks to one space
  - `store.rs` - `ChatStore`: `VecDeque` FIFO of visible messages, log file opened eagerly in append/create mode; `accept` appends the log record FIRST and rejects the message on I/O failure (queue and log never diverge); evicts oldest beyond capacity; `visible_text` joins `author: text` lines with `'\n'`; new stores start empty, the log is never read back
  - `error.rs` - `ChatError { Capacity, LogOpen, LogAppend }`; hand-written `Display` contracts, `std::error::Error::source`, no panics
```

(Executor: final wording may be tuned within these file boundaries; keep one line per file.)

### 1.2 Replace line 22 (renderer bullet)

Delete:
`- renderer/ - converts RenderInput to raw RGB24 video frames; no YouTube/FFmpeg dependency (placeholder; future phase)`

Insert this block:

```
- renderer/ - converts RenderInput to complete raw RGB24 frames (width * height * 3 bytes, row-major, R/G/B); independent of YouTube API types, chat state, network and FFmpeg (implemented, not yet wired into app::run; the `#[allow(dead_code)]` on `mod renderer;` in `src/main.rs` retires at the Phase 03 wiring step)
  - `mod.rs` - module docs; `#[path = "renderer.rs"] mod contract;` mapping keeps the trait out of `clippy::module_inception`
  - `renderer.rs` - `RenderInput { text }` (complete visible multiline text) and the `Renderer` trait (`render -> Result<Frame, RenderError>`); validation happens at construction, `render` fails only on frame-allocation overflow
  - `settings.rs` - `RendererSettings::from_config` (re-checks positive dimensions, vertical layout ≥ 1 line, parses `#RRGGBB` `Rgb` case-insensitively) and `Rgb` with uppercase `Display`; no file-existence check (Phase 01 precedent)
  - `frame.rs` - `Frame` with checked `width * height * 3` construction (`filled`), `blend_at` no-op outside the frame, deterministic round-half-up coverage blending
  - `text.rs` - `TextRenderer` (ab_glyph `FontVec`, font parsed once at construction, missing/unparseable fonts are typed errors, no silent fallback); line-box placement baseline = `top_margin + ascent + i * line_height`; per-pixel clipping at the physical frame boundary, horizontal cut at `x == width`, no wrapping/font resizing; unknown codepoints render as notdef, skipped with advance kept
  - `error.rs` - `RenderError { Geometry, FrameSize, Layout, InvalidColor, FontOpen, FontParse }`; hand-written `Display`, `source()`, no panics
  - `*_tests.rs` / `text_test_helpers.rs` - 19 unit tests (`#[cfg(test)] #[path]` in-tree test modules; font fixture `fonts/Exo2-Light.otf`)
```

Chat module tests: mention once inside the chat block, e.g. after `error.rs`: `model_tests.rs` / `store_tests.rs` one-line entry — `16 unit tests` via temp files (`tempfile`).

### 1.3 Update the fonts/ bullet (line 43)

Delete:
`- fonts/ - font files used by the renderer`

Insert:
`- fonts/ - font assets: `Exo2-Light.otf` + `OFL.txt` (SIL Open Font License 1.1; vendored deterministic renderer-test fixture with provenance and SHA-256 recorded in `fonts/README.md`) and the pre-existing tracked `console.ttf` (provenance indeterminate, referenced by no test; the configured runtime font path remains operator-supplied)`

### 1.4 Update the Cargo.toml root-file bullet (line 27)

Delete:
`- Cargo.toml - Rust package manifest: `rust-youtube-streamer-service` v0.4.0, edition 2021; deps: serde (derive), serde_json, tracing, tracing-subscriber (env-filter); dev-dep: tempfile`

Insert:
`- Cargo.toml - Rust package manifest: `rust-youtube-streamer-service` v0.5.0, edition 2021; deps: ab_glyph (renderer font rasterization), serde (derive), serde_json, tracing, tracing-subscriber (env-filter); dev-dep: tempfile`

### 1.5 Update the src/main.rs root-file bullet (line 29)

Delete:
`- src/main.rs - entry point delegating to the `app` layer (`mod app; mod config;`, no lint suppressions)`

Insert:
`- src/main.rs - entry point delegating to the `app` layer (`mod app; mod chat; mod config; mod renderer;`, with deliberate `#[allow(dead_code)]` on the not-yet-wired `chat` and `renderer` modules, retiring at Phase 03 wiring)`

### 1.6 No other changes

- `app/`, `config/` entries untouched; `youtube/` and `streaming/` bullets stay as "placeholder; future phase".
- No speculative future files/config fields anywhere.

## Step 2 — Update `.agent/project-info/context.md`

Anchor points (current file = 118 lines):

### 2.1 Append "Recent Changes (2026-10-11, Phase 02 Tasks 1–2)" 

Insert AFTER the existing `## Recent Changes (2026-10-10, Phase 01.2 implementation)` section (its last line is the Task-4 bullet ending ...) and BEFORE `## Known Deviations and Limitations (Phase 01)`. Content (bulleted, same bullet style as prior sections; final wording may be tuned by the executor within these bullets' boundaries):

- Branch `feat/phase02-renderer-chat-store`; version already at `0.5.0` this phase; tasks 1–2 executed via the Critical Workflow.
- Task 1 — `src/chat/`: `ChatMessage` (author/text normalized to one line, optional `message_id`/epoch-second timestamp kept off-screen, `Display` = `author: text`) plus `ChatStore` (bounded `VecDeque` FIFO; visible capacity derived with overflow-safe `u64` floor(`(height − top_margin − bottom_margin) / line_height`), minimum one line required; log opened eagerly append/create; `accept` appends the log record FIRST and a failed append rejects the message from both log and queue; new stores start with an empty queue and never read the log back). Typed `ChatError { Capacity, LogOpen, LogAppend }`. Module documented as independent of YouTube/streaming/FFmpeg; `#[allow(dead_code)] mod chat;` in `src/main.rs` deliberate with a Phase 03 retirement plan.
- Task 2 — `src/renderer/`: `RenderInput` + `Renderer` trait (declared as `contract` via `#[path = "renderer.rs"]`; plan amendment analog), `RendererSettings::from_config` (re-validates geometry/vertical layout, parses `#RRGGBB` `Rgb`), `Frame` (checked `width * height * 3` byte RGB24 buffer, deterministic round-half-up coverage blending, off-frame blend no-op), `TextRenderer` (ab_glyph 0.2.28; font loaded and parsed exactly once at construction; line-box baseline `top_margin + ascent + i * line_height`; pen starts at `left_margin`; horizontal clipping at `x == width` with no wrapping/font resizing; vertical ink past frame bottom dropped; empty input yields the untouched background frame). Typed `RenderError { Geometry, FrameSize, Layout, InvalidColor, FontOpen, FontParse }`. Right/bottom margins are config-only in the MVP renderer (documented deviation). Unknown codepoints skip drawing but keep advance.
- Fonts: vendored `fonts/Exo2-Light.otf` + `fonts/OFL.txt` (SIL OFL 1.1; SHA-256 and provenance in `fonts/README.md`; downloaded inside the Compose container); `fonts/console.ttf` predates this phase, tracked, provenance indeterminate, referenced by no test; the runtime font path remains operator-supplied.
- Dependency: `ab_glyph = "0.2"` (locked 0.2.28) added to `Cargo.toml`/`Cargo.lock` — pure-Rust rasterization, no system graphics libraries, Rust 1.82/edition-2021 compatible, Windows+Linux, justifyable per the no-`google-youtube3` dependency policy.
- Tests: 16 chat + 19 renderer unit tests; full standard gate via the Alpine VM MCP `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` exit 0 (`ALL CHECKS PASSED`, 81 tests, log `logs/checks/20261011T184938Z.log`).
- Task 3 (this change): project-structure.md per-file maps for `src/chat/` and `src/renderer/`, fonts/ and root-file refreshes; README Current Status paragraph; docs/configuration.md font-existence/provenance corrections; tech.md dependency bullet; context.md itself.
- No wiring into `app::run`, no config/schema expansion, no YouTube/FFmpeg/OAuth/streaming behavior added (TODO constraints honored).

### 2.2 Add "Known Deviations and Limitations (Phase 02, Tasks 1–2)"

Insert AFTER the existing `## Known Deviations and Limitations (Phase 01)` section, BEFORE `## Immediate Next Steps`. Bullets:

* Chat log record format is `<epoch_seconds> <author>: <text>\n` — a deliberate deviation from brief §11.2's RFC3339-style example (simpler, timezone-free, losslessly ordering); flagged here, brief not rewritten.
* Renderer MVP drops ink beyond the physical frame bottom and treats `right_margin`/`bottom_margin` as start-offset/config-only values (brief §9.5 clipping intent preserved horizontally; documented deviation).
* Unknown/unsupported codepoints are skipped (notdef, `GlyphId(0)`) while their horizontal advance is still consumed (visible-gap limitation).
* Visible capacity = floor((height − top_margin − bottom_margin) / line_height), minimum 1 (fractional remainder rounds down; no `max_messages` config field was added).
* `#[allow(dead_code)]` on `mod chat;` / `mod renderer;` in `src/main.rs` is deliberate while the modules wait for Phase 03 `app::run` wiring; it retires then.
* The OFL license copy shipped is the Google Fonts Exo 2 OFL text (ab_glyph repo's canonical upstream URL 404'd — fallback documented in `fonts/README.md`).
* `fonts/console.ttf` provenance is indeterminate; operators supply a documented font at runtime; tests use only `fonts/Exo2-Light.otf`.
* Verification remains container-only (Alpine VM); rendering is validated by unit tests, not by any native Windows/Linux run — no native renderer behavior is claimed.

### 2.3 Refresh "Immediate Next Steps"

Update item 1 (currently "After Phase 01.2 is complete, re-review ... proceed to Phase 02 — Renderer and Chat Store.") to: "Phase 02 Tasks 1–2 are complete; Tasks 3–4 (this metadata task, then tests/builds/final gate and the tracked completion report at `.agent/reports/20261010-phase-02-renderer-chat-store.md`) close the phase. Phase 03 will wire `chat`/`renderer` into `app::run` (retiring the `#[allow(dead_code)]` mods) and start the FFmpeg process pipeline." Items 2–4 stay verbatim.

### 2.4 Preserve everywhere else

No edits to Any earlier "Recent Changes" section, "Scope Decisions", or "Notes" (the `brief.md` wins note stays).

## Step 3 — README.md (minimal update)

### 3.1 Current Status section

Keep lines 40–52 intact except: after the paragraph block about Phase 01 (line 46 ends the bullet list; line 48 is the verbatim placeholder sentence), insert ONE new paragraph between line 48's paragraph and the current line-50 FFmpeg sentence (i.e., after the placeholder paragraph, before "FFmpeg remains a separately provided..."):

"Phase 02 adds two independent src-level components on this foundation: a bounded in-memory chat store (`src/chat/`) that appends accepted messages to an independent append-only chat log and keeps only the currently visible messages in a FIFO queue, and an RGB24 text renderer (`src/renderer/`, ab_glyph-based) that turns complete visible multiline text into raw video frames — both implemented with typed errors and unit tests but not yet wired into the `run` pipeline; neither changes any CLI behavior or the placeholders below."

Then append to the existing line-52 area one sentence (or a short follow-up sentence inside that paragraph): "The chat store's chat log (config `chat.log_file`) is created only when the (not yet wired) chat store component is used."

Rule check: the added text must contain NO claim of streaming/rendering behavior in the running executable; it must keep the exact placeholder paragraph (line 48) untouched.

### 3.2 No other README edits

TOC unchanged (no new headings). `Key Technologies` gets no Tokio claims already present; leave as is. `Commands and Configuration`, `Logging`, build sections untouched.

## Step 4 — docs/configuration.md (only where descriptions changed)

### 4.1 Field Reference closing paragraph (line 76)

Current sentence says font existence is "checked by the consuming component in a later phase". The consuming component (the renderer) now exists src-level. Replace that sentence with:

"Existence of the font file or the FFmpeg executable is checked by the consuming component, not at configuration load: the renderer opens and parses the configured font at construction with typed errors (no file-existence or parseable-font check at configuration load itself — this follows the Phase 01 validation precedent, and the renderer is wired into the runtime in a later phase). FFmpeg itself is a separately provided external prerequisite — this phase neither installs nor launches it."

The final FFmpeg sentence must be preserved verbatim.

### 4.2 renderer.font bullet (line 57)

Append to the bullet: " — runtime font must be a parseable font file; the renderer validates it at construction (typed `FontOpen`/`FontParse` errors). Test fixture `fonts/Exo2-Light.otf` ships under the SIL Open Font License 1.1 (`fonts/OFL.txt`); tracked `fonts/console.ttf` predates the renderer, has indeterminate provenance, and is referenced by no test — operators should point `font` at a documented, licensed font file."

### 4.3 Untouched

`chat.log_file` bullet, exit codes line 103, both mode bullets 99–100 (verbatim), all other sections. No schema claims change (no new config fields exist).

## Step 5 — `.agent/project-info/tech.md` (planner-managed correction)

Add ONE bullet at the END of the `## Dependencies (policy)` section (after the "Exact crate selection happens during implementation, not now." bullet):

"* Decided during Phase 02 (renderer): `ab_glyph = "0.2"` for font rasterization — pure Rust, no platform-installed graphics libraries, Rust 1.82/edition-2021 compatible, Windows+Linux, pinned via the tracked lockfile; replaces nothing and is independent of YouTube/streaming crates."

No other tech.md edits (Stack/External Processes/Configuration/Platforms/Authentication Modes remain accurate).

`.agent/project-info/architecture.md` — NO changes: the `src/chat/` row (bounded FIFO of visible messages) and `src/renderer/` row (RenderInput → raw RGB24 frames, trait contract, no YouTube dependency) already match the implementation; data flow/boundary rules unchanged; "For current repo state, see `context.md`" already routes current state to context.md.

## Step 6 — Workflow steps (4.3–4.6) for this task

- 4.3 — code-reviewer/code-simplifier pass scoped EXCLUSIVELY to doc accuracy: verify every factual claim in the edited files against TODO §3, the actual `src/chat` + `src/renderer` facts (double-space log separation, 16/19 test counts, ab_glyph 0.2.28, gate log `20261011T184938Z`, 81 tests) and the false-claim guards (M4 list below). Flag anything unverifiable; docs-simplifier may merge redundant wording but must preserve the verbatim sentences.
- 4.4 — docs-specialist: NOT REQUIRED as a code pass (no source code or comments change in this task — justification recorded in the task log). The docs-specialist MAY optionally polish the newly added context.md/structure.md prose within the drafted bullet boundaries if the 4.3 review flags wording issues; that is the only trigger.
- 4.5b — N/A here (the tracked completion report is Task 4's deliverable at `.agent/reports/20261010-phase-02-renderer-chat-store.md`; do NOT create it now).
- 4.6 — after the final gate in Step 7 passes, mark the TODO heading `### 3. Documentation and Project Metadata` → `### 3. Documentation and Project Metadata [DONE]` (edit the TODO file, commit; leave Task 4 and all acceptance-criteria checkboxes for Task 4). Precedent: per-task `[DONE]` marks, acceptance flips at phase close.

## Step 7 — Verification and Commits

### 7.1 Verification

- Docs-only: no `cargo` formatting impact expected. Run the standard gate ONCE at the end via the Alpine VM MCP:
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
  Expect exit 0, 81 tests, ALL CHECKS PASSED (the authoritative re-run stays with Task 4).
- Manual doc self-checks (docs-specialist-style): TOC anchors in README/docs-configuration still resolve (no headings removed); project-structure.md reflects ACTUAL files (cross-check against `src/chat/*` and `src/renderer/*` glob: exactly `error,model,mod,store(+tests)` and `render(frame,error,mod,renderer,settings,text(+tests/helpers))`); no unverifiable claims (compare each numeric claim to real logs/files).

### 7.2 Commits (ordered, messages exact; stage only the listed files; run `git status` and check `.gitignore` compliance before each commit)

1. `docs: map phase 02 chat and renderer modules in project structure` — `.agent/project-structure.md`.
2. `docs: record phase 02 tasks 1-2 in context` — `.agent/project-info/context.md`.
3. `docs: note phase 02 components in README and configuration reference` — `README.md`, `docs/configuration.md`, `.agent/project-info/tech.md`.
4. `chore: mark phase 02 task 3 done` — `.agent/todos/20261010/20261010-todo-2.md` (after Task-3 gate passes).
- No packaging changes (Cargo.toml/Cargo.lock untouched by this task). No push in this sub-task (4.1b = plan execution ends before the caller's gate/push steps; if the caller's runner handles push, it targets `origin` only per the remote-safety rule).

## M4 — Verbatim Boundary Checklist (4.3 review input)

- Forbidden claims: no ffmpeg/OAuth/streaming/broadcast/audio/orchestration exists; renderer/chat are unwired src-level components.
- Exit codes unchanged: 0 help/success, 1 startup failure, 2 usage, 3 not-implemented.
- Preserved verbatim: README line 48; docs/configuration.md lines 99–100; README line 50 + configuration.md line 76's final "separately provided external prerequisite" sentence; context.md Notes "brief.md is the source of truth" line.
- Deviation disclosure required (not silent): epoch-second log timestamp vs brief §11.2 RFC3339 example; right/bottom margin clipping; notdef skip-with-advance; OFL fallback URL; console.ttf provenance.

## Out of Scope (hard)

No code changes, no TODO-only-plus-listed-files edits, no report creation (Task 4), no `app::run` wiring, no config/schema changes, no `.gitignore`/Dockerfile/script edits, no push in this sub-task.

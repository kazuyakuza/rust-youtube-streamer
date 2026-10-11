# Implementation Plan — Phase 02, Task 4: Tests, Builds and Completion Report

- Date: 2026-10-11
- Sub-step: Task 4, sub-step 4.1b (Analysis and Planning → implementation plan)
- Scope authority: `.agent/todos/20261010/20261010-todo-2.md`, section `### 4. Tests, Builds and Completion Report` + `## Acceptance Criteria` + `## Constraints and Out of Scope` (TOP PRIORITY). This plan never overrides the TODO.
- Global plan: `.kilo/plans/20261010-phase-02-renderer-chat-store.md` (Task 4 row 6; non-front-end → 4.1b + 4.5b only).
- Generator plans consumed: `.kilo/plans/20261010-p02-task1-chat-model-store.md` (D5 test spec) and `.kilo/plans/20261010-p02-task2-text-renderer.md` (R5 test spec).
- Branch: `feat/phase02-renderer-chat-store` (exists; HEAD `83b16e0` observed clean at planning time; package v0.5.0). Do NOT create/switch branches.
- Target implementer: JUNIOR developer under 50% restriction — every decision below is final; no judgment calls delegated.
- Explicitly OUT of this sub-task: git push (step 5 only), TODO file rename to `-DONE` (step 5 only), merge to `main` (step 5 only). Heading mark (`[DONE]` on `### 4.`) and acceptance-criteria flips ARE in this task per pre-approved caller instruction (Phase 01.1/01.2 precedent).

## Reading List (mandatory before implementation)

1. `.agent/todos/20261010/20261010-todo-2.md` — sections `### 4.`, `## Acceptance Criteria`, `## Constraints and Out of Scope`.
2. `.agent/project-info/context.md` — Current Work Focus + "Recent Changes (2026-10-11, Phase 02 Tasks 1–2)" + Known Deviations/Limitations.
3. `.kilo/plans/20261010-p02-task1-chat-model-store.md` — D5 (chat test spec) + A1–A5 amendments.
4. `.kilo/plans/20261010-p02-task2-text-renderer.md` — R5 (renderer test spec) + A6–A9 amendments.
5. `.agent/reports/20261009-phase-01-1-build-artifact-workflow.md` and `.agent/reports/20261010-phase-01-2-windows-build-workflow.md` — report section style precedents.
6. `scripts/dev-checks.sh`, `scripts/build-linux.sh`, `scripts/build-windows.sh` — READ ONLY; you run them, you never modify them.
7. Test files (read for the audit): `src/chat/model_tests.rs`, `src/chat/store_tests.rs`, `src/renderer/frame_tests.rs`, `src/renderer/settings_tests.rs`, `src/renderer/text_tests.rs`, `src/renderer/text_test_helpers.rs`.
8. `.gitignore` — re-read before every commit (Gitignore Compliance rule).
9. `.agent/project-structure.md` — current structure map (used by the docs-accuracy audit).

## Verified Baseline (observed at planning time; do not re-derive, but re-confirm Step 0)

- Branch `feat/phase02-renderer-chat-store`, working tree clean, HEAD `83b16e0` (`chore: mark phase 02 task 3 complete`).
- Latest full gate: `logs/checks/20261011T195050Z.log` — exit 0, four PASS lines, `ALL CHECKS PASSED`, **81 tests** (`running 81 tests ... 81 passed; 0 failed; 0 ignored`). Suite composition: 46 baseline (config/app) + 16 chat (5 model + 11 store) + 19 renderer (4 frame + 4 settings + 11 text).
- `src/renderer/.gitkeep` IS tracked (`git ls-files` confirms) although the module now has 10 real files; `src/chat/.gitkeep` was already removed in Task 1.
- `.agent/project-info/context.md` line 7 ("Current Work Focus") still says "No Phase 02 implementation has started" — stale; refresh is routed and pre-approved (Planner scope extension beyond Task-3's frozen-scope decision).
- Builds verified working in Phase 01.1/01.2: `scripts/build-linux.sh` → `dist/rust-youtube-streamer-service`; `scripts/build-windows.sh` → `dist/windows/rust-youtube-streamer-service.exe`; `dist/` gitignored `.gitignore:34`, never staged.
- `Cargo.toml`/`Cargo.lock` need NO changes in this task (ab_glyph 0.2.28 already locked in Task 2; no new dependencies at all).

## High-Level Approach

Task 4 is a **verification + evidence + closure** task, not a code-writing task. The test list in TODO Task 4 was DELIVERED by Tasks 1–2 by design (Task 1 plan D5 explicitly states the chat tests "DELIVER the ... TODO Task 4 test list"). This plan's coverage audit (T1) confirms every Topic-4 bullet maps onto existing named tests — conclusion: **no new tests are required** (decision final below). The implementer therefore: (1) records the completed hygiene fixes, (2) runs the authoritative gate ONCE and both release builds via the Alpine VM MCP, (3) drafts the tracked completion report filling evidence slots ONLY from observed output, (4) flips the acceptance-criteria boxes verified by that evidence, marks `### 4.` `[DONE]`, and returns a summary. Nothing in `src/`, `Cargo.toml`, `Cargo.lock`, `config/`, or `fonts/` changes in this task.

## Encoded Decisions (final; implementer must not choose)

### T1 — Test-coverage audit (mapping table; conclusion: NO GAPS, no new tests)

Audit every TODO §4 test bullet against existing named tests (names observed in the green `20261011T195050Z.log` run):

| TODO §4 bullet sub-item | Existing test(s) (exact names) | Status |
| --- | --- | --- |
| Capacity calculation — exact fit | `chat::store::store_tests::capacity_uses_floor_division_of_usable_height` | covered |
| Capacity — fractional remainder | same test (exact-fit + fractional cases asserted together per Task 1 plan D5 #6) | covered |
| Capacity — zero/invalid capacity | `chat::store::store_tests::capacity_rejects_unusable_or_overflowing_geometry` (+ `renderer::settings_tests::settings_from_config_rejects_invalid_vertical_layout` for renderer layout) | covered |
| Capacity — overflow-safe | `chat::store::store_tests::capacity_rejects_unusable_or_overflowing_geometry` (u64 arithmetic over `u32::MAX` inputs) + `renderer::frame_tests::frame_rejects_zero_or_overflowed_sizes` | covered |
| Empty-queue startup | `chat::store::store_tests::new_store_starts_empty_and_formats_single_message` | covered |
| FIFO eviction | `chat::store::store_tests::fifo_evicts_oldest_beyond_capacity` | covered |
| Append-only log ordering | `chat::store::store_tests::log_family_and_ordering_preserved_beyond_queue`, `log_record_format_with_message_timestamp`, `log_record_uses_generation_timestamp_when_message_has_none`, `new_store_ignores_existing_log_history` | covered |
| Newline handling | `chat::model::model_tests::normalization_flattens_all_line_breaks` + `chat::store::store_tests::newlines_in_messages_flatten_to_one_record_one_display_line` | covered |
| Display formatting | `chat::model::model_tests::display_format_is_username_then_message` + `new_store_starts_empty_and_formats_single_message` | covered |
| Optional metadata | `chat::model::model_tests::metadata_never_appears_in_display` + `metadata_accessors_return_input_verbatim` | covered |
| I/O failures | `chat::store::store_tests::log_open_failure_is_typed_and_surfaces_path` + `append_failure_rejects_message_and_log_stays_consistent` | covered |
| Queue-log independence | `log_family_and_ordering_preserved_beyond_queue` + `new_store_ignores_existing_log_history` | covered |
| Frame dimensions/byte count | `renderer::frame_tests::frame_filled_has_exact_byte_count` | covered |
| All-black empty input | `renderer::text_tests::empty_text_yields_completely_black_frame` | covered |
| Multiline order/placement | `renderer::text_tests::multiline_orders_top_to_bottom` + `margins_take_effect` | covered |
| Clipping | `renderer::text_tests::horizontal_clips_at_frame_edge` + `vertical_clips_at_frame_bottom` | covered |
| Font loading + errors | `renderer::text_tests::missing_font_file_returns_typed_error` + `garbage_font_file_returns_parse_error` | covered |
| Invalid geometry | `renderer::frame_tests::frame_rejects_zero_or_overflowed_sizes` + `renderer::settings_tests::settings_from_config_rejects_zero_frame_dimensions` + `settings_from_config_rejects_invalid_vertical_layout` | covered |
| Deterministic pixel assertions | All renderer tests use pixel-scan/count helpers (`count_ink_in_rows`-style via `text_test_helpers.rs`); NO golden image files exist | satisfied |

**Final conclusion (decision F1):** every TODO §4 bullet is covered; **ZERO new tests are written in Task 4**. The suite remains 81 tests. Do NOT add redundant tests, do NOT restructure test files, do NOT touch any `src/` file. The implementer verifies the audit by reading each listed test file and confirming each named test exists and matches the mapping; copy this table (with the status column) verbatim into the report's Coverage Audit section.

**Audit-mismatch protocol (stop rule):** if the implementer's read of a listed test file contradicts the mapping (test missing, renamed, or asserting something materially different), the implementer does NOT fix or rewrite anything — STOP and report the exact discrepancy to the caller. Only the caller/Planner decides (re-plan → minimal missing tests would then be specced in a new plan).

### T2 — Evidence protocol (exact commands, capture rules, what counts as evidence)

All container commands run via the `alpine-vm` MCP: call `alpine-vm_vm_status` BEFORE the first `alpine-vm_vm_run_command`, then one `alpine-vm_vm_run_command` per row below (single commands only; timeoutMs 900000 for build/gate rows, 600000 for `ls` rows). Exit code is authoritative (MCP stderr capture may be partial for cargo — a missing `Finished` line is not a failure; README policy).

Fresh-run matrix (run in this exact order, once each, after hygiene commits 1–2 of the Commit Plan):

| # | Command (verbatim) | Expected | Capture into report |
| --- | --- | --- | --- |
| F1 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` | exit 0; 4× `PASS`; `ALL CHECKS PASSED`; exactly `81 passed; 0 failed` | exit code, log filename printed by the script (`logs/checks/<UTC>.log`), the four `PASS ...` lines, the `test result: ok. 81 passed; ...` line, `--- Summary ---` block |
| F2 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` | exit 0; `Build succeeded: saved the Linux release executable to dist/rust-youtube-streamer-service (<N> bytes).` | exit code + the verbatim `Build succeeded` line (byte count = observed only) |
| F3 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh` | exit 0; `Build succeeded: saved the Windows x86-64 GNU release executable to dist/windows/rust-youtube-streamer-service.exe (<N> bytes).` | exit code + verbatim `Build succeeded` line + the two trailing caveat lines |
| F4 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows` | exit 0; both artifacts listed non-empty | verbatim listing (independent cross-check of both byte sizes and paths) |

"Build result" = artifact path + observed byte size, evidenced by BOTH the script's printed size and the identical `ls -l` value in F4; if the two disagree, STOP and report. Never compute or invent sizes; never invent SHA-256 hashes for artifacts (none are mandated by the TODO; do not generate them).

Test-count gate: F1 must show exactly 81 tests (the code tree is unchanged relative to the known-green run). Any other count or any FAIL line → STOP rules below.

STOP-on-fail rules (apply after EVERY run F1–F4): if exit ≠ 0 or an expected line is missing → capture the full output, change NOTHING (no fmt auto-apply, no code fixes, no retries, no report-drafting continuation for the failed item), record the failure in the report's commands section with its actual exit status, leave the corresponding acceptance boxes `[ ]` with the reason, and return the findings to the caller. The TODO explicitly permits failure documentation ("or actual failures/limitations are documented") — documenting a real failure faithfully is a valid completion; improvising a fix is not.

Cited (NOT re-executed) evidence is allowed only from committed/tracked sources under `.agent/` and `logs/checks/`: prior gate logs `20261011T184938Z.log` and `20261011T195050Z.log`, the Task 1/Task 2 plans, and the Phase 01.1/01.2 reports. ML/VM failure-path probes from earlier phases are NOT re-run (no such requirement in TODO §4).

Manual-platform-tests section (bound NOT-RUN statements — no native run will happen in this task; record each verbatim in the report):

```text
Native Windows execution of dist/windows/rust-youtube-streamer-service.exe: NOT RUN.
Native Linux execution of dist/rust-youtube-streamer-service: NOT RUN.
Renderer frame output exercised outside unit tests (visual/display run): NOT RUN.
Chat store runtime behavior outside unit tests (real log file via an actual app run): NOT RUN.
```

Adjoin one sentence (factual, no new claims): the earlier user-run Windows launch (Phase 01.2) validated only that `auth`/`run` placeholders load configuration and exit 3; it involved no renderer/chat behavior.

### T3 — Acceptance-criteria verification matrix (13 boxes; flip rule)

Flip rule (uniform): a box flips `[ ] → [x]` ONLY when every evidence source in its row was observed green during this task (or is pre-existing committed evidence corroborated by the gate); a box that cannot be verified stays `[ ]` with the reason recorded in the report's matrix row, AND the missing verification is flagged to the caller in the sub-agent summary. `git grep` exits 1 when there are NO matches — record `exit=1 (no matches)` as the PASSING outcome for absent-string checks.

| # | Acceptance criterion (TODO) | Evidence sources | Exact check to perform |
| --- | --- | --- | --- |
| 1 | Bounded FIFO queue + append-only log independent of visible queue state | `src/chat/store.rs`; tests `fifo_evicts_oldest_beyond_capacity`, `log_family_and_ordering_preserved_beyond_queue`, `new_store_ignores_existing_log_history` | read store.rs (log is never read back; VecDeque FIFO; log-append-first); F1 green covers the tests |
| 2 | Visible capacity derived from validated geometry; no `max_messages` config property | tests `capacity_uses_floor_division_of_usable_height`, `capacity_rejects_unusable_or_overflowing_geometry`; `src/config/model.rs`; `config/config.example.json` | host: `git grep -in max_messages` → `exit=1 (no matches)` (whole tracked tree) |
| 3 | New stores start empty; no log replay | tests `new_store_starts_empty_and_formats_single_message`, `new_store_ignores_existing_log_history` | confirm both in the F1 green run (they are in the 81) |
| 4 | Display `username: message`; metadata never visible | tests `display_format_is_username_then_message`, `metadata_never_appears_in_display`, `metadata_accessors_return_input_verbatim` | confirm green in F1 |
| 5 | Renderer trait + concrete renderer independent of YouTube and FFmpeg | module boundaries; `src/renderer/**` | host: `git grep -n "crate::youtube" -- src/chat src/renderer` → no matches; `git grep -n "crate::streaming" -- src/chat src/renderer` → no matches; `git grep -n "std::process" -- src/chat src/renderer` → no matches |
| 6 | Empty input → fully black RGB24 frame, exactly `width * height * 3` bytes | tests `empty_text_yields_completely_black_frame`, `frame_filled_is_uniform_color`, `frame_filled_has_exact_byte_count` | confirm green in F1 |
| 7 | Multiline uses font/size/line height/margins, ordered, clipped, no wrapping | tests `multiline_orders_top_to_bottom`, `margins_take_effect`, `horizontal_clips_at_frame_edge`, `vertical_clips_at_frame_bottom`; Task 2 plan R3/A8 (no wrap, no font resize) | confirm green in F1 + read `src/renderer/text.rs` clip logic (one read-through) |
| 8 | Font/geometry/overflow/log-I/O errors actionable, no panic | error files `src/chat/error.rs`, `src/renderer/error.rs`; tests `missing_font_file_returns_typed_error`, `garbage_font_file_returns_parse_error`, `log_open_failure_is_typed_and_surfaces_path`, `append_failure_rejects_message_and_log_stays_consistent`, `frame_rejects_zero_or_overflowed_sizes`, `capacity_rejects_unusable_or_overflowing_geometry`, `settings_reject_invalid_color_shapes`, `invalid_color_at_construction`, `settings_from_config_rejects_invalid_vertical_layout`, `settings_from_config_rejects_zero_frame_dimensions` | confirm green in F1; host: `git grep -n "panic!" -- src/chat src/renderer` → no matches; read both error.rs files and confirm hand-written Display + `source()` (already documented; verify once) |
| 9 | Unit tests cover capacity/FIFO/logging + renderer pixels/layout/errors | T1 audit table (complete, no gaps) + F1 `81 passed` | copy the T1 table into the report; cite the F1 test-result line |
| 10 | Standard dev-check gate passes (or failures documented) | F1 fresh run | exit 0 + `ALL CHECKS PASSED` → flip; otherwise document failure, leave `[ ]` |
| 11 | Both Linux and Windows release builds succeed (or documented) | F2 + F3 | both exit 0 → flip; any failure → document, leave `[ ]` |
| 12 | README/config docs/project structure/context accurate; no full-streaming claims | docs-accuracy audit (below) | read the four files once with the checklist; record findings in the report |
| 13 | Tracked completion report exists with real evidence | the report itself at `.agent/reports/20261010-phase-02-renderer-chat-store.md` | after writing: `git add` + verify tracked in its commit; all values in it must trace to observed output |

Docs-accuracy audit checklist for box 12 (read README.md, docs/configuration.md, `.agent/project-structure.md`, `.agent/project-info/context.md`; ALL four must hold; record each as a report checklist row with a citation to the file):
- (a) chat + renderer described as implemented but NOT yet wired into `app::run` (Phase 03 wiring stated or implied correctly);
- (b) `auth`/`run` still described as placeholders (config validated → not-implemented → exit 3);
- (c) NO claim that OAuth, YouTube API, FFmpeg process execution, audio, or live streaming is implemented anywhere in the four files;
- (d) fonts: vendored `Exo2-Light.otf` OFL 1.1 provenance + operator-supplied runtime font note present (per Task 2 plan R2 and `fonts/README.md`).
Any violation → do NOT edit docs in this task (Task 3 is closed); record the violation in the report, leave box 12 `[ ]` with the reason, and flag it to the caller.

### T4 — Hygiene steps (routed findings; exact edits bound)

1. **`src/renderer/.gitkeep` removal**: `git rm src/renderer/.gitkeep`, commit `chore(renderer): remove leftover gitkeep placeholder from the renderer module` (commit 2 in the Commit Plan). This is the ONLY change to the code tree. Precedent: `src/chat/.gitkeep` deletion in Task 1; project-structure.md already lists 10 real renderer files. Do NOT touch `src/chat/` (already clean) or `logs/.gitkeep` (belonging to `logs/`, still required).
2. **`context.md` Current Work Focus refresh**: replace the ENTIRE stale paragraph at `## Current Work Focus` (line 7, the one starting "Phase 01 (Configuration and Application Skeleton) is complete...") with the bound text below. ONLY that paragraph changes; every other line of `context.md` is untouched (Overwrite-TODO-File-Prevention discipline). Bound final text (write ONLY if every fresh run was green AND all 13 boxes flipped; otherwise STOP before this edit and report to the caller):

```text
Phase 02 (Renderer and Chat Store) is complete on branch `feat/phase02-renderer-chat-store` (package v0.5.0). All 4 tasks ran their Critical Workflow cycles: Task 1 chat message model + ChatStore (`src/chat/`, 16 unit tests), Task 2 RGB24 text renderer (`src/renderer/`, ab_glyph 0.2.28, 19 unit tests), Task 3 documentation/metadata, and Task 4 tests/builds/completion report (coverage audit found no gaps: all TODO Task-4 test bullets map onto the existing 81-test suite; the authoritative dev-checks gate and both release builds ran green via the Alpine VM MCP). The tracked completion report is `.agent/reports/20261010-phase-02-renderer-chat-store.md`; all 13 acceptance criteria in `.agent/todos/20261010/20261010-todo-2.md` are verified `[x]`. Verification remains container-only: native Windows/Linux execution of the new components is NOT RUN. Next: Phase 03 wires `chat`/`renderer` into `app::run` (retiring the deliberate `#[allow(dead_code)]` module attributes) and starts the FFmpeg process pipeline.
```

Commit: `docs: refresh phase 02 current work focus in project context` (commit 5 in the Commit Plan).

3. **Task-4 plan file tracking**: this plan file (`.kilo/plans/20261010-p02-task4-tests-builds-report.md`) is untracked after creation; the implementer stages it FIRST as commit 1 `docs: track phase 02 task 4 planning documents` (precedent `d8563df` / `f3a90c5`).

### T5 — Completion report structure (follows Phase 01.1/01.2 section style)

Create `.agent/reports/20261010-phase-02-renderer-chat-store.md` (UTF-8, real newlines). Expected > 100 lines → TOC mandatory. Title: `# Phase 02 Completion Report — Renderer and Chat Store`. Sections in this exact order:

1. **Table of Contents** — anchor links mirroring the Phase 01.2 report's TOC style (`[Overview](#overview)`, ...).
2. **Overview** — what Phase 02 added (two independent runtime components; no FFmpeg/YouTube/OAuth/streaming); state line: branch, HEAD observed via `git log --oneline -5` at drafting time, date; source TODO path; container-only caveat sentence.
3. **Files Created/Changed** — table per changed file: Task column (1–4), Change description, Commit hash OBSERVED from `git log --oneline` at drafting time (never from memory). Rows: Task 1 `src/**`, `src/config/mod.rs` (A1), Cargo.lock/Cargo.toml; Task 2 renderer files + fonts vendoring + Cargo.toml/Cargo.lock; Task 3 docs files; Task 4: this plan file, `src/renderer/.gitkeep` (deletion), this report (self row, commit marked `pending` per the Phase 01.2 report's line-32 precedent — hashes of commits created AFTER the report is written are left `pending` and recorded in the implementer's returned summary, NOT back-patched).
4. **Dependency Rationale** — ab_glyph 0.2 (pinned 0.2.28): pure-Rust, no system graphics libraries, Rust 1.82/edition-2021 compatible, Windows+Linux, Apache-2.0; locked via tracked Cargo.lock; condensed from Task 2 plan R2 + context.md dependency bullet; state explicitly that Task 4 added NO dependencies and changed NO Cargo files.
5. **Test Coverage Audit (TODO Task-4 mapping)** — the T1 table verbatim + the no-gaps conclusion sentence + the `81 passed` observation.
6. **Exact Docker/MCP Commands & Exit Statuses** — FRESH rows: F1–F4 byte-verbatim commands with exit codes and captured lines (per T2 capture column). CITED rows: prior gate logs `20261011T184938Z.log`, `20261011T195050Z.log` (as named file references with their `81 tests` counts), and the Phase 01.2 statement that failure probes were validated in that phase (not re-run). Note the MCP caveat sentence (exit status authoritative; stderr partial).
7. **Linux Build Result (observed path/size)** — path `dist/rust-youtube-streamer-service`, byte size = the value printed by F2 and cross-checked by F4; named-volume note; `not a Windows .exe` caveat from the script output.
8. **Windows Build Result (observed path/size)** — same shape for `dist/windows/rust-youtube-streamer-service.exe`; reproduction of the script's two caveat lines (cross-compilation ≠ native runtime; FFmpeg external).
9. **Manual Platform Tests Actually Performed** — the bound four NOT-RUN statements from T2 + the one clarifying sentence about the Phase 01.2 user launch.
10. **Acceptance Criteria Verification** — a 13-row matrix table: criterion (short label) | evidence (tests/greps/runs + where recorded) | decision (`[x]` / stays `[ ]` + reason). This is the durable record feeding the TODO flips in Step S8.
11. **Documentation Accuracy Audit** — the T3 box-12 four-item checklist result with file citations.
12. **Environment Notes** — container-only verification scope; `core.autocrlf=true` repo host with the `.gitattributes` LF guard; MCP exit-status-authoritative policy; named-volume `rust-streamer-target` (no `target/` at root).
13. **Evidence Index** — links (repo-relative): the global plan + per-task plan/adherence/review/simplify files for Tasks 1–3, this task's plan file, prior phase reports, the prior gate logs, the fresh gate log filename from F1; then the observed commit list (`git log --oneline` at drafting time) plus the `pending` rows for the post-report commits.
14. **Known Limitations & Not-Run Checks** — bound items: native Windows/Linux runtime NOT RUN; renderer validated by unit tests only, not wired into `app::run`; chat log record epoch-seconds format deviation (charts to context.md limitation, cite); unknown-codepoint notdef skip; `right_margin`/`bottom_margin` config-only deviation; `fonts/console.ttf` provenance indeterminate; build success = compilation evidence ONLY.

Evidence-honesty rules for drafting: every number (exit code, byte size, test count) must be pasted from captured output; percentages, hashes, dates of prior runs must come from cited files; any statement of the form "X was observed" without a captured value is FORBIDDEN. Commit hashes for this task's own commits created after drafting are `pending` rows per precedent.

## Atomic Implementation Steps

### Step S0 — Baseline confirmation (host, bash tool, single commands)

1. `git status --porcelain` → expect EMPTY output. If not empty → STOP and report (never commit unrelated files).
2. `git branch --show-current` → expect `feat/phase02-renderer-chat-store`.
3. `git log --oneline -3` → expect task-3 completion commits, `83b16e0` on top.
4. Read `.agent/todos/20261010/20261010-todo-2.md` §4 + Acceptance Criteria; confirm 13 checkboxes all `[ ]`.

### Step S1 — T1 audit re-verification (read-only; no edits)

Open each test file listed in the T1 mapping; confirm every named test exists and its subject matches the mapping row. On any mismatch → STOP rules of T1 (report, no fixes).

### Step S2 — Commit 1: track the task-4 plan file

- `git status --porcelain` (confirm only `.kilo/plans/20261010-p02-task4-tests-builds-report.md` untracked) → `git add .kilo/plans/20261010-p02-task4-tests-builds-report.md` → `git commit -m "docs: track phase 02 task 4 planning documents"`.
- Verify after commit: `git status --porcelain` EMPTY.

### Step S3 — Commit 2: `src/renderer/.gitkeep` removal

- `git rm src/renderer/.gitkeep` → `git commit -m "chore(renderer): remove leftover gitkeep placeholder from the renderer module"`.
- Verify: `git ls-files src/renderer` lists exactly the 10 real files, no `.gitkeep`; `git status --porcelain` EMPTY.

### Step S4 — Authoritative gate run (F1, Alpine VM MCP) — RUN ONCE

`alpine-vm_vm_status` → then F1 verbatim (T2 matrix). Capture per T2. Expected: exit 0, four PASS, `81 passed; 0 failed`, `ALL CHECKS PASSED`, log path `logs/checks/<UTC>.log`. Record the log filename. On any deviation → STOP-on-fail rules.

### Step S5 — Build runs (F2, F3, then F4) — Alpine VM MCP

Run F2 → record; run F3 → record; run F4 listings → record and cross-check both byte sizes against F2/F3 outputs. STOP-on-fail rules apply per run. Order is bound (Linux first, then Windows, matching TODO §4 wording).

### Step S6 — Host-side tracking checks (bash tool, single commands; capture all)

1. `git check-ignore -v dist/rust-youtube-streamer-service dist/windows/rust-youtube-streamer-service.exe` → expect both listed under `.gitignore:34:dist/`.
2. `git ls-files dist/` → expect EMPTY stdout.
3. `git status --porcelain --ignored=matching -- dist/` → expect exactly `!! dist/`.
4. `git status --porcelain -- logs/` → expect EMPTY (gate log ignored via the `logs/*` rule; nothing untracked inside `logs/`).
5. `git grep -in max_messages` → expect `exit=1 (no matches)`.
6. Box-5 greps (three commands per T3 row 5) → all `no matches`.
7. `git grep -n "panic!" -- src/chat src/renderer` → expect no matches.

### Step S7 — Draft the completion report

Write `.agent/reports/20261010-phase-02-renderer-chat-store.md` per T5, filling:
- F1–F4 outputs from the captures of Steps S4–S6;
- the docs-accuracy audit results (read the four files once now, record S7 checklist rows);
- commit hashes observed from `git log --oneline -40`;
- the 13-row acceptance matrix with per-box decisions;
- NOT-RUN statements verbatim.
Drafting may only use collected observations; nothing may be invented. The report file stays UNTRACKED until the next step.

### Step S8 — Commit 3: report (+ report self-check)

- `git add .agent/reports/20261010-phase-02-renderer-chat-store.md` → `git commit -m "docs: add phase 02 renderer chat store completion report"`.
- Verify: `git status --porcelain` EMPTY; `git ls-files .agent/reports` shows the new file.

### Step S9 — Acceptance-criteria flips + heading mark (Commit 4)

Edit `.agent/todos/20261010/20261010-todo-2.md` with surgical edits ONLY:
1. Append ` [DONE]` to `### 4. Tests, Builds and Completion Report` (heading line only; do not modify any other heading or content).
2. In `## Acceptance Criteria`, flip `[ ]` → `[x]` for EXACTLY the boxes whose evidence rows were observed green in Steps S4–S7 ( ordinarily all 13). Any box whose evidence failed or was not observed stays `[ ]` and its reason must already be recorded in the report matrix (S7). NEVER flip an unverified box; NEVER reverse an expectation silently; flag any unflipped box to the caller in the summary.
- `git add .agent/todos/20261010/20261010-todo-2.md` → `git commit -m "chore: mark phase 02 task 4 and acceptance criteria done"`.
- Overwrite-prevention verify: `git diff HEAD~1 --stat` shows the TODO file only; `git diff HEAD~1` shows only the heading line change plus the intended checkbox characters.

### Step S10 — Commit 5: context.md Current Work Focus refresh

- Apply the T4-bound conditional replacement to `.agent/project-info/context.md` (allowed ONLY if all fresh runs green AND all 13 boxes flipped; otherwise STOP and report BEFORE this edit).
- `git add .agent/project-info/context.md` → `git commit -m "docs: refresh phase 02 current work focus in project context"`.

### Step S11 — Final tree verification (host; capture all)

1. `git status` → clean, branch `feat/phase02-renderer-chat-store`, `nothing to commit, working tree clean`.
2. Re-run Step S6 checks 1–4 (dist/ ignored, no tracked dist files, logs/ clean).
3. `git log --oneline -8` → confirm this task's 5 commits in order (plan → gitkeep → report → TODO → context).
4. `git ls-files src/renderer` → the 10 real files, no `.gitkeep`.
5. Confirm NO push happened (`git remote -v` optional evidence; do NOT run `git push` under any circumstance).

### Step S12 — Return summary to caller

Report: what was done (per step), what was NOT done (no push, no TODO rename, no merge, no code changes), all exit codes, artifact sizes, test count, gate log filename, the exact list of flipped boxes (or the unflipped ones with reasons), and the `pending` commit hashes for commits 3–5 that were created after the report text was written (commit-1/2 hashes ARE recordable inside the summary since they precede drafting). The caller/Planner consumes this for 4.3 review and 4.5b adherence.

## Commit Plan (final order; messages verbatim; explicit-path staging only — NEVER `git add -A`)

| # | Files (exact paths) | Message |
| --- | --- | --- |
| 1 | `.kilo/plans/20261010-p02-task4-tests-builds-report.md` (new) | `docs: track phase 02 task 4 planning documents` |
| 2 | `src/renderer/.gitkeep` (deletion) | `chore(renderer): remove leftover gitkeep placeholder from the renderer module` |
| 3 | `.agent/reports/20261010-phase-02-renderer-chat-store.md` (new) | `docs: add phase 02 renderer chat store completion report` |
| 4 | `.agent/todos/20261010/20261010-todo-2.md` | `chore: mark phase 02 task 4 and acceptance criteria done` |
| 5 | `.agent/project-info/context.md` | `docs: refresh phase 02 current work focus in project context` |

Before EVERY commit: re-read `.gitignore`, run `git status --porcelain`, stage ONLY that commit's listed paths, and verify the staged list with `git diff --cached --stat`. `dist/`, `logs/checks/*`, `fonts/console.ttf`, `config/config.json`, `credentials/*` are never staged. Commits 1–2 precede the gate (so F1 is authoritative on the final code tree); commits 3–5 are documentation-only and change no code after the gate — record that fact in the report's Environment Notes sentence: "the dev-checks gate and both builds ran on the tree containing all code changes; only markdown files changed afterwards (commits 3–5 verifiable via git log)."

## Explicitly NOT in this plan (boundaries)

- NO code changes of any kind after commit 2 (`src/`, `Cargo.toml`, `Cargo.lock`, `config/`, `fonts/`, `scripts/`, `Dockerfile`, `docker-compose.yml` untouched; `.gitignore` untouched).
- NO new dependencies, NO version bump (v0.5.0 already current for the phase).
- NO wiring into `app::run`, NO config/schema changes, NO docs edits beyond the three routed files (report, TODO flips, context.md paragraph).
- NO `git push`, NO branch merge, NO TODO rename (`-DONE`), NO branch creation/deletion (workflow steps 5 only).
- NO native Windows/Linux execution, NO FFmpeg/YouTube/OAuth/streaming behavior, NO invented evidence of any kind.
- NO re-running of prior-phase failure probes; NO golden-image files; NO redesign of tests (T1 conclusion: no gaps).

## Self-Verification Against TODO Task 4 (every requirement mapped)

| TODO Task 4 requirement | Plan coverage |
| --- | --- |
| Unit tests: capacity exact/fractional/zero-invalid/overflow-safe | T1 table rows 1–4 (all existing, verified by name); conclusion: no gaps → no new tests |
| Unit tests: empty-queue startup, FIFO eviction, append-only log ordering | T1 rows 5–7 |
| Unit tests: newline handling, display formatting, optional metadata, I/O failures, queue-log independence | T1 rows 8–12 |
| Unit tests: frame dims/byte count, all-black empty, multiline order/placement, clipping, font loading/errors, invalid geometry | T1 rows 13–18 |
| Prefer deterministic pixel assertions over golden files | T1 row 19 (all pixel-scan helpers verify; no golden files) |
| Run standard dev gate via Alpine VM MCP, the exact compose command | Step S4 (F1) — TODO-verbatim command, run ONCE, exit-status + log capture |
| Rebuild both artifacts (exact compose commands) | Step S5 (F2, F3) + F4 artifact listings |
| Build success = compilation evidence only; no unperformed native claims | T2 NOT-RUN block; report §7–§9 caveats; limitations section bound |
| Tracked report at `.agent/reports/20261010-phase-02-renderer-chat-store.md` with changed files, dependency rationale, commands+exit statuses, test outcomes, build results, manual platform tests actually performed, limitations; never invent evidence | T5 full section spec + evidence-honesty rules; S7; NOT-RUN statements bound; D5 dependency rationale section |
| Mark tasks/acceptance criteria complete only when verified; no `-DONE` rename before Critical Workflow completion | S9 flip rule + T3 matrix; heading mark only; rename/merge/push excluded (boundaries) |
| Acceptance criteria: 13 boxes verified with evidence | T3 matrix (each box: evidence source, exact check, flip rule) |
| No OAuth/YouTube/FFmpeg/audio/streaming implemented; existing constraints intact | Boundaries section + box-5/box-12 greps + docs-accuracy audit |
| Routed finding: remove tracked leftover `src/renderer/.gitkeep` | T4 item 1 + Step S3 (commit 2) |
| Routed finding: refresh stale context.md Current Work Focus | T4 item 2 + Step S10 (commit 5, conditional wording bound) |

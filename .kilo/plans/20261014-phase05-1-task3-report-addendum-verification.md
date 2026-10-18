# Phase 05.1 — Task 3 Implementation Plan — Completion Report Resilience Addendum + Final Verification

- Date plan produced: 2026-10-17 (4.1b architector pass)
- TODO authority: `.agent/todos/20261014/20261014-todo-1.md`, section
  `### 3. Completion Report Resilience Addendum + Final Verification` + the global
  `## Scope and Boundaries` / `## Explicitly Out of Scope` / `## Acceptance Criteria`
- Global plan: `.kilo/plans/20261014-phase05-1-operator-feedback.md` (decision D-GF6 binds:
  record-only addendum — facts + ranked hardening candidates + the
  cannot-clean-up-after-own-crash reality; no implementation claims; explicitly future-work
  planning input for a LATER TODO)
- Precedent plans: Task-1 `20261014-phase05-1-task1-eligibility-multichannel-docs.md`,
  Task-2 `20261014-phase05-1-task2-watch-link-print.md`, Task-2 adherence
  (`20261014-phase05-1-task2-adherence.md` — its A-3 carry is executed by this plan §5)
- Grade note: written for a JUNIOR developer implementing in sub-step 4.2; every choice is
  made here — do not substitute "equivalent" wording, files, or commands.

## 0. Verified State (observed 2026-10-17, HEAD `f783421`)

- Branch: `feat/phase05-1-operator-feedback`; HEAD `f783421`
  (`docs: mark Phase 05.1 task 2 complete and track its adherence report`); `git status
  --porcelain` EMPTY (tree clean).
- `Cargo.toml` version: `0.9.0` (line 3; verified).
- Test baseline: 357 passed / 0 failed — gate log `logs/checks/20261015T031613Z.log`
  (Task-2 adherence §3, authoritative observed count on the v0.9.0 tree).
- `.agent/project-structure.md` root-files line (line 107) still says
  `rust-youtube-streamer-service` v0.8.0` — the stale line carried to Task 3 by the
  Task-2 adherence A-3 (explicit carry-order, quoted in that report §6/§7).
- Phase-05 report file (the addendum target):
  `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md` — 356 lines; it HAS
  a TOC (lines 3–22, 18 entries, last = `- [Evidence Index](#evidence-index)`); the file
  ends with the Evidence Index's fenced commit-ledger code block (closing fence on line
  356). The addendum APPENDS after that fence; ZERO existing lines change except the one
  appended TOC entry.
- This plan file itself exists untracked at its final path; the 4.2 implementer commits it
  FIRST (Task-2 precedent: commit 1 `docs: track the phase 05.1 task 2 … plan`).

## 1. Scope, Boundaries, Out of Scope (Task-3 zone)

Scope (TODO Task 3 bullets + caller instruction):

1. Append a dated, record-only resilience addendum to the Phase-05 report.
2. Execute the one carried one-line refresh in `.agent/project-structure.md`
   (v0.8.0 → v0.9.0).
3. Final-tree verification: standard dev-checks gate + the repo hygiene/secret audit,
   exact commands/exit statuses/log path/artifact sizes recorded in the addendum.
4. Do NOT flip any TODO acceptance-criteria checkbox in 4.2 — flips belong to THIS Task
   3 cycle's 4.6 (see §10).

Boundaries (hard):

- Files touched in 4.2 are EXACTLY three:
  1. `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md` (addendum + one
     TOC line)
  2. `.agent/project-structure.md` (one token refresh)
  3. `.kilo/plans/20261014-phase05-1-task3-report-addendum-verification.md`
     (this plan — commit first, no edits after)
- ZERO diffs allowed in 4.2 in: `src/**`, `Cargo.toml`, `Cargo.lock`, `scripts/**`,
  `docs/**`, `README.md`, `.gitignore`, `.gitattributes`, Docker files,
  `config/**`, `credentials/**`, `.agent/todos/**`, `.agent/project-info/**`.
  Guard commands in §6 S8 enforce this with `git diff --stat`.
- Out of scope (TODO `## Explicitly Out of Scope`): implementing ANY hardening item
  (bounded FFmpeg exit wait, ureq timeout, hang watchdog, external supervisor/service
  registration); Live Chat (Phase 06); changes to redaction guarantees, shutdown order,
  broadcast-fate policy, exit codes; running the manual E2E; creating new documentation
  files; creating new TODO files for the hardening work.
- No branch creation/switch, no version action, no merge, NO PUSH — restricted to
  workflow steps 2/3/5 which already happened / come later (D-GF5).

## 2. Decisions (D-T3R-1 … D-T3R-10)

- **D-T3R-1 — Addendum title (exact):**
  `## Addendum (2026-10-14): Resilience Planning — Crash/Concurrency Analysis`
  (em dash U+2014, exactly as written; the 2026-10-14 date is the date of the recorded
  operator Q&A analysis, not today's date).
- **D-T3R-2 — Placement:** append at the very END of the report, after the line-356
  closing fence of the Evidence Index commit-ledger block, separated by a blank line.
  ONE new line in the TOC: appended directly under
  `- [Evidence Index](#evidence-index)`. Line labels in the TOC use the full heading text.
  No other TOC change; no renaming; no reflow of any existing line.
- **D-T3R-3 — Anchor (verified against the report's own slugger behavior, e.g.
  `#exact-dockermcp-commands--exit-statuses` for `Exact Docker/MCP Commands & Exit
  Statuses`):** the GitHub slugger lowercases, removes `(`, `)`, `:`, `/` and the em dash
  and maps remaining spaces to hyphens, so the TOC entry is exactly:
  `- [Addendum (2026-10-14): Resilience Planning — Crash/Concurrency Analysis](#addendum-2026-10-14-resilience-planning--crashconcurrency-analysis)`
- **D-T3R-4 — Record-only rule (D-GF6 binding):** the addendum contains NO implementation
  claims, NO code sketches, NO scheduling promises ("will be implemented in Phase XXX" is
  banned), NO new TODO files, NO config/schema/error changes. It is planning INPUT for a
  later TODO; other tasks' plans are never rewritten (D-GF2); the report's existing
  sections are never rewritten.
- **D-T3R-5 — Citation ceiling:** no invented citations, no new official URLs. The ONLY
  official URL allowed is the already-recorded one — the Life-of-a-Broadcast page
  `https://developers.google.com/youtube/v3/live/life-of-a-broadcast`, cited as "recorded
  in the Phase-05 Task-1 plan (`.kilo/plans/20261013-phase05-task1-official-api-research-design.md`)"
  and "documented operator-facing in `docs/production-run.md` §Broadcast/Stream Fate".
  Everything else cites only existing repo records (report sections by NAME, context.md
  Known Deviations, brief §12.4, `src/app/run/mod.rs` module-doc fate policy, Task-2
  adherence) — by path/name, never by invented page contents or invented dates.
- **D-T3R-6 — Carried structure-map refresh:** in `.agent/project-structure.md` change
  the root-files Cargo.toml bullet ONLY: the token `v0.8.0` → `v0.9.0` (see §5 exact
  strings). The OTHER `v0.8.0` occurrences in the repo (report line 25; context.md
  Current-Work-Focus line 7) are HISTORICAL Phase-05 facts and stay untouched — out of
  zone. Verify with `git diff` that the edit is one token on line 107.
- **D-T3R-7 — No rebuilds in Task 3.** The final-tree diff has ZERO code files (S8
  guard), so both release artifacts remain the Task-2 fresh rebuilds. The addendum records
  them as CITED evidence: Linux `dist/rust-youtube-streamer-service` 5197144 bytes,
  Windows `dist/windows/rust-youtube-streamer-service.exe` 8618273 bytes (Task-2 adherence
  §3, fresh dist timestamps 2026-10-15 on the v0.9.0 tree), plus a fresh read-only
  `ls -l` cross-check on the VM (S6) confirming the observed sizes still match; if they
  differ, record the OBSERVED values (observed-total rule) and note the difference.
- **D-T3R-9 — Gate expectation:** 357 passed / 0 failed, exit 0 (Task-2 tree gate
  expected 355 baseline + 2 watch-link pins = 357, observed in
  `logs/checks/20261015T031613Z.log`). Task 3 adds zero tests. If the observed count is
  NOT exactly 357, PAUSE and escalate to the caller — never "adapt" the report text.
- **D-T3R-8 — Commit plan:** FOUR commits in 4.2 (see §8) — plan tracking, addendum,
  structure-line refresh, evidence recording — mirroring the repo's per-concern docs
  commit style; then 4.5b adherence report lands in 4.6's commit (predecessor style
  `6d3441f`/`f783421`).
- **D-T3R-10 — Commits are docs-only; before EACH `git commit` run
  `git status --porcelain` and stage ONLY the intended file(s) by explicit path (never
  `git add -A`, never `git add .`), per the Gitignore Compliance Rule.**

## 3. Addendum Content Design (the exact outline; implementer writes the prose)

All facts/citations below are already recorded in the repo; the implementer only shapes
prose around them and must not add, drop, or re-rank items. Target length ~50–70 lines
(docs file — the src 200-line cap does not apply).

### 3.1 `## Addendum (2026-10-14): Resilience Planning — Crash/Concurrency Analysis`

Intro paragraph (before any `###`), covering:
- This addendum records the 2026-10-14 operator Q&A crash-concurrency analysis for
  FUTURE planning only (global plan decision D-GF6, record-only).
- Source of the analysis: a user-directed chat follow-up (2026-10-14) after the
  Phase-05 closure Q&A; the user asked that the analysis be recorded in completion
  reports so it can be planned for in the future; it is planning input, not a work order.

### 3.2 `### Crash-recovery reality` (analysis point a — all of it required)

- An in-process watchdog (or any auxiliary process owned by the app) CANNOT clean up
  after a crash of the main process: a crashed process runs no code, so no handler,
  thread, or child of the app can symbolically run recovery on its behalf.
- This is the standing recorded reality — brief §12.4 ("An abrupt process crash cannot
  execute the application's own cleanup code"), fate-policy rule 8 in the
  `src/app/run/mod.rs` module docs, and the report's Known Limitations list item on
  crash (`docs/production-run.md` §Broadcast/Stream Fate also states it operator-facing).
- Recovery after a crash therefore happens OUTSIDE the application:
  1. YouTube-side: the broadcast is created with `enableAutoStop=true` (stage O2, decision
     D-T1-7), so YouTube auto-completes the broadcast about a minute after ingestion
     stops — official Life-of-a-Broadcast behavior (`https://developers.google.com/youtube/v3/live/life-of-a-broadcast`,
     recorded in the Phase-05 Task-1 plan; cited in `docs/production-run.md` §Broadcast/Stream Fate).
  2. Operator-side: leftover `ready` broadcasts/streams (accumulated by repeated failing
     runs; typed `userBroadcastsExceedLimit` on the limit) are deleted manually in
     YouTube Studio — automated cleanup is out of scope by policy.
- Related documented limitations this reality sits beside (cite by name): the unbounded
  FFmpeg exit wait (`finish()`), the absent ureq request-level timeout (D-T2-16(a)),
  and repeated-Ctrl+C inertness are all already on the report's Known Limitations
  list / `docs/production-run.md` §Shutdown Behavior.

### 3.3 `### Ranked hardening candidates (future planning input — record-only)`

An ordered list, ranks 1–4 EXACTLY as analyzed (do not re-rank, do not merge, do not
detail designs):

1. **Bounded FFmpeg exit wait + escalation.** Today `finish()` in
   `src/streaming/ffmpeg.rs` waits unbounded — no timeout configuration field exists
   (Phase-03 documented limitation, carried into production operations; report Known
   Limitations item (c)). A bounded wait with a defined escalation (e.g., typed
   escalation outcome rather than an indefinite wait) is the top-ranked candidate.
2. **ureq request-level timeout.** Today the concrete ureq 2 transports (token endpoint,
   authorization verifier, `GoogleLiveApiTransport`) perform requests with no
   request-level timeout (recorded deviation D-T2-16(a); report Known Limitations item
   (d)): one stalled TCP connection can hang one API call (the post-start confirmation
   gate join cost is documented). Second-ranked.
3. **In-process hang watchdog thread.** A dedicated thread could detect prolonged
   lack of progress (e.g., an FFmpeg process no longer consuming stdin, or a wait loop
   stuck beyond its documented budget), escalate by terminating the FFmpeg child, and
   surface a typed error. This addresses HANGS only — it does NOT address crashes, which
   run no application code (see `### Crash-recovery reality`). Third-ranked.
4. **External supervisor (systemd).** The brief (§12.4) leaves service supervision to a
   future Ubuntu deployment ("A future Ubuntu deployment may use systemd or another
   external supervisor"), and the recorded project scope decision states the
   application is a normal executable, not a service — supervisor configuration belongs
   to deployment, not to the application. Fourth-ranked; the brief also records that a
   supervisor "can restart the service, but it does not by itself guarantee correct
   cleanup of a previously created broadcast."

### 3.4 `### Standing behaviors that stay` (analysis point c)

- Repeated Ctrl+C inertness STAYS as documented: the installed handler replaces default
  termination (the Phase-03 five-point Ctrl+C limitations list, source
  `src/app/pipeline/interrupt.rs` module docs; restated in `docs/production-run.md`
  §Shutdown Behavior). A process stuck in a long blocking operation must be terminated
  with OS tools — that remains the documented escape and this analysis proposes no
  change to it.
- The shutdown order, resource-fate policy, and secret-redaction guarantees are NOT
  touched or re-discussed by this addendum.

### 3.5 `### What was not done / how this section is to be used` (analysis point d — one short block, exact claims)

- Nothing is claimed implemented: no hardening item above was coded, prototyped, adopted
  into a TODO, or claimed as test coverage; no configuration field, timeout, error
  variant, or module changed.
- The gold wording sense required (implementer's own sentence): this addendum is
  future-work PLANNING INPUT for a later TODO; ranked candidates are candidates, not
  commitments.

### 3.6 `### Final-tree verification evidence (Task 3)` (filled AFTER the S5 gate + S6 audit runs)

- One compact paragraph first: "Ran on the final Task-3 tree: gate + the D-T4-10
  hygiene/secret audit set adapted to the current tree; all exits 0; nothing was
  implemented by this addendum, so code/artifact facts are unchanged from Task 2."
- Then a `| Command | Exit | Observed result |` table (same shape as the report's
  `## Exact Docker/MCP Commands & Exit Statuses` and `## Security Review` tables),
  recorded VERBATIM from the actual runs. Required rows (verbatim observed values; see
  §7 for the fresh-verified expectations):
  1. Gate row: the exact VM command, exit 0, `357 passed; 0 failed`, `ALL CHECKS
     PASSED`, the new UTC log name `logs/checks/<UTC>.log` captured from the script
     output (first invocation only; the report says the summary may need a
     `tail -n 30`-style read from the log if the MCP capture cuts early — same honest
     recording as the Phase-05 report).
  2. `ls -l` cross-check rows for BOTH artifacts on the VM (observed byte sizes; if the
     observed sizes differ from the Task-2 cited values, record observed and mark the
     delta out-of-zone — nonzero byte-count difference is possible only if something
     touched `dist/`, which should not happen).
  3. Tracking rows: `git ls-files dist/` → empty; `git ls-files videos/` → empty;
     `git ls-files credentials/` → `credentials/.gitkeep` only; `git ls-files "config/config.json"`
     / `git ls-files "*.mp4"` / `git ls-files "*.exe"` → empty.
  4. `git check-ignore -v` rows: `config/config.json` → `.gitignore:48:/config/config.json`;
     `dist` → `.gitignore:34:dist/`; `credentials/tokens.json` → `.gitignore:52:credentials/*`
     (line numbers as verified 2026-10-17; record whatever fresh output observes).
  5. Dead-code/staging rows: `allow(dead_code)` = 8 attribute lines
     (`app/auth/mod.rs:86`, `chat/model.rs:44`, `chat/store.rs:123`,
     `youtube/authorization.rs:35`, `youtube/authorization.rs:93`,
     `youtube/error.rs:24`, `youtube/mod.rs:95`, `youtube/mod.rs:100`) + 1 comment-text
     mention (`youtube/mod.rs:68`); `allow(unused_imports)` = 0.
  6. Secret-pattern rows: `key=` = 1 doc-comment mention (`google_live_api_transport.rs:3`);
     `NOT_IMPLEMENTED_EXIT` = 0; `liveChat` = 0; `transition(` = 0;
     `liveBroadcasts.delete` = 0; `liveStreams.delete` = 0; `ya29` = 0; `AIza` = 0.
  7. Tree-adaptation rows (Phase-05.1 additions): `watch?v` in `src/` = exactly 2 hits
     (`src/app/run/broadcast.rs:31` constant, `src/app/run/mod_tests.rs:28` pinned test
     literal); `stream-key-fixture` in `src/` = 6 hits all in `src/streaming/*_tests.rs`;
     `client_secret`/`refresh_token` = schema/type path names + test fixtures only, never
     a literal value; `println!|eprintln!` in `src/app/run/` = 3 pre-existing lines
     (`mod.rs:55`, `mod.rs:59`, `mod.rs:70`), none added.
- Close the table with one sentence: any deviation between an expected and observed
  value above would have paused execution and escalated; the recorded table is the
  observed run.

## 4. Prohibited content (hard bans inside the addendum AND the whole 4.2 diff)

- No new official URLs or fresh web research claims — the ONLY allowed URL is
  D-T3R-5's Life-of-a-Broadcast one.
- No code, config, error, or field changes; no `#[allow...]`/imports/source edits.
- No TODO-file edits in 4.2 of any kind (no checkbox flips, no `[DONE]` marks).
- No new plan/TODO/docs files for hardening work.
- No invented quota numbers, versions, counts, hashes, or log names; every number/hash
  recorded in the addendum must appear in an actual observed command output from S5/S6
  or come from an existing tracked record cited by path.
- No rewrite of ANY existing report line other than the one appended TOC entry (D-GF2);
  no rewrite of the project-structure.md line other than the single version token
  (D-T3R-6).

## 5. The Structure-Map One-Line Refresh (exact strings)

File: `.agent/project-structure.md`, line 107 (verified; the ONLY `v0.8.0` occurrence in
that file — re-grep `v0\.8\.0|v0\.9\.0` there before editing).

- OLD (substring inside line 107):
  ```
  Cargo.toml - Rust package manifest: `rust-youtube-streamer-service` v0.8.0, edition 2021;
  ```
- NEW:
  ```
  Cargo.toml - Rust package manifest: `rust-youtube-streamer-service` v0.9.0, edition 2021;
  ```
- Nothing else on that line changes; no other lines change; the report-internal
  historical occurrences elsewhere in the repo are NOT touched.

## 6. Execution Steps (4.2 implementer; in order; one VM command per MCP invocation)

- **S0 — Pre-flight.** On the Alpine VM via MCP: call the status check FIRST, then
  `git status --porcelain` and `git log --oneline -3` on the repo host: expect empty
  porcelain, HEAD `f783421`, branch `feat/phase05-1-operator-feedback`. Confirm
  `Cargo.toml:3` `version = "0.9.0"`. Any mismatch → escalate.
- **S1 — Commit the plan file.** `git add .kilo/plans/20261014-phase05-1-task3-report-addendum-verification.md`,
  then `git status --porcelain` (staged list = exactly this one file), then
  `git commit -m "docs: track the phase 05.1 task 3 report addendum plan"`.
- **S2 — Report edit (TOC + addendum).** Using the edit tool per §3 outline:
  1. Append the TOC line (D-T3R-3) directly under
     `- [Evidence Index](#evidence-index)` (last TOC entry).
  2. Append, after the file's final fenced commit-ledger block, a blank line and the
     full addendum per §3.1–§3.5 (leave `### Final-tree verification evidence` present
     but to be filled in S5–S7; write the paragraph that says evidence rows follow).
  3. `git diff --stat` — expect exactly this one file changed.
- **S3 — Commit the addendum.** Stage ONLY the report file
  (`git add .agent/reports/20261013-phase-05-production-youtube-live-runtime.md`),
  re-check porcelain, then
  `git commit -m "docs: append the resilience planning addendum to the phase 05 report"`.
- **S4 — Structure-map refresh.** Apply §5's exact one-token edit
  (`.agent/project-structure.md`); confirm `git diff` shows ONLY line 107's token
  change; then stage ONLY that file
  (`git add .agent/project-structure.md`), re-check porcelain, then
  `git commit -m "docs: refresh the structure map package version line"`.
- **S5 — Final-tree standard gate (VM via MCP; ONE command per invocation; status
  check first).** Verbatim: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`.
  Capture: exit code (expect 0), the four `PASS` lines
  (`PASS fmt-check exit=0`, `PASS check exit=0`, `PASS test exit=0`,
  `PASS clippy exit=0`), `test result: ok. 357 passed; 0 failed; ...`, `ALL CHECKS
  PASSED`, and the fresh log name `logs/checks/<UTC>.log` printed by the script. If the
  MCP capture cuts before the summary, read the log tail on the VM with
  `sh -c "tail -n 30 /rust-youtube-streamer/logs/checks/<UTC>.log"` (same honest record
  as the Phase-05 report's `sh`-tail pattern). If the observed test count is not exactly
  357 → PAUSE and escalate (D-T3R-9).
- **S6 — Artifact freshness cross-check (VM, read-only, no build).**
  `ls -l /rust-youtube-streamer/dist/rust-youtube-streamer-service` and
  `ls -l /rust-youtube-streamer/dist/windows/rust-youtube-streamer-service.exe`;
  record the observed byte sizes and compare against the D-T3R-7 cited values
  (5197144 / 8618273). Do NOT run any build script.
- **S7 — Host-side hygiene/secret audit (the adapted D-T4-10 set; run LAST on the
  final tree; each a single plain command on the repo host; record outputs verbatim
  then complete the addendum's evidence table).** Exact list and fresh-verified
  expected outputs → §7. Then fill `### Final-tree verification evidence` rows from the
  S5/S6/S7 actual outputs.
- **S8 — Diff-surface guard.** `git diff 793be77..HEAD --stat` must list EXACTLY the
   three intended files (plan file, report, structure map) and NOTHING else; ZERO `src/**`
  / Cargo / scripts / docs / README / .gitignore diffs is the binding expectation.
- **S9 — Commit the recorded evidence.** Stage ONLY the report file again;
  `git commit -m "docs: record the task 3 final-tree gate and audit evidence in the report addendum"`.
- **S10 — Signal completion:** return the plan path, the executed command list with exit
  statuses, the gate log path, artifact sizes, the four commit hashes, and explicit
  confirmation that README/docs/src/Cargo/todos were NOT touched and NO checkbox
  flipped.

## 7. Final-Tree Gate & Audit Expectations (each freshly verified on HEAD `f783421`, 2026-10-17)

Gate (S5):

- Command: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
- Expected: exit 0; `PASS fmt-check exit=0`; `PASS check exit=0`; `PASS test exit=0`;
  `PASS clippy exit=0`; `ALL CHECKS PASSED`;
  `test result: ok. 357 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; ...`;
  fresh log `logs/checks/<UTC>.log` (name captured from the run).
- Known tolerated signals (NOT failures): the two test-profile warn-only dead-code
  warnings (`with_token_url`, `token_type`) — the adjudicated Phase-04 carry-over
  (clippy `-D warnings` passes).

Audit (S7) — expected results (each observed fresh on HEAD `f783421`):

| Command | Expected result (fresh-verified 2026-10-17) |
| --- | --- |
| `git status` | branch `feat/phase05-1-operator-feedback` |
| `git status --porcelain` | empty after S9 pre-commit staging (record the run-time state honestly per phase) |
| `git ls-files dist/` | empty |
| `git ls-files videos/` | empty |
| `git ls-files credentials/` | `credentials/.gitkeep` only |
| `git ls-files "config/config.json"` | empty |
| `git ls-files "*.mp4"` | empty |
| `git ls-files "*.exe"` | empty |
| `git check-ignore -v config/config.json` | `.gitignore:48:/config/config.json` |
| `git check-ignore -v dist` | `.gitignore:34:dist/` |
| `git check-ignore -v credentials/tokens.json` | `.gitignore:52:credentials/*` |
| `git grep -n -F "allow(dead_code)" -- src/` | 8 attribute lines (list above) + 1 comment mention (`youtube/mod.rs:68`) |
| `git grep -n -F "allow(unused_imports)" -- src/` | 0 hits |
| `git grep -in "liveChat" -- src/` | 0 hits |
| `git grep -n "transition(" -- src/` | 0 hits |
| `git grep -n "liveBroadcasts.delete" -- src/` | 0 hits |
| `git grep -n "liveStreams.delete" -- src/` | 0 hits |
| `git grep -n -F "key=" -- src/` | 1 hit: `src/youtube/google_live_api_transport.rs:3` doc-comment |
| `git grep -n "NOT_IMPLEMENTED_EXIT" -- src/` | 0 hits |
| `git grep -c "ya29" -- src/` | 0 hits |
| `git grep -c "AIza" -- src/` | 0 hits |
| `git grep -n "watch?v" -- src/` | exactly 2 hits: `src/app/run/broadcast.rs:31`, `src/app/run/mod_tests.rs:28` |
| `git grep -n "stream-key-fixture" -- src/` | 6 hits, all in `src/streaming/{args_tests,ffmpeg_stream_tests,rtmps_destination_tests}.rs` |
| `git grep -n "client_secret" -- src/` | schema/type path names + fixture strings only (never a literal value) |
| `git grep -n "refresh_token" -- src/` | schema/type path names + fixture strings only (never a literal value) |
| `git grep -n -E "println!|eprintln!" -- src/app/run/` | 3 pre-existing lines: `mod.rs:55`, `mod.rs:59`, `mod.rs:70` |

Expected-vs-observed arithmetic for the audit: TOTAL 0-secret surfaces, 8+1 dead-code
allow-line inventory (matches the Phase-05 record and the structure map's dead-code
inventory section), 2 watch-URL single-source hits (matches the Task-2 adherence §4),
3 pre-existing console lines (matches the Task-2 adherence §4). Any other count →
PAUSE, escalate, and record honestly.

## 8. Commit Plan (precedent style, lowercase conventional docs subjects)

| # | Files staged (explicit paths) | Message |
| --- | --- | --- |
| 1 | `.kilo/plans/20261014-phase05-1-task3-report-addendum-verification.md` | `docs: track the phase 05.1 task 3 report addendum plan` |
| 2 | `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md` | `docs: append the resilience planning addendum to the phase 05 report` |
| 3 | `.agent/project-structure.md` | `docs: refresh the structure map package version line` |
| 4 | `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md` | `docs: record the task 3 final-tree gate and audit evidence in the report addendum` |

Style precedent at HEAD: `docs: track the phase 05.1 task 2 watch-link plan`,
`docs: map the watch-link additions in project structure`,
`docs: mark Phase 05.1 task 2 complete and track its adherence report` (`f783421`,
`793be77`, `13e368c`, …). No amend, no rewording of existing commits, no push, no merge,
no version change.

## 9. 4.5b Adherence Expectations (what the next architector pass will check)

- Diff surface: `git diff 793be77..HEAD --stat` = exactly the three intended files; the
  plan's §4 bans hold (no src/Cargo/docs/scripts/config/credentials/TODO edits).
- Addendum shape: exact heading/TOC entry/anchor; sections in order; each required fact
  present (intro, crash reality with `enableAutoStop` + Studio recovery, ranked 1–4
  list, standing behaviors, no-claims sentence, evidence table).
- Record-only: no implementation claim, no scheduling promise, no invented citation; the
  only official URL cited = Life-of-a-Broadcast, attributed to the Phase-05 Task-1
  plan record.
- Evidence honesty: every number in the addendum's evidence table comes from an actual
  S5/S6/S7 observed output; gate count = 357 or an escalated-explanation; artifact sizes
  = Task-2 values or observed explanations.
- Commits: four subjects verbatim per §8; no amend/push/merge; per-commit porcelain
  checks recorded.
- TODO file untouched until 4.6.

## 10. 4.6 Completion Step Spec (the flips that 4.6 will make — encode now, execute then)

File: `.agent/todos/20261014/20261014-todo-1.md`. MUST be done in a SEPARATE 4.6
invocation, never during 4.2 (the caller's standing rule). Steps:

1. Append `[DONE]` to the Task-3 heading exactly:
   `### 3. Completion Report Resilience Addendum + Final Verification [DONE]`
   (Tasks 1–2 headings already carry `[DONE]`; do not touch them or anything else in
   the file — Overwrite-TODO-File-Prevention Rule).
2. Flip the eight acceptance-criteria checkboxes (lines 47–54) `   - [ ]` → `   - [x]`,
   each justified by complete final-tree evidence (checkbox-state rule: flip ONLY
   criteria whose evidence is complete on the final tree):

   | TODO AC line | Criterion (short) | Flip justification (complete evidence) |
   | --- | --- | --- |
   | 47 | eligibility/24 h docs with citations | Task-1 docs + adherence landed (commit chain `ba0ad54`/`eb8dd90`/`6d3441f`) |
   | 48 | multi-channel identity docs | Task-1 docs + adherence landed; production-run cross-note present |
   | 49 | context.md operator-fact line | Task-1 landed (one dated line recorded) |
   | 50 | watch URL printed after confirmation gate; no key/RTMPS leak | Task-2: code/tests landed + adherence MET (§3/§4; greps clean) |
   | 51 | pinned offline tests + final-tree gate + rebuilt artifacts with sizes | 357 final-tree gate (Task-3 S5, log path cited in the addendum's evidence table) + Task-2 rebuilds 5197144/8618273 (adherence-cited, evidence exists) |
   | 52 | dated resilience addendum + hygiene/secret audit result | THIS task: addendum + audit table recorded and committed |
   | 53 | auth/demo unchanged; semver-minor bump with lockfile | zero auth/demo diffs (Task-2 adherence §2); `Cargo.toml` 0.9.0 verified; lockfile action done at workflow step 3 |
   | 54 | project-structure.md and tech.md updated where touched | W12 maps + THIS task's v0.9.0 refresh; tech.md no-update ruling recorded (Task-2 adherence §6) |

   Rationale for flipping ALL eight: every criterion's evidence is complete on the final
   tree after S9; the watch-link lines (50/51) cite the Task-2 adherence's recorded
   test/gate/build evidence PLUS this task's final-tree gate; none of the eight
   references live-operator verification (the manual E2E holds separately and is NOT
   an AC here).
3. Stage ONLY the TODO file plus the 4.5b adherence report produced by this cycle
   (`git add .agent/todos/20261014/20261014-todo-1.md .kilo/plans/20261014-phase05-1-task3-adherence.md`),
   re-check porcelain, then
   `git commit -m "docs: mark Phase 05.1 task 3 complete and track its adherence report"`.
4. NOT in 4.6 (they belong to workflow steps 5, caller-owned): TODO rename with `-DONE`,
   tmp cleanup sweep, merge to `main`, push to `origin` ONLY.

## 11. Self-Check vs the TODO Task 3 Text

| TODO Task-3 bullet / caller requirement | Covered by |
| --- | --- |
| Append dated addendum → 2026-10-14 crash-concurrency analysis for future planning | §3.1–§3.6, D-T3R-1..5 |
| Cannot-clean-up-after-own-crash + YouTube-side `enableAutoStop` + operator Studio cleanup | §3.2 |
| Ranked hardening candidates (bounded FFmpeg wait, ureq timeout, hang watchdog, external supervisor/systemd) | §3.3 ranks 1–4 |
| Carried documented limitations stay as-is; repeated-Ctrl+C inertness + OS tools escape stay | §3.4 |
| Nothing claimed implemented; future-work framing; explicit later-TODO input | §3.5, D-T4-6/D-T3R-4 |
| Report exact commands, exit statuses, gate log path, artifact sizes | §3.6 table, S5/S6, D-T3R-7 |
| Standard gate on final tree; test-count confirmation | S5, D-T3R-9 |
| Hygiene/secret audit once more (adapted D-T4-10 set) | S7, §7 table |
| `.agent/project-structure.md` one-line v0.9.0 refresh (Task-2 A-3 carry) | §5, S4, D-T3R-6 |
| One addendum commit + one structure-line commit (+ group/evidence handling chosen) | §8 |
| No TODO AC flips in 4.2; exact flips stated for 4.6 | §0 item 4, §10 |
| Adherence expectations | §9 |

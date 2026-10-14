# Phase 05 — Task 4 Implementation Plan — Tests, Documentation, Builds, and Manual Verification Procedure

- Date: 2026-10-14 (planning; implementer executes on the current date)
- TODO authority: `.agent/todos/20261013/20261013-todo-1.md`, Task 4 (`### 4. Tests,
  Documentation, Builds, and Manual Verification Procedure`) + the TODO's global
  Scope/Boundaries + Explicitly-Out-of-Scope + Acceptance Criteria sections.
- USER RULING (binding, global plan §1.3): the TODO's Live-Chat acceptance-criteria lines are
  an error — Live Chat is OUT OF SCOPE of Phase 05. Task 4's coverage audit and completion
  report MUST record those lines as out-of-scope/Phase-06 and never claim them.
- Global plan (binding): `.kilo/plans/20261013-phase05-production-youtube-live-runtime.md`
  (D-G1..D-G11).
- Foundation plans (binding, consume verbatim): Task 1
  `.kilo/plans/20261013-phase05-task1-official-api-research-design.md` (§1 research record +
  D-T1-1..13), Task 2 `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
  (+ Amendments 1/2; carry-forwards E1/E2/E3 from its adherence report), Task 3
  `.kilo/plans/20261013-phase05-task3-run-orchestration.md` (+ its adherence report: V-1..V-10,
  V-10 resolved by `c838793`; E2/E3 transcribed there as Task-4 report duties).
- Prior reports (format precedent): `.agent/reports/20261012-phase-04-oauth-credentials.md`
  and `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md`.
- Status: APPROVED (per-task plans auto-approved by the user's "Approve Global and Tasks
  Plans" ruling).
- Branch: `feat/phase05-production-live-runtime`; version 0.8.0; base HEAD `1043148`
  (clean tree, verified 2026-10-14). This sub-step: 4.1b (Analysis & Planning). The implementer
  executes §8 (steps) with §9 (commits) in 4.2; reviewers use §10 (adherence) in 4.3–4.5b.

---

## 1. Verified starting state (fresh reads, 2026-10-14)

- Branch `feat/phase05-production-live-runtime`, HEAD `1043148`
  (`docs: mark Phase 05 task 3 complete and track its adherence report`), clean tree.
- Latest recorded green gate: 355 passed / 0 failed / `ALL CHECKS PASSED`
  (`logs/checks/20261014T205831Z.log`; c838793-tree, tasks 3's final commits are docs/mark
   commits only). The 355 total is the CURRENT EXPECTATION, not a guaranteed figure — the
   observed-total rule (D-T4-9) governs: the fresh gate log's count is authoritative.
- Task 3 landings fully in place: `src/app/run/` (17 files incl. tests), `src/streaming/`
  extended (`rtmps_destination.rs`, `spawn.rs`, `args.rs` stream vector, `pipe_drain.rs`,
  `local_finalize.rs`, `CompletedOutput`), `src/youtube/` wired (156 youtube tests), CLI usage
  line updated, exit-3 deleted, `allow(dead_code)` inventory = exactly 8 attribute lines (+1
  comment-text mention), `allow(unused_imports)` = 0.
- `auth` and `demo` are byte-behavior-unchanged (Task-3 adherence, evidence cited).

### 1.1 Documentation drift found (exact lines; all MUST be corrected this task)

Facts verified by fresh reads of the current docs:

| File | Line(s) | Stale content |
|---|---|---|
| `README.md` | 43–47 (Current Status) | Phase 04-era status; "`run` remains a non-interactive placeholder (exit 3)"; "249 tests" |
| `README.md` | 62 | `run` bullet "**not implemented yet**; never starts FFmpeg" |
| `README.md` | 66 | exit codes: `3 = mode not implemented yet (run)` |
| `README.md` | TOC (7–21) | no Production Run section yet |
| `docs/configuration.md` | 77 & 82 | `ffmpeg.executable` "Used at `demo` start" / "wired into the runtime via the `demo` command" — run now uses it too |
| `docs/configuration.md` | 107 | `run` — "**not implemented yet** … exits 3 … names the four missing YouTube prerequisites" |
| `docs/configuration.md` | 111 | exit codes: `3 = the run placeholder only` |
| `docs/authentication.md` | 21 | "Neither `run` (non-interactive placeholder, exit 3) nor `demo` … contacts OAuth endpoints" |
| `docs/authentication.md` | 48 | "`3` is never returned by `auth` (it is reserved for the `run` placeholder)" |
| `docs/build.md` | 76 | "`run` streaming mode is still a placeholder … exits with code 3" |
| `docs/build-windows.md` | 109 | "`run` remains a placeholder … exits with code 3" |
| `docs/demo.md` | 53 | "`3` — never used by `demo` (it belongs to the `run` placeholder only)" |
| `docs/google-cloud-prerequisites.md` | 42 | "`liveBroadcasts` / `liveStreams` lifecycle | future production `run` backend | later phases" (the run backend now exists) |

`docs/demo.md` line 80's "147 tests" is HISTORICAL Phase-03 verification evidence — retain
unchanged. `docs/authentication.md` §Verification Evidence and its limitations text remain
factually valid (no live Google authorization performed) — retain; only the two drifted lines
(21, 48) are edited.

### 1.2 Carry-forward duties owned by THIS task (record, no code)

From Task-2 adherence §5 carry-forwards:
- **E2**: the completion report's transport/dependency section must record (a) the verified
  ureq 2 3xx-delivery behavior against D-T2-5's originally-worded `Err(Status)` path —
  as-built, the ONE `respond` path classifies BOTH the Ok(non-2xx) arm and the
  `Err(ureq::Error::Status)` arm through the same status-first classifier, so 3xx (redirects
  disabled) lands in `ApiRejected { status }` either way, pinned by the loopback test
  `redirects_are_not_followed`; and (b) the `send_string(body.to_string())` + explicit
  `Content-Type: application/json` equivalence to the plan's `send_json` sketch (ureq locked
  WITHOUT the `json` feature; enabling it would contradict the no-new-/no-changed-dependency
  ruling; wire bytes identical, loopback body assertions pin them).
- **E3**: the completion report must note as-built facts: D-F (roster-A test 11's second arm
  uses an absent/non-JSON body because the frozen Task-1 model tolerates a missing `items`
  key as an empty list — `Ok(None)` per D-T2-7), and D-A/D-B/D-C (F-T2-1's single-file sketch
  landed as the `api_request.rs` + `google_live_api_transport.rs` cap re-cut; roster-A tests
  split into two siblings; roster-B tests split into happy/failure siblings).
From Task-3 adherence §8: E1 is RESOLVED as recorded exception D-T3-15 (transcribe the
resolution + the four signature names in the report's deviations/limitations section).

---

## 2. Binding Decisions (D-T4-1 .. D-T4-14)

Refinements under Task 4's authority. Any deviation requires a planner adjudication.

- **D-T4-1 — Coverage audit verdict: NO new automated tests.** TODO Task 4 bullet 1's seven
  categories (API request/response mapping, broadcast lifecycle, RTMPS target redaction,
  orchestration ordering, startup failures, shutdown, demo preservation) are ALL already
  covered by the Task 1–3 rosters (fresh `#[test]` greps verified: `src/app/run` = 17 tests;
  `src/streaming` = 39 incl. args 6, ffmpeg_stream 3, rtmps_destination 2; `src/youtube` = 156,
  per the Task-3 adherence record). The full mapping table is §3 — it is the audit RECORD and
  goes verbatim into
  the report. NO test file is created in Task 4. If the implementer or a reviewer believes a
  genuine offline gap exists, they STOP and return the specific gap + missing-assertion text
  to the caller for a planner ruling — no improvised tests (junior-50% rule). The chat-related
  TODO bullet phrase and the "chat ingestion" acceptance-criterion phrase are recorded
  out-of-scope (Phase 06) in §3's table and in the report — never as missing test coverage.
- **D-T4-2 — New doc name: `docs/production-run.md`.** The operator guide for the production
  run. Sections (exact list, in this order; add a TOC — file will exceed 100 lines; real
  newline characters, no literal `\n`; ≥100-line rule satisfied):
  1. Title `Production Run`
  2. Overview — what one `run` invocation does (creates ONE new broadcast + ONE live-stream
     resource from `youtube.broadcast.*`; binds; waits for readiness; acquires the RTMPS
     destination; streams continuous rendered frames to YouTube over RTMPS via the configured
     external FFmpeg; the frame content contains NO chat text — Live Chat is Phase 06,
     recorded on the TODO defect ruling).
  3. Prerequisites — (a) the Google Cloud setup of [`docs/google-cloud-prerequisites.md`]
     (API enablement, Desktop-app client at `oauth.client_config_path`); (b) completed
     authorization via `auth` (tokens at `oauth.credentials_path`; re-run `auth` per
     [`docs/authentication.md`] when revoked/unrefreshable); (c) FFmpeg installed separately
     (never bundled; `ffmpeg.executable` configurable); (d) a filled-in `config/config.json`
     (all six sections; recommended `unlisted` for first verification); (e) a parseable font.
  4. What happens at startup, in order — the ten D-T3-2 stages in operator-facing wording:
     config load/validation → the single Ctrl+C handler pair is installed (stop flags armed
     before any cloud call) → credentials load + verify-first probe + (only if the probe
     rejects) exactly-once refresh + atomic persist + re-verify → the required-scope gate
     BEFORE any resource creation → stream insert → broadcast insert → bind
     (boundStreamId asserted) → bounded readiness wait (up to 180 s, 5 s interval —
     cancellable) → RTMPS destination acquisition (kept secret, never printed) → FFmpeg
     start (RTMPS-target argument set: FLV muxer, 2-second-target GOP, no local file) →
     post-start confirmation that `streamStatus == active` (up to 120 s, 5 s interval,
     thread-parallel with the frame loop — documented refinement) → continuous frames.
     State plainly: nothing is marked live until the API reports the stream active while
     FFmpeg is running.
  5. What the logs look like (without secrets) — table of the human-visible lines/events in
     order (startup config validated; `broadcast prepared for streaming` with the non-secret
     `broadcast_id`; `FFmpeg started toward the configured streaming destination`;
      `stream confirmed active on YouTube`), plus the standing secret guarantees: OAuth tokens,
     refresh tokens, client secrets, stream keys, full RTMPS URLs, and authorization codes
     NEVER appear in logs, error lines, or diagnostics; FFmpeg's own stderr echo of the output
     URL is replaced with `[redacted]` in captured diagnostics; the stream key/RTMPS URL is
     never embedded in command logs, error text, or debug output.
  6. Shutdown behavior — Ctrl+C: the frame loop stops within ≤ one frame interval; FFmpeg
     stdin closes (EOF) and the process is waited on; any in-progress API wait is cancelled;
     success prints on stdout, verbatim:
     `streaming stopped; the broadcast auto-completes on YouTube after ingestion ends`
     and exits 0; a Ctrl+C before streaming started prints
     `run stopped by the stop request before streaming started` and exits 0 (no
     auto-complete claim in that case — the broadcast never went live); any fatal failure is
     one redacted `error:` line on stderr and exit 1 (documented exit-code table: 0 = clean
     stop / pre-streaming stop; 1 = startup or runtime fatal; 2 = usage error; exit code 3 no
     longer exists). Repeated Ctrl+C is inert (the handler replaces default termination —
     inherited Phase-03 limitations list), and the FFmpeg exit wait is unbounded (documented
     Phase-03/05 limitation — no timeout field exists).
  7. Broadcast/stream fate (policy + honest limitations) — the resources are NEVER deleted
     and NEVER manually transitioned by this application (no `transition`/`delete` calls
     exist in the binary); the broadcast was created with `enableAutoStop=true`
     (`enableAutoStart=true`, no monitor stream), so YouTube automatically ends the broadcast
     about a minute after ingestion stops (official Life-of-a-Broadcast wording); after a
     successful clean stop the broadcast is expected in YouTube Studio as `complete`/ended —
     confirm there; startup failures after the stream/broadcast inserts leave the
     `ready` resources in place (nothing went live — no video flowed), and repeated failing
     runs ACCUMULATE orphaned broadcasts/streams (typed `userBroadcastsExceedLimit` outcome
     on the limit); operator guidance = delete stale broadcasts/streams in YouTube Studio
     (automated cleanup is out of scope); on an abrupt crash there is no application cleanup
     (brief §12.4) — inspect and end any still-live broadcast in YouTube Studio manually.
     Failure recovery pointers: OAuth problems → `docs/authentication.md` (recovery table);
     Google Cloud/API-quota topics → `docs/google-cloud-prerequisites.md`; any still-live
     broadcast after a crash → YouTube Studio (this document, §7; the separate
     `docs/streaming-recovery.md` named in brief §12.4 is NOT created — decision D-T4-11).
  8. Manual verification procedure (the safe test plan; operator-run) — D-T4-8, verbatim.
  9. Out of scope — Live Chat (Phase 06), no broadcast reuse/multi-broadcast/dashboard, no
     automated resource deletion, audio.
  10. See Also — README, docs/configuration.md, docs/authentication.md,
      docs/google-cloud-prerequisites.md, docs/demo.md.
- **D-T4-3 — README deltas** (in place; no other README section content changes except the
  two bullet lines and the Current Status rewrite):
  1. TOC: insert `- [Production Run (YouTube Live)](#production-run-youtube-live)` directly
     after the `Authentication (OAuth 2.0 — Key Features)` entry (TOC/section order kept
     consistent — put the new header after that Authentication section in the body).
  2. Current Status: rewritten to the Phase 05 state: Phases 00–04 complete; Phase 05
     (production live runtime) implemented — the `auth`, `demo`, and `run` commands are all
     real; `run` creates/binds one new broadcast+stream from the validated config, readiness-
     checks with bounded cancellable waits, acquires the RTMPS destination, and streams
     continuous chat-free frames via the configured external FFmpeg over RTMPS; shutdown is
     ordered (stop flag + wait cancellation + FFmpeg stdin EOF + wait) and the broadcast
     auto-completes via `enableAutoStop` (never auto-deleted/transitioned); suite green with
     **355** container tests — OR the observed fresh-gate count if changed; limitations
     repeated: automated verification is container-only, native Windows/Linux runtimes and
     the live YouTube end-to-end run are NOT performed by the agent (the manual procedure is
     operator-run; see docs/production-run.md).
  3. Commands list: replace the `run` bullet (line 62) with: "`run` — streams the configured
     broadcast to YouTube Live: creates one new broadcast and live-stream resource from
     `youtube.broadcast.*`, waits for readiness, acquires the RTMPS ingestion destination
     (never logged), and sends continuous rendered frames through the configured external
     FFmpeg executable; no chat text appears in Phase 05 (Live Chat is Phase 06). Success
     stops cleanly with exit 0 on Ctrl+C; any startup or runtime failure exits 1 with one
     redacted `error:` line; never interactive. Full guide: [`docs/production-run.md`](docs/production-run.md)."
  4. Exit-codes line (line 66): replace with `0 = --help, a successful demo recording, a
     verified auth success, or a clean run stop; 1 = configuration or logging startup
     failure, a demo runtime failure, any auth failure, or any run fatal failure;
     2 = usage error` — the phrase `3 = mode not implemented yet (run)` is DELETED.
  5. The new `## Production Run (YouTube Live)` section: a short summary block (what run
     does, prerequisites one-liner incl. FFmpeg external + authorized credentials, what logs
     show without secrets, shutdown + broadcast fate with the enableAutoStop/no-auto-delete
     policy, the anchor link to `docs/production-run.md`, and the standing NOT-RUN honesty
     line). Keep the existing FFmpeg-is-external paragraphs intact.
  6. The closing italic footer line (`*Initial README — detailed documentation…*`) stays.
- **D-T4-4 — docs/configuration.md deltas**:
  1. ffmpeg `executable` bullet + the paragraph at line 82: the "used at `demo` start"
     wording becomes "used by the `demo` and `run` commands"; the closing sentence "wired
     into the runtime via the `demo` command" becomes "wired into the runtime for local
     recording (`demo`) and the production RTMPS stream (`run`); launch-process failures
     surface the same typed error text for both modes".
  2. `run` bullet (line 107): full replacement with the D-T4-3-style summary — implemented
     path, one new broadcast+stream from config, readiness waits bounded + cancellable
     (180 s/5 s readiness, 120 s/5 s post-start confirmation), RTMPS destination never
     logged, ordered shutdown, exit codes 0/1 (and the `run` usage line already implemented),
     link to `docs/production-run.md`.
  3. Exit-codes line (line 111): drop the `3 = the run placeholder only` clause and extend
     the exit-1 list with run-origin fatal failures; note the pre-streaming Ctrl+C stop and
     the clean stop both exit 0.
  4. Placeholder wording GONE: the phrases "not implemented yet", "exits 3", the
     four-missing-prerequisites sentence are all removed from `configuration.md`.
  5. Line 30's usage block may gain nothing (the run line is already there).
- **D-T4-5 — Drift corrections + cross-link (authentication.md, build.md, build-windows.md,
  demo.md, google-cloud-prerequisites.md)**:
  1. `docs/authentication.md` line 21: "Neither `run` … placeholder, exit 3…" → correct to:
     `demo` (local-only recording) never contacts OAuth endpoints; `run` CONSUMES the
     credential service non-interactively (load → verify-first → refresh-once → persist →
     re-verify) and points to `docs/production-run.md`.
  2. `docs/authentication.md` line 48: "reserved for the `run` placeholder" → "`auth` never
     returns 3" stays, but the parenthetical becomes "exit code 3 is no longer used by any
     command" (exact wording flexible; the FACT is: no exit 3 anywhere).
  3. `docs/authentication.md` §Failure/Recovery: add one cross-link sentence: production-run
     failures during an unattended `run` are classified in `docs/production-run.md`; the
     recovery path for revoked/unrefreshable credentials remains re-running `auth`.
  4. `docs/build.md` line 76 + `docs/build-windows.md` line 109: replace the stale
     "run placeholder / exits 3" sentence with: the Phase 05 production `run` command is
     implemented; building the artifacts still performs NO live Google/YouTube activity and
     validates NO native runtime behavior (the manual production verification is
     operator-run; see `docs/production-run.md`).
  5. `docs/demo.md` line 53: `"3" — never used by demo` stays, but the parenthetical
     becomes "exit code 3 no longer exists in the CLI" (the run placeholder is retired).
  6. `docs/google-cloud-prerequisites.md` line 42 table row: "later phases" →
     "the production `run` command (Phase 05: `docs/production-run.md`)" for the
     liveBroadcasts/liveStreams row. Nothing else in that file changes.
- **D-T4-6 — Metadata updates**:
   1. `.agent/project-structure.md`: final v0.8.0 refresh — re-verify EVERY per-file claim
      against `git ls-files`/the tree (the Task-3 landed state already matches; the
     expected Task-4 additions are: `docs/production-run.md` in the `docs/` entry list; a
     dated note line in the dead-code staging inventory ONLY IF it changed — expected NO
     change at 8 attribute lines; update the version note for the final structure commit).
     Verify the `src/app/run/` 17-file claim, `src/streaming/` 19-file claim (incl.
     `spawn.rs`, `source_helpers.rs`), and the `src/youtube/` 62-file claim before writing.
   2. `.agent/project-info/tech.md`: append ONE Phase-05 block under Dependencies + a
      lifecycle paragraph: the live API transport decision (ureq 2 `GoogleLiveApiTransport`:
     redacts-disabled ureq agent, OAuth Bearer only, never an API key, no request-level
     timeout — documented limitation; `send_string` + explicit JSON content type because the
     locked ureq has no `json` feature); the attempt-bounded wait constants (readiness
     36 × 5 s; post-start confirmation 24 × 5 s); the lifecycle ruling
     (autoStart/autoStop, no monitor stream, no transition/delete calls ever);
     `RunError` exit mapping (clean/pre-streaming stop 0; all fatals 1); NO new dependencies
     in Phase 05 (Cargo.toml/lock unchanged since the Phase-04 bump).
  3. `.agent/project-info/context.md`: (a) Current Work Focus rewritten — Phase 05 complete
     through Task 4 automation; manual E2E remains operator-run outstanding; (b) a NEW
     "Recent Changes (2026-10-13/14, Phase 05 …)" section summarizing the whole phase
     (branch `feat/phase05-production-live-runtime`, v0.7.1 → 0.8.0, the four tasks with
     their plan/adherence paths, commit-highlight facts (from the ledger assembled in
     §8-S4), 355-test gate, artifact sizes (from §8-S6's fresh runs), E1/E2/E3 record);
     (c) a NEW "Known Deviations and Limitations (Phase 05)" section — the section list is
     report-spec §5 item 17; (d) Immediate Next Steps rewritten: Phase 05 closed pending
     manual E2E
     evidence; Phase 06 next (Live Chat) with the hand-off pointers (the byte-unchanged
     `MessageSource` seam; V2-staged `live_lifecycle`/`live_models` retirement contract;
     run-time chat scope research deferred there).
- **D-T4-7 — Completion report**: NEW tracked file
  `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md` (EXACT name; a NEW
  file — never edit a prior report). Format mirrors the Phase-04 report: TOC, then the
  section list in §5. Markdown-generation-rule authority: the implementer creates this file
  under this plan's caller-commissioned instruction (planner-owned output tracked).
  Punctuation/line-count drift policy (Phase-04 precedent): if a section count differs from
  an estimate, the OBSERVED fresh evidence wins and a post-review docs-specialist pass may
  append a correction section.
- **D-T4-8 — Manual verification procedure (canonical wording listed here; recorded in
  `docs/production-run.md` §8 AND referenced by the report)** — the operator-run safe test
  plan. Numbered, executable steps + reminders, in this exact order:
  1. **Scope reminder (READ FIRST)**: this procedure is performed by the human operator
     against a real YouTube account. It is DISTINCT from every automated/container check in
     this work (the agent did not run, and never claims, this procedure; its result may be
     recorded only by the operator, e.g. in an issue/comment — never as build evidence).
  2. **Privacy + secrets reminder**: keep OAuth tokens (`oauth.credentials_path`), the client
     secret (`oauth.client_config_path`), and any stream key/RTMPS URL you may inspect in
     YouTube Studio PRIVATE; never paste them into the repo, docs, issues, logs, or commits.
     Use a test broadcast with `privacy_status: "unlisted"` in `config/config.json`.
  3. Prereq check: valid `config/config.json` (all six sections), FFmpeg installed and
     `ffmpeg.executable` correct, credentials authorized via the documented `auth` run.
  4. Start: run the executable: `rust-youtube-streamer-service run`.
  5. Expected startup output (order only, no secrets): config/authorization progress; one
     `broadcast prepared for streaming` line carrying the broadcast_id; the FFmpeg-started
     line; the `stream confirmed active on YouTube` line. NOTE the exact broadcast_id shown —
     it identifies the resources in later steps.
  6. Actual video arrival: open YouTube Studio → the broadcast enters live; verify real
     frames appear (continuous rendered video; the Phase 05 stream has NO chat text —
     expected, chat is Phase 06). Confirm at least ~1–2 minutes of continuous video.
  7. Ctrl+C shutdown: after confirming video, press Ctrl+C. Expected: frame loop stops;
     the stdout line `streaming stopped; the broadcast auto-completes on YouTube after
     ingestion ends`; process exit code 0.
  8. Post-shutdown broadcast status: wait about 1–2 minutes (the `enableAutoStop`
     auto-complete window), then open YouTube Studio → confirm the broadcast identified in
     step 5 shows a completed/ended status (`complete`); record the observed transition
     moment. (This autoStop fact comes from the official Life-of-a-Broadcast documentation
     cited in the Phase-05 Task-1 plan §1.1/D-T1-7 — any deviation observed live is recorded
     honestly, never reconciled by editing the code.)
  9. On any startup failure: capture the single redacted `error:` line, verify no token/key
     text appears in it, and use the failure/recovery tables (production-run §7,
     authentication.md) — again keeping secrets private.
  10. Evidence-recording guidance for the operator: record the unlisted-broadcast creation,
      video arrival, Ctrl+C shutdown message, exit code, and post-shutdown status — each
      with timestamps — in whatever tracking artifact the operator prefers.
- **D-T4-9 — Verification/build commands (observed evidence duty)**: exactly these, through
  the Alpine VM MCP (`alpine-vm_vm_status` FIRST, then separate invocations — never a chained
  command):
  ```sh
  docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
  docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh
  docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh
  ls -l /rust-youtube-streamer/dist/rust-youtube-streamer-service
  ls -l /rust-youtube-streamer/dist/windows/rust-youtube-streamer-service.exe
  ```
  Expected: gate exit 0 (`ALL CHECKS PASSED`, four PASS lines, 355 tests expected — observed
  log count is authoritative; log path `logs/checks/<UTC>.log` recorded); builds exit 0 with
  the byte counts printed by the scripts and confirmed identical by the `ls -l` cross-checks.
  Order: the user-docs commits land FIRST, then the fresh gate (docs do not affect cargo,
  so one fresh gate on the final docs tree covers them) and the builds; the metadata commits
  and the report land AFTER the gate/builds/audit runs cite them (§9's ordering note is
  binding).
- **D-T4-10 — Hygiene/secret audit (exact commands; host-side git, read-only)** — run LAST
  (on the final pre-report content), record verbatim outputs in the report §Security Review.
  EXACT command set (one command per line; each is a single plain command — never a chain):
  ```sh
  git status
  git status --porcelain
  git ls-files dist/
  git ls-files credentials/
  git ls-files videos/
  git ls-files "*.mp4"
  git ls-files "*.exe"
  git ls-files "config/config.json"
  git check-ignore -v config/config.json
  git check-ignore -v dist
  git check-ignore -v credentials/tokens.json
  git grep -n -F "allow(dead_code)" -- src/
  git grep -n -F "allow(unused_imports)" -- src/
  git grep -in "liveChat" -- src/
  git grep -n "transition(" -- src/
  git grep -n "liveBroadcasts.delete" -- src/
  git grep -n "liveStreams.delete" -- src/
  git grep -n -F "key=" -- src/
  git grep -n "NOT_IMPLEMENTED_EXIT" -- src/
  git grep -c "ya29" -- src/
  git grep -c "AIza" -- src/
  git grep -n "client_secret" -- src/
  git grep -n "refresh_token" -- src/
  git grep -n "stream-key-fixture" -- src/
  ```
  Expected results: `dist/`/`videos/`/`config.json` ls-files EMPTY; `credentials/` = `.gitkeep`
  only; no `*.mp4`/`*.exe` tracked; `check-ignore` names `.gitignore` rules for
  `/config/config.json`, `dist/`, and `credentials/*`; `allow(dead_code)` = exactly 8
  attribute lines + 1 comment-text mention; `allow(unused_imports)` = 0; `liveChat` = 0;
  `transition(`/delete call sites = 0; `key=` = doc-comment only; `NOT_IMPLEMENTED_EXIT` = 0;
  `ya29`/`AIza` = 0; `client_secret`/`refresh_token` hits only as schema/type path names and
  test fixtures, never a literal token value; `stream-key-fixture` only in test files +
  doc mentions. If the host shell quoting rejects a line, run its verbatim equivalent on the
  Alpine VM git (record which side produced each result).
- **D-T4-11 — streaming-recovery decision**: `docs/streaming-recovery.md` is NOT created in
  Task 4 (it is not a TODO Task-4 deliverable; the recovery content belongs inside
  `docs/production-run.md` §7 per D-T4-2 item 7). Brief §12.4's manual recovery content
  (inspect/end a live broadcast in YouTube Studio) IS COVERED there as documented; the
  report's Known-Limitations section notes that the separately named file remains future
  work.
- **D-T4-12 — Commit changes: docs-only.** No `src/**`, no Cargo files, no `.gitignore`, no
  scripts, no config-example change, no Dockerfile/compose change — ZERO code-edit exceptions
  (if a review discovers a code bug, return it to the caller; Task 3's cycle owned code).
  Files/paths allowed to change in 4.2:
  - NEW `.kilo/plans/20261013-phase05-task4-verification-docs.md` (this plan; committed first),
  - `README.md`, `docs/configuration.md`, `docs/production-run.md` (NEW),
    `docs/authentication.md`, `docs/build.md`, `docs/build-windows.md`, `docs/demo.md`,
    `docs/google-cloud-prerequisites.md`,
  - `.agent/project-structure.md`, `.agent/project-info/tech.md`,
    `.agent/project-info/context.md`,
  - `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md` (NEW).
  NOTHING else.
- **D-T4-13 — Green-at-each-point + acceptance-criteria flips**: every commit is docs-only,
  so each point leaves the observed gate valid (docs do not affect cargo; the re-verification
  after the final report commit is S11/the caller's 4.6 — same expected counts). The
  acceptance-criteria checkbox flips and the Task-4 `### 4 …[DONE]` mark are the CALLER's 4.6
  step (the implementer does NOT flip them in 4.2). In the report, the criteria matrix records
  per criterion: `ready (evidence)` or `out-of-scope — Phase 06 (user ruling)` for the two
  chat lines (TODO criterion line 88 and the "chat ingestion" phrase of line 91).
- **D-T4-14 — Documentation honesty rules (brief §13/§16)**: never claim (a) any live/real
  YouTube interaction, (b) native Windows/Linux runtime validation, (c) any secret material
  available to log, (d) `streaming-recovery.md` exists. Every manual-only claim is phrased
  as "operator-run; recorded in docs/production-run.md; NOT executed by the agent".

---

## 3. Coverage audit map (todo Task 4 + acceptance criteria → evidence)

Legend: ✔ covered by existing offline tests; 🕐 this task's execution evidence (commands,
not tests); ⛔ user-ruling: out-of-scope/Phase-06 (recorded, NEVER claimed missing or done).

| TODO Task-4 bullet / criterion | Existing evidence | Status |
|---|---|---|
| Bullet 1a — API request/response mapping tests | `google_live_api_transport_tests.rs` (2 assembly tests on loopback), `live_models_tests.rs` (8), `api_request` spellings pinned | ✔ |
| Bullet 1b — broadcast lifecycle tests | `live_ingestion_tests.rs` (happy 6) + `live_ingestion_failure_tests.rs` (9) + `readiness_wait` 12 + `live_lifecycle_tests.rs` (7) | ✔ |
| Bullet 1c — RTMPS target redaction tests | `rtmps_ingestion_tests.rs` (6), `rtmps_destination_tests.rs` (2), `ffmpeg_stream_tests.rs` (3: clean exit / `[redacted]` diagnostics / destination-free Spawn), `live_error_display_tests.rs` (redacting Debug) | ✔ |
| Bullet 1d — orchestration ordering tests | `runtime_tests.rs` (5: ledger §5, scope-gate, broadcast-failure, gate-fatal, shutdown order) + `gate_tests.rs` (3 zero-sleep) | ✔ |
| Bullet 1e — startup failure tests | `run/error_tests.rs` (4 incl. exit mapping + the primary matrix), `run/credentials_tests.rs` (3), config loader 21 | ✔ |
| Bullet 1f — shutdown tests | cancellation/wait tests (`wait_cancellation` 2, `readiness` loop 10 + confirmation 2, runtime test 5 cancel-before-join) + the `primary_shutdown_outcome` matrix | ✔ |
| Bullet 1g — local-demo preservation tests | 14 demo tests byte-unchanged + `args_tests` locked local vector + the 2 new CLI usage tests (`usage_text_advertises_the_implemented_run_command`, `usage_text_no_longer_claims_not_implemented`) | ✔ |
| Bullet 1h — Live Chat ingestion tests | `liveChat` grep = 0; chat source seam byte-unchanged; §3 verdict: out-of-scope per ruling | ⛔ recorded |
| Bullet 2 — fakes-only/no network/no real broadcast | standing rule enforced by every suite (fake transports, loopback 127.0.0.1 servers only); fresh gate run this task | ✔ |
| Bullet 3 — dev-checks gate | §8-S5 FRESH run | 🕐 |
| Bullet 4 — Linux rebuild | §8-S6 | 🕐 |
| Bullet 5 — Windows rebuild | §8-S6 | 🕐 |
| Bullet 6 — docs + structure/tech/context updates | §4, §8-S3/S4/S7 | 🕐 |
| Bullet 7 — tracked report under `.agent/reports/` | this task (D-T4-7) | 🕐 |
| Bullet 8 — safe manual test plan (real unlisted broadcast, video arrival, Ctrl+C, post-shutdown status, privacy reminders, user-vs-agent distinction) | `docs/production-run.md` §8 per D-T4-8 verbatim | 🕐 (content — execution is operator-owned, never performed by the agent) |
| Bullet 9 — no credentials/tokens/keys/RTMPS URLs/local MP4s/build artifacts committed | §8-S8 audit greps | 🕐 |
| AC 83 — run exit 3 gone, non-interactive | cli tests + `NOT_IMPLEMENTED_EXIT` grep = 0 | ✔ |
| AC 84 — Phase-04 credential service reused; no OAuth duplication | `run/credentials.rs` consumes `CredentialService` verbatim; runtime tests 1–2; `Cargo.toml` diff empty | ✔ |
| AC 85 — new broadcast/stream created from configured metadata, bound, readiness-checked | `live_ingestion` happy tests; `prepare` ledger | ✔ |
| AC 86 — RTMPS destination from the response; no key/URL exposure | redaction suite (1c row) | ✔ |
| AC 87 — FFmpeg transmits configured continuous video to YouTube Live incl. frames while chat empty; demo local MP4 unchanged | AUTOMATED PART: `ChatFreeSource` (yields None; loop continuous), `stream_pipeline_passes_the_destination_to_the_ffmpeg_argv`, demo regression suite; ACTUAL YOUTUBE TRANSMISSION: manual procedure only — recorded as NOT YET performed by the agent, evidence comes only from the operator-run procedure | ✔ automated + 🕐 manual-portion independence |
| AC 88 — chat fetched/logged/rendered | user ruling: out-of-scope — TODO defect line; NEVER claimed; Phase 06 owns it | ⛔ recorded |
| AC 89 — bounded/cancellable waits/polls/shutdown | readiness/wait/cancellation suites (1f row) | ✔ |
| AC 90 — typed errors all boundaries + secret-safe diagnostics | error tests across `youtube/live_error`, `app/run/error`, `streaming/error`, `auth`, `demo` | ✔ |
| AC 91 — offline tests cover lifecycle/orchestration/redaction/cancellation/demo (the "chat ingestion" phrase here is part of the ruled-out chat criterion; the remaining list item is met) | every suite offline; fresh gate | ✔ (chat phrase ⛔ recorded) |
| AC 92 — normal shutdown/startup-failure resource behavior documented + tested where practical | documented: `run/mod.rs` module docs (D-T3-6 rule 8) + `docs/production-run.md` §7; tested where practical: zero `transition`/`delete` greps + `userBroadcastsExceedLimit`-class tests (`quota_reasons_map`) | ✔ |
| AC 93 — docker checks pass; both artifacts rebuilt; blockers documented else | §8-S5/S6 fresh runs | 🕐 |
| AC 94 — docs/structure/context/report accurate | Documentation Accuracy Audit table in the report | 🕐 |
| AC 95 — no secrets/artifacts committed | §8-S8 audit | 🕐 |

Report instruction: the report's Test-Coverage-Audit section reproduces this table verbatim,
INCLUDING the ⛔ rows, and never flips a ⛔ row to covered.

## 4. Documentation content specification (pointer; full operator prose lives in D-T4-2/3/4/5)

The new doc's section deliveries and the README/configuration/auth/drift edits are fully
specified by D-T4-2 (its item 2 IS the section list + item 8 being the D-T4-8 block),
D-T4-3 (README), D-T4-4 (configuration.md), D-T4-5 (drift corrections + cross-link). No other
documentation file is modified in this task.

---

## 5. Completion report content specification

File: `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`. EXACT section
list (Phase-04-report format; each section and its required content):

1. **Table of Contents** — one anchor per section below (the exact set of items 2–18).
2. **Overview** — branch/version/HEAD-at-report-time; the phase's one-paragraph summary; the
   verification-boundary paragraph: container-only automated evidence; NO manual YouTube/
   native validation performed by the agent; the user ruling (chat out of scope) restated.
3. **Selected API Operations & Scopes** — verbatim-consistent with Task-1 plan §1.1/§1.2:
   the O1–O5 table (`liveStreams.insert`, `liveBroadcasts.insert`, `liveBroadcasts.bind` →
   `liveStreams.list` readiness, `liveBroadcasts.list` status) with HTTP paths/parts, the
   NOT-called record (no `transition`, no `delete`, no `liveChat*` Phase-06), the scope
   ruling (`https://www.googleapis.com/auth/youtube.force-ssl` documented-sufficient;
   REQUIRED_SCOPE unchanged), and the official citations (liveBroadcasts/liveStreams/errors
   reference pages; rtmps-ingestion guide; the authentication page incl. OAuth-not-API-key
   and service-account-unsupported notes; Life-of-a-Broadcast autoStart/autoStop wording;
   FFmpeg-protocols rtmp URL shape). Quota-cost line (insert/bind 50 units; list ≈ 1;
   $0/10,000/day).
4. **Dependencies** — Phase 05 added NONE; task-1 ruling D-T1-1; enumerate the pinned stack
   in this phase's runtime surface (ureq 2, serde/serde_json, oauth2, ab_glyph, ctrlc,
   tracing/tracing-subscriber, std) and that `Cargo.toml`/`Cargo.lock` diffs in the phase
   window are EMPTY; carry E2's recording into this section (3xx-delivery + `send_string`
   equivalence facts). E3's D-F/D-A/D-B/D-C notes live in the Deviations section.
5. **Lifecycle / Shutdown / Resource-Fate Decisions** — D-T1-7 (autoStart/autoStop, monitor
   stream off), D-T1-8/D-T3-2 (the ten stages incl. the ONE documented refinement: the
   thread-parallel post-start confirmation gate — + the why), D-T1-9/D-T3-6 (the eight
   shutdown rules incl. no-masking primary-precedence and the eight-rule fate policy:
   never delete/transition; `enableAutoStop` auto-completes; startup-failure leftovers stay
   `ready`; pre-streaming Ctrl+C makes no auto-complete claim; crash = manual Studio
   recovery; repeat-run accumulation documented), D-T3-15/E1 exception transcription (four
   signature names + rationale), D-T3-12 (the voided ready→live log note).
6. **Secret-Redaction Review** — the D-T3-11 surface enumeration with the test names that
   pin each surface; fresh-grep redaction confirmation output (§8-S8's `key=`/`ya29`/
   `AIza` rows); the statement that no live secret material ever entered the tree.
7. **Files Created/Changed** — full phase ledger: verbatim `git log --oneline main..HEAD`
   (the Phase-04-report precedent), grouped by task with the per-task plan/adherence
   pointers and a per-task changed-file breakdown; the report's own hash intentionally
   absent (embed only already-landed hashes); the Task-4 docs commits listed as landing
   after the cited fresh evidence runs.
8. **Test Coverage Audit (TODO Task-4 mapping)** — the §3 table verbatim (with ⛔ rows),
   plus the observed per-suite inventory derived from the FRESH gate log (Phase-04 §table
   style) and the phase test-arithmetic chain: 254 → 282 (T1) → 332 (T2) → 355 (T3, +27 − 4)
   → 355 (T4, zero new) — with the observed-total rule note.
9. **Exact Docker/MCP Commands & Exit Statuses** — the fresh gate, both builds, and the
   `ls -l` cross-checks as verbatim command lists with observed exit codes/log paths/byte
   counts; if any run is blocked, record the blocker EXACTLY — never claim success.
10. **Linux Build Result / Windows Build Result** — artifact paths + observed byte counts +
    `ls -l` match; the NOT-native-runtime wording and the cross-compilation ≠ native runtime
    line in both sections.
11. **Acceptance-Criteria Verification** — the 13-criterion matrix from §3 (numbers
    correspond to the TODO's AC list 83–95), flip decision per row (`ready` / ⛔).
12. **Manual Verification Procedure (operator-run)** — D-T4-8 reproduced (or explicitly
    referenced as `docs/production-run.md` §8's authoritative copy + a short summary); the
    header states: the agent did NOT execute this; results never build phase evidence.
13. **Checks Actually Performed vs NOT Performed** — honest ledgers: performed = the fresh
    gate + builds + `ls -l` + hygiene greps + doc audits; NOT performed = the manual
    unlisted-broadcast E2E, native Windows/Linux execution of the run path, any live
    YouTube ingest/inspection, any live Google call.
14. **Documentation Accuracy Audit** — the per-doc table (this §4's touched files with
    PASS/finding rows incl. the drift-correction citations).
15. **Security Review** — the D-T4-10 audit outputs (verbatim expected-result comparison
    table + any finding); gitignore-rule intact notes; no `credentials/README.md`
    created (Decision B restated).
16. **Deviations / Carry-Forwards** — E2/E3 transcriptions (the wording duties, §1.2);
    E1's resolution pointer (the D-T3-15 recorded exception: the four signature names +
    rationale, transcribed verbatim from the Task-3 plan/adherence); the Task-2/Task-3
    adjudication lists referenced (D-A..L / V-1..V-10 remain documented in their adherence
    reports); any Task-4-local deviation (docs-specialist corrections) recorded here.
17. **Known Limitations & Phase-06 Handoff** — enumerate:
    (a) Live Chat deferred to Phase 06 (the scope-research + seam attachment point
    `src/app/pipeline/source.rs`; the V2-staged `live_lifecycle`/`live_models` retirement
    contract); (b) native Windows/Linux runtime NOT RUN; (c) the unbounded FFmpeg-finish
    wait limitation stands (no timeout field per TODO scope); (d) ureq calls carry no
    request-level timeout (D-T2-16(a); one stalled TCP connection can hang one call; gate
    join cost documented); (e) repeated-run resource accumulation (Studio-deleting
    guidance); (f) the `google-cloud-prerequisites` video-verification caveat; (g) Windows
    rename-arm/ACL carry-overs (Phase-04 L1/L4); (h) `expires_in_seconds`-advisory and
    listener read-timeout carry-overs as they affect production operations.
18. **Evidence Index** — plan/adherence paths, fresh-log paths, prior reports, full commit
    ledger.

---

## 6. High-level implementation approach

Task 4 is a documentation/evidence task: no production code changes are planned (§3's audit
shows every required test category already covered). The implementer (1) commits the plan,
(2) writes the operator guide + the user-docs refresh and drift corrections (commits 2–5),
(3) runs the FRESH gate and both release builds with `ls -l` cross-checks on that final
docs tree, (4) runs the hygiene/secret audit greps, (5) refreshes the three metadata files
(commits 6) citing the observed evidence, and (6) writes + commits the tracked completion
report (commit 7) embedding that evidence, with the chat criteria recorded out-of-scope and
no manual-run claims.

## 7. Composition pattern reminder (VM/MCP conventions)

All docker/cargo/build commands via the Alpine VM MCP: `alpine-vm_vm_status` called before
every `run` batch; commands NOT chained; separate invocations one command each; observed
exit code is authoritative; cargo stderr (`Finished` lines) may be visually absent from the
captured VM output (the documented MCP caveat). Artifact sizes verified on the VM with
`ls -l` (paths in D-T4-9). Hygiene/secret git greps run on the HOST (per the rules they
power nothing here, but keep the convention: MCP tools preferred for docker/cargo only).

---

## 8. Implementation Steps (4.2; sequential; tiny; each step's commands given verbatim)

- S0 Verification baseline (host git, read-only): `git status --porcelain` (expect empty),
  `git branch --show-current` (expect `feat/phase05-production-live-runtime`),
  `git log --oneline -3` (expect HEAD `1043148` or later doc commits). NO tree mutation.
- S1 Commit the plan: `git add .kilo/plans/20261013-phase05-task4-verification-docs.md`
  + `git commit -m "docs: track the phase 05 task 4 implementation plan"`.
- S2 Create `docs/production-run.md` per D-T4-2 (full operator guide; the D-T4-8 manual
  verification procedure is section 8 of the file). NO invented claims; every fact traceable
  to D-T4-2/D-T4-8/D-T1-8/D-T3-2/D-T3-6/D-T1-9/D-T3-7/D-T3-8 evidence.
- S3 User-facing docs commit group:
  1. README per D-T4-3 (Current Status rewrite, run bullet, exit-codes line, TOC + section).
  2. `docs/configuration.md` per D-T4-4.
  3. Drift fixes per D-T4-5 items 1–6 (authentication.md, build.md, build-windows.md,
     demo.md, google-cloud-prerequisites.md) incl. the cross-link.
  - Suggested commits (exact wording may adapt; content per §4 spec):
    `docs: refresh user documentation for the implemented run command` (README bullet/exit
    codes/TOC/section + stub links),
    `docs: add the production run operator guide` (production-run.md),
    `docs: align configuration and authentication guides with the production run`
    (configuration.md + authentication.md + google-cloud-prerequisites.md),
    `docs: retire stale run placeholder claims in build and demo guides` (build.md,
    build-windows.md, demo.md).
- S4 Metadata re-verifications (read-only): fresh `git grep` inventories
  (`allow(dead_code)`/`allow(unused_imports)`/`liveChat`/exit-3/`transition(`/`key=`), the
  per-file tree list claims (project-structure's claims), and `git log --oneline main..HEAD`
  for the ledger. NO source changes; record observed values (they feed D-T4-6/context+tech
  and the report's Security-Review/tests-arithmetic sections).
- S5 FRESH gate (docs already committed):
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
  → expect exit 0, `ALL CHECKS PASSED`, 355-test line; record `logs/checks/<UTC>.log` path
   AND the observed count. On ANY failure: STOP, return the verbatim log lines to the caller;
   no improvised fixes — a docs-only tree must not break cargo, so any failure signals a
   pre-existing tree problem only the caller may adjudicate.
- S6 FRESH builds: the two §2 D-T4-9 build commands (each its own invocation), then the two
  `ls -l` cross-checks (VM). Record: exact byte counts (script-printed AND `ls -l`-confirmed
  identical). NO doc hardcoding of the byte sizes.
- S7 Metadata writes (in this order; commit after all three):
  1. `.agent/project-structure.md` final v0.8.0 refresh (D-T4-6 item 1).
  2. `.agent/project-info/tech.md` Phase-05 block (D-T4-6 item 2).
  3. `.agent/project-info/context.md` per D-T4-6 item 3 (Current Focus, the new Recent
     Changes section, the new Known Deviations & Limitations section — items below in
     §5.17, and the rewritten Immediate Next Steps).
  - Commit: `docs: record phase 05 production runtime project metadata` (structure + tech +
    context may share one commit — the Phase-03/04 `docs` commit precedent; the tree/docs
    split may also use separate commits — PICK the single combined one).
- S8 Hygiene/secret audit (D-T4-10's exact command set) — record every output verbatim.
- S9 Write the completion report `.agent/reports/20261013-phase-05-production-youtube-live-runtime.md`
  per §5 (Phase-04-format TOC + sections; every existing hash verified via
  `git log --oneline main..HEAD`/`git rev-parse` before quoting; the report's own hash
  deliberately absent — embed only already-landed hashes). NO over-claim: the criteria/limit
  wording follows D-T4-13/D-T4-14 exactly; the chat rows are ⛔-marked.
- S10 Commit the report:
  `git add .agent/reports/20261013-phase-05-production-youtube-live-runtime.md`
  + `git commit -m "docs: add the phase 05 completion report"`.
- S11 Post-commit determinism check (cheap): one fresh
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
  on the report-tree (docs-only since S5; expect identical counts). If the caller's 4.6 next
  step performs its own final audit, this S11 result may be cited there instead — record
  both paths honestly in the report §Checks if it is skipped (S11 may be skipped ONLY if
  the caller's 4.6 will rerun the gate; otherwise run it).
- S12 Hygiene final: `git status --porcelain` (clean except nothing pending), `git ls-files
  dist/` (empty), no ignore-matching files staged. Task 4's 4.2 ends here; steps
  4.3–4.6 follow in their own sub-agent invocations.

---

## 9. Commit Plan (implementer; feature-branch commits only; NO push, NO merge, NO version
bump; D-G11)

```text
1. docs: track the phase 05 task 4 implementation plan               (S1)
2. docs: add the production run operator guide                       (S2:
   docs/production-run.md + the manual verification section)
3. docs: refresh user documentation for the implemented run command  (S3: README)
4. docs: align configuration and authentication guides with the production run
                                                                     (S3: configuration.md +
                                                                     authentication.md +
                                                                     google-cloud-prerequisites.md)
5. docs: retire stale run placeholder claims in build and demo guides (S3: build.md +
                                                                     build-windows.md + demo.md)
6. docs: record phase 05 production runtime project metadata         (S7: structure.md +
                                                                     tech.md + context.md)
7. docs: add the phase 05 completion report                          (S9/S10; embeds the
                                                                     S5–S8 evidence)
```

Seven commits total. The hygiene/secret audit (S8) produces observations recorded INSIDE
commit 7's report — it is not a separate commit. Green-at-each-point: commits 1–6 are
docs-only (cargo unaffected); commit 7 is also docs-only, so the S5 gate remains valid for
the final tree; the optional S11 re-gate (or the caller's 4.6 re-gate) provides the
final-tree citation. Commits 1–6 precede the fresh gate (the evidence tree); commit 7
documents its results.

Ordering note: if the implementer prefers different grouping, the ONLY binding rules are
(a) the plan commit first; (b) all user-docs commits BEFORE the fresh gate; (c) metadata +
report commits AFTER; (d) never push/merge/bump.

---

## 10. Adherence Expectations (4.3–4.5b reviewers)

- Diff surface: EXACTLY the D-T4-12 file list; zero `src/**`/Cargo/config/scripts changes.
- All four Task-4 evidence duties realized: fresh gate (exit 0 + log path + observed
  count), both builds exit 0 with `ls -l`-verified byte counts, the D-T4-10 audit outputs
  verbatim-recorded, and the manual procedure present in `docs/production-run.md` §8 +
  referenced in the report.
- Report self-consistency: no claim of a manual run; chat criteria ⛔-marked and never
  counted as coverage; observed-test-total rule respected; all hashes pre-verified
  (the report's own commit hash absent per the Phase-04 precedent); E2/E3/E1 recordings
  present verbatim.
- Doc drift: the §1.1 fixed-lines table verified CORRECTED (each listed line no longer
  contains the stale phrasing; spot-read each file); TOC anchors/links resolve
  (`README.md`, `docs/production-run.md`, `docs/configuration.md`); no claim anywhere that
  `docs/streaming-recovery.md` exists; the `docs/demo.md` 147-test line UNCHANGED.
- Metadata correctness: `project-structure.md` matches the tree per fresh `git ls-files`;
  context.md contains the new Recent-Changes/Known-deviations sections and no secrets;
  tech.md factual (no invented dependency lines).
- Code rules: nil (docs-only). Markdown files: real newlines, no `\n`-literals;
  >100-line docs carry TOCs.

---

## 11. TODO Task-4 bullet → plan mapping self-check

| TODO Task-4 bullet | Plan coverage |
|---|---|
| Add unit/integration tests for the seven named categories; no Live Chat tests | D-T4-1 + §3 (all ✔ existing; chat ⛔) |
| Fake transports/chat sources/FFmpeg for automated tests; no Google contact | standing rule — audit-checked in S4/S8; recorded in report |
| Run the dev-checks gate | D-T4-9 + S5 |
| Rebuild Linux and Windows | D-T4-9 + S6 |
| Update README/configuration/other docs, structure/tech/context, tracked report | D-T4-3/4/5/6/7 + S3/S7/S9 |
| Safe manual plan (real unlisted broadcast, video arrival, Ctrl+C, post-status confirmations; privacy + distinctions) | D-T4-8 (in docs/production-run.md §8 + report §12) |
| Confirm nothing secret/artifact committed | D-T4-10 + S8 |

---

## 12. Explicitly Out-of-Task (restated for the implementer)

- NO production/source code change of any kind (any discovered defect → return to caller).
- NO new dependency, NO `Cargo.toml`/`Cargo.lock` edit, NO config-example edit.
- NO acceptance-criteria checkbox flips and NO `[DONE]` marks (caller's 4.6).
- NO TODO rename, NO merge, NO push (workflow step 5 territory; only branch-local commits
  in 4.2).
- NO Live Chat artifact or claim — Phase 06 per the user ruling.
- NO claim that the manual E2E was run; no native-runtime claim anywhere.
- Do NOT edit previous phases' reports (append-free); this task's report is a NEW file.

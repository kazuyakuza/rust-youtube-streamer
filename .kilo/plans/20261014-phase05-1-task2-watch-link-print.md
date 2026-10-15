# Phase 05.1 — Task 2 Implementation Plan — Console Watch-Link Print

- Date: 2026-10-14
- TODO authority: `.agent/todos/20261014/20261014-todo-1.md`, Task 2 (`### 2. Console Watch-Link
  Print`) + the global Scope/Boundaries + Explicitly-Out-of-Scope sections.
- Global plan (binding): `.kilo/plans/20261014-phase05-1-operator-feedback.md`
  (D-GF1 non-secret watch URL `https://www.youtube.com/watch?v=<broadcast_id>`, D-GF5 version/branch
  handling, D-GF2 docs-zone ruling).
- Foundation plan (binding, consume verbatim): Task-3 orchestration plan
  `.kilo/plans/20261013-phase05-task3-run-orchestration.md` (D-T3-2 startup stages, D-T3-7 tracing
  whitelist + stdout reservation, D-T3-8 exit contract, D-T3-9 seams, D-T3-11 redaction enumeration,
  D-T3-14 code rules). This plan may not contradict them; refinements below are the bindings
  Task 2 owns.
- Designer guarantee of record for the derivation: `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
  §1.3, "Broadcast response: `id: string` — broadcast ID (== the corresponding `video` resource
  ID.)", recorded from the official
  `https://developers.google.com/youtube/v3/live/docs/liveBroadcasts` reference page
  ("both last updated 2026-09-14"). The watch-link derivation treats the broadcast id as the
  video id on that basis (§3 below records the full chain; docs wording is constrained accordingly,
  D-T2W-9).
- Status of this sub-step: 4.1b (Analysis & Planning). The implementer executes §8 in 4.2;
  reviewers use §11 in 4.3–4.5b.
- Branch: `feat/phase05-1-operator-feedback`; version 0.9.0 (bump already done at workflow step 3);
  expected base HEAD `6d3441f`, clean tree; baseline suite 355 tests green (observed gate counts are
  authoritative over any estimate in this plan).

---

## 1. Verified starting state (fresh reads, 2026-10-14)

- `src/app/run/mod.rs` (182 lines): `run_runtime` (lines 121–158) consumes `prepared` exactly once —
  `info!(broadcast_id = prepared.broadcast_id(), "broadcast prepared for streaming")` (line 136), a
  `stream_id` copy, then `prepared.into_ingestion().into_url()` re-wrapped into `RtmpsDestination`
  (the ONE reveal expression), then `spawn_startup_gate(GatePlan { transport, cancellation,
  stream_id, stop_flag })`, then `runner.run(config, StreamingExecution { … })`.
- `src/app/run/gate.rs` (114 lines): the confirmation gate runs on a dedicated std thread;
  `run_confirmation`'s `Ok(())` arm emits `info!("stream confirmed active on YouTube")` (line 98)
  and records `GateFinish::Confirmed`. This is the ONLY site of the confirmation event and the ONLY
  place the confirmation milestone is observable mid-stream (the runner joins the gate only at
  shutdown, so any emission "after the gate observes active" must live in this thread's Ok arm).
- `src/app/run/gate.rs` `GatePlan` fields today: `transport`, `cancellation`, `stream_id`,
  `stop_flag` (all `pub(super)`).
- `src/app/run/streaming.rs` `StreamingExecution` carries `stream_id: String` with the documented
  comment "rides along as non-secret operator diagnostics for recorded-test capture (the gate owns
  its own copy)" — instantiated precedent for an operator-diagnostic ride-along field.
- `src/app/run/broadcast.rs` (37 lines): the broadcast-resource stage (`prepare_broadcast`,
  `StageRuntime`, `epoch_seconds_now`) — the run-side locus of broadcast identity.
- `src/youtube/live_ingestion.rs`: `PreparedBroadcast` exposes `broadcast_id() -> &str` and
  `stream_id() -> &str`; the derived `Debug` is secret-safe via the redacting `RtmpsIngestion`
  wrapper; no change required there.
- `src/app/run/runtime_test_fixtures.rs` exposes `pub(super) BROADCAST_ID_FIXTURE =
  "broadcast-id-fixture"` and `STREAM_ID_FIXTURE = "stream-id-fixture"`; the scripted transport and
  the gate fixtures keep material `-fixture`-shaped; no test contacts Google.
- Test precedents: `mod_tests.rs` pins constant strings verbatim (stop messages, exit code);
  `runtime_tests.rs` TD-1 asserts the recorded runner's captured bundle (`destination_debug`,
  `stream_id`, `gate_finish`); `gate_tests.rs` builds `GatePlan` through one local `gate_plan`
  helper. There is NO tracing-subscriber capture precedent in `run/` (tracing messages are inline
  literals today); the pinned-line strategy therefore follows the constant-pin + recorded-capture
  precedents (D-T2W-5).
- Docs state: `docs/production-run.md` "What the Logs Look Like (Without Secrets)" table has 4
  rows; row 4 is `stream confirmed active on YouTube`; the intro sentence reads "via `tracing` on
  stderr; stdout carries only the final stop line". The manual procedure step 5 (line 125) lists
  the expected startup lines. `README.md` line 124 (the Production Run section) DOES reference the
  event list ("(`broadcast prepared for streaming` with the non-secret broadcast id, `FFmpeg
  started toward the configured streaming destination`, `stream confirmed active on YouTube`)") —
  so the README edit condition in TODO bullet 37 is MET and the edit is allowed.
- Baseline 0.9.0 tree: 355 tests green; no version/dependency actions belong to Task 2 (D-GF5,
  D-T2W-7).

## 2. Binding Design Decisions (D-T2W-1 .. D-T2W-10)

Any deviation requires a planner adjudication. Code rules per D-T3-14 apply to every edited file
(≤200 lines, ≤50-line bodies, ≤2 depth, ≤2 params, private-by-default, no commented-out code).

### D-T2W-1 — Event shape, emission point, and ordering (pinned)

- **Level**: `info!` (consistent with every other D-T3-7 event).
- **Exact printable line message** (single source of truth, one named constant):
  `pub(super) const WATCH_LINK_MESSAGE: &str = "public watch link for the live stream";`
- **Structured field set**: `watch_url` ONLY (non-secret; the URL embeds the broadcast id after the
  `?v=` prefix, so carrying `broadcast_id` a second time in the same event would duplicate the id;
  the existing `broadcast_id` field of the `broadcast prepared for streaming` event is UNTOUCHED and
  stays the operator's id anchor). The D-T3-7 tracing whitelist gains exactly one field name:
  `watch_url`.
- **Emission expression** (compile-pinned to the constant so test-pinned text cannot drift from the
  emitted text — the tracing macro builds its `message` field via `format_args!`, which requires a
  literal format string, therefore the constant is passed through the `"{}"` format arm):
  `info!(watch_url = %watch_url, "{}", WATCH_LINK_MESSAGE);`
  (`%` Display shorthand on the owned `String`; no allocation beyond the event's own rendering.)
- **Emission point**: `src/app/run/gate.rs`, private `fn run_confirmation`, the `Ok(())` arm,
  written immediately AFTER the existing `info!("stream confirmed active on YouTube")` line and
  BEFORE `GateFinish::Confirmed` is recorded. This is the exact post-confirmation-success moment
  the TODO bullet requires; the gate thread is the only site where `stream confirmed active on
  YouTube` and the watch link can be temporally adjacent (the runner joins the gate at shutdown
  only).
- **Ordering (pinned)**: `stream confirmed active on YouTube` FIRST, `public watch link for the
  live stream` SECOND, both on the same gate thread. No other arm (Cancelled/Fatal/Panicked) ever
  emits the watch-link event — a cancelled or failed confirmation never claims a watch link.
- **Channel**: stderr via `tracing` (never stdout — stdout stays reserved for the final stop line,
  D-T3-7). No `println!`/`eprintln!` anywhere in this task.
- Emission ordering (event adjacency in the Ok arm) is verified by SOURCE REVIEW at 4.3 — not by a
  unit test (no subscriber-capture precedent exists in `run/`, and adding one is out of scope);
  the recorded-capture test pins the URL VALUE (D-T2W-5) and the constants pin the TEXT.

### D-T2W-2 — Derivation site: one pure builder in the broadcast stage; NO new file (ruling)

- **Function name / signature / file (pinned)**:
  `pub(super) fn public_watch_url(broadcast_id: &str) -> String` in `src/app/run/broadcast.rs`
  (the broadcast-resource stage — where run-side broadcast identity is owned), accompanied by
  `pub(super) const WATCH_URL_PREFIX: &str = "https://www.youtube.com/watch?v=";` directly above it.
  Body (whole function, exact):
  ```rust
  /// The public watch URL of the prepared broadcast (D-T2W-2): YouTube
  /// treats the broadcast id as the corresponding video's id (recorded
  /// Phase-05 Task-1 research, §1.3, from the official liveBroadcasts
  /// reference). The URL is NON-SECRET — it carries only the broadcast
  /// id already printed in `broadcast prepared for streaming`.
  pub(super) fn public_watch_url(broadcast_id: &str) -> String {
      format!("{WATCH_URL_PREFIX}{broadcast_id}")
  }
  ```
  1 arg (≤2 rule ✓), pure (no I/O, no clock, no config), no `Display`/`Debug` fallout (a plain
  `String` return; no derive is added to any type).
- **No validation is added**: the input is the API-returned broadcast id already extracted by
  `inserted_broadcast_id` upstream; adding a new failure class would be exit-code drift (forbidden).
- **Ruling — no new source file**: `WATCH_URL_PREFIX` + `public_watch_url` extend the EXISTING
  `broadcast.rs` rather than creating a new `watch_link.rs`-style file (broadcast.rs sits at
  ~50 lines after the edit, far under the 200 cap; a new file would add a `mod` line + structure-map
  entry for zero cohesion gain). Recorded as the standing Task-2 ruling for 4.3 review.
- **Naming**: `watch_url` for variables/fields; `public_watch_url` for the builder; the two
  constants keep the SCREAMING_CASE repo convention.

### D-T2W-3 — Carriage of the derived URL (derive once, pass it down; no `src/streaming/` /
`src/youtube/` change)

- `run_runtime` derives EXACTLY ONCE, immediately after the existing `broadcast prepared for
  streaming` info event and BEFORE the `prepared.into_ingestion()` move (order enforced by the
  borrow: `public_watch_url(prepared.broadcast_id())` must precede `into_ingestion`):
  `let watch_url = public_watch_url(prepared.broadcast_id());`
- `GatePlan` gains `pub(super) watch_url: String` (after `stream_id`, before `stop_flag`); the gate
  thread emits from this owned copy. Derivation stays OUT of the gate thread (emission-only
  responsibility, pinned in D-T2W-1).
- `StreamingExecution` gains `watch_url: String` — the EXACT mirror of the existing `stream_id`
  ride-along (same redacting precedent, same "the gate owns its own copy" phrasing in its doc
  comment). Production `PreparedStreaming::run` destructures it as `watch_url: _watch_url`
  (identical shape to `stream_id: _stream_id`, which is why no dead-code/unused warning appears in
  the dev profile — the same pattern escapes today).
- The recording fixture (`runtime_runner_fixture.rs`) captures the field so the orchestration test
  can assert the exact URL (D-T2W-5); `RecordedExecution` gains `watch_url: String`.
- **Boundary ruling (recorded)**: `src/streaming/**` and `src/youtube/**` require ZERO edits. The
  derivation needs no new accessor (`broadcast_id()` exists), no youtube type changes, and no
  streaming seam change (`RtmpsDestination` is untouched; the watch URL shares nothing with it).
  If a reviewer proposes widening either boundary, escalate — do not improvise.

### D-T2W-4 — Secrets audit (the emitted line contains ONLY the non-secret URL)

Enumerated reasons, each standing from existing guarantees (nothing new is invented):

1. The emitted URL is `{WATCH_URL_PREFIX}{broadcast_id}` — the prefix is a fixed public constant
   and the only variable part is `broadcast_id`, the API response id already printed by
   `broadcast prepared for streaming` (D-T3-7 whitelisted field). New secret material: NONE.
2. The derivation path never touches the destination: `public_watch_url` reads ONLY
   `prepared.broadcast_id()`. `RtmpsIngestion::into_url` (the ONE reveal point) keeps exactly one
   production call site — the existing `RtmpsDestination::new(prepared.into_ingestion().into_url())`
   expression in `run_runtime`; it is NOT reused for the watch URL.
3. The stream key/streamName and RTMPS URLs live only inside the redacting wrappers
   (`RtmpsIngestion`, `RtmpsDestination`); neither `Debug` nor `Display` is added or weakened
   anywhere in this task (zero derive changes; the existing `RtmpsDestination(<redacted>)`
   contract stays test-pinned by runtime_tests TD-1 unchanged).
4. The emitted event has field set exactly `{watch_url}` plus the constant-sourced message; no
   other field, no error text, no panic message is touched; `RunError` variants and the exit-code
   contract are byte-unchanged (no variant added, no mapping changed, no `demo`/`auth` touched).
5. Reviewer grep audits (4.5b, executed fresh):
   - `git grep -n "watch?v" src/` → EXACTLY 2 hits: `WATCH_URL_PREFIX` in `broadcast.rs` and the
     pinned-literal full URL in the new `mod_tests.rs` test — the single-source audit for the URL
     text (the constant is the only runtime definition site; the test literal is pinned text).
   - `git grep -in "rtmps\\|stream key\\|ingest" src/app` → only the pre-existing test-fixture and
     doc-comment hits (the watch-link edit adds none).
   - `git grep -n "println!\\|eprintln!" src/app/run/` → the two existing stops in `mod.rs` only.
   - `git grep -n "derive(Debug" src/app/run/` → unchanged from baseline.

### D-T2W-5 — Test plan (offline; no Google; pinned text + recorded capture)

Pinned strategy mirrors the two repo precedents: verbatim constant pins (`mod_tests.rs`) and
recorded-execution capture (`runtime_tests.rs` + `runtime_runner_fixture.rs`). NO
tracing-subscriber capture machinery is introduced (no precedent; out of scope).

- **New tests (exactly 2, both in the EXISTING `src/app/run/mod_tests.rs` — no new test file; the
  no-new-file ruling in D-T2W-2 coherently governs the test side too, keeping the constant pins
  in the file whose module-documentation announces "Run-mode constant contracts")**:
  1. `watch_link_message_and_url_are_pinned` —
     `assert_eq!(WATCH_LINK_MESSAGE, "public watch link for the live stream");` and
     `assert_eq!(public_watch_url(BROADCAST_ID_FIXTURE),
     "https://www.youtube.com/watch?v=broadcast-id-fixture");`
     (imports: `use super::broadcast::public_watch_url; use super::gate::WATCH_LINK_MESSAGE;
     use super::runtime_test_fixtures::BROADCAST_ID_FIXTURE;` — cross-sibling `pub(super)` access
     is the established gate_tests/fixture precedent).
  2. `derived_watch_url_carries_no_secret_material` — assert the URL contains
     `BROADCAST_ID_FIXTURE` and `WATCH_URL_PREFIX`; assert it does NOT contain `rtmps`,
     `stream-name-fixture`, `bearer`, or `access-token`. (The negative guards are structural
     defense-in-depth for the secret-audit surface, not a redaction regression risk.)
- **Mechanical edits (counted, no new tests)**:
  - `gate_tests.rs`: the `gate_plan` helper gains
    `watch_url: public_watch_url(BROADCAST_ID_FIXTURE),` (fixture-shaped derivation through the
    real builder — the plumbing compiles and runs under the existing 3 zero-sleep gate tests).
  - `runtime_tests.rs`: TD-1 `run_runtime_orchestrates_credentials_then_resources_then_streaming`
    gains ONE assertion: `assert_eq!(recorded.watch_url,
    public_watch_url(BROADCAST_ID_FIXTURE));` (the recorded-capture evidence for the emitted URL
    value).
  - `runtime_runner_fixture.rs`: `RecordedExecution` gains `watch_url: String`;
    `RecordingStreamingRunner::run` captures `execution.watch_url.clone()`.
- Expected delta: **+2 new tests → 357 predicted (the OBSERVED gate log count is authoritative; if
  the arbiter counts differ, record the observed arithmetic in the adherence report).**
- Everything stays offline: no fixture material change (`broadcast-id-fixture` reused), no network,
  no real FFmpeg, no subscriber state. Test-attributable drift risk: NONE (constants + owned data).

### D-T2W-6 — Docs updates (Task-2 zone per D-GF2/TODO bullet 37)

1. `docs/production-run.md` — "What the Logs Look Like (Without Secrets)" table gains row 5
   (directly after row 4 `stream confirmed active on YouTube`; markdown table, same 3-column
   shape):
   `| 5 | public watch link for the live stream | carries the non-secret watch_url field —
   https://www.youtube.com/watch?v=<broadcast_id>, the watch URL built from the broadcast id (an
   unlisted broadcast is watchable by anyone holding the link; unlisted is not private) |`
   (backtick the code spans in the actual edit; keep the row ordering 1→5).
   Also, manual procedure step 5 (the "Expected startup output (order only, no secrets)" line):
   append the watch-link line to the event list — "the `public watch link for the live stream`
   line carrying the public watch URL (the watch page of the noted broadcast_id, non-secret; an
   unlisted broadcast is watchable by anyone holding the link)" — and verify-only the rest of the
   step (do not rewrite step framing, the operator-status note, or steps 1–10 beyond this list).
   The section intro sentence ("via `tracing` on stderr; stdout carries only the final stop
   line") already covers the new row — verify and leave byte-identical unless it contradicts.
2. `README.md` — the Production Run section's log paragraph (line ~124) DOES reference the event
   list, so it gains the watch-link event in the same inline style, e.g. after the confirmation
   event in the parenthetical list: "... , `stream confirmed active on YouTube`, then
   `public watch link for the live stream` carrying `https://www.youtube.com/watch?v=<broadcast_id>`
   — a non-secret URL built from the broadcast id (an unlisted broadcast is watchable by anyone
   holding the link); those are all never-secret". Keep the surrounding never-secret claims,
   Ctrl+C, and Studio-guidance sentences intact; no TOC change (no new section).
3. Docs wording constraint (D-T2W-9): docs describe the URL as "the watch URL built from the
   broadcast id" (or the full constant with `<broadcast_id>` placeholder); they must NOT claim a
   NEW official citation for the watch-URL format itself — the only guarantee cited in docs/tasks
   is the recorded broadcast-id==video-id research fact (§3).
4. NOT touched in Task 2: `docs/google-cloud-prerequisites.md`, `docs/authentication.md`,
   `.agent/project-info/context.md` (Task 1's zone, already landed), `docs/configuration.md`,
   `docs/demo.md`, `docs/build*.md`.

### D-T2W-7 — No version, no dependency, no exit-code, no `demo`/`auth` action

- Version stays 0.9.0 (bump landed at workflow step 3 — not Task 2's action).
- `Cargo.toml`/`Cargo.lock` untouched (no new dependency; D-T3-13 analog).
- `RunError`, exit codes, `run_command` arms, `RUN_STOP_MESSAGE`/`RUN_STARTUP_STOP_MESSAGE`,
  `cli.rs`, `auth/`, `demo/` — byte-unchanged; the frozen D-T3-8 contracts hold.
- Stop messages and the stdout reservation are untouched; no `Display`/`Debug` derive added.
- Startup stages 1–9 and the shutdown order stay byte-equivalent in behavior. Project-structure
  wording: only the `app/run/` map lines that name test counts / the touched files are refreshed
  (§6); `tech.md` needs NO Task-2 update (no stack/dependency change; recorded ruling — AC 54's
  clause activates only "where the changes touch them").

### D-T2W-8 — Verification & artifacts (and the NO-VM-smoke ruling)

- Standard verification battery per the composition pattern (§10 of the Task-3 plan, Alpine VM MCP,
  `vm_status` FIRST, never chained commands):
  - `cargo test --locked app::run` (spot-run: 17+2 = 19 expected for the run suite, observed
    authoritative), `cargo fmt --check`, `cargo clippy --locked -- -D warnings`;
  - full gate `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh
    scripts/dev-checks.sh` → expect exit 0, four PASS lines, `ALL CHECKS PASSED`,
    `logs/checks/<UTC>.log`, observed total recorded (357 predicted).
- Both release artifacts rebuilt inside this task cycle (D-GF5): `sh scripts/build-linux.sh` and
  `sh scripts/build-windows.sh` on the FINAL code tree; `ls -l` of `dist/rust-youtube-streamer-service`
  and `dist/windows/rust-youtube-streamer-service.exe` recorded with exact observed byte sizes
  (sizes change with code — never doc-hardcoded; logging the observed values in the 4.6 evidence is
  the standing convention).
- **Ruling — NO VM CLI smoke of the confirmation path**: the confirmation-gate path cannot run
  offline end-to-end (it needs real credentials and a real Google session); no CLI invocation makes
  ANY live claim. The pinned offline tests (constants + recorded capture + gate zero-sleep tests)
  are the emitted-line evidence; the existing CLI surface is untouched, so no `--help`/usage re-probe
  is needed either. Record this as the honest verification-limitation statement in 4.6 evidence
  (native/host smoke remains operator territory; `dist/` sizes are build facts, not runtime proof).

### D-T2W-9 — Derivation guarantee chain (recorded, one way)

- The facts of record: (a) the official liveBroadcasts reference, as transcribed in the binding
  Phase-05 Task-1 plan §1.3 (page last updated 2026-09-14), states the broadcast `id` equals the
  corresponding `video` resource id; (b) therefore the watch page of the streamed broadcast is the
  watch URL whose `?v=` parameter is that id; (c) the operator-facing URL text (`WATCH_URL_PREFIX`)
  is a code constant, single-sourced and grep-audited (D-T2W-4).
- Docs/prose wording is constrained to "the watch URL built from the broadcast id" — no NEW
  official-citation claim for the `https://www.youtube.com/watch?v=` form is introduced anywhere in
  Task-2 artifacts; if any reviewer demands a fresh official citation for the URL FORMAT, escalate
  to the caller rather than assuming one.

### D-T2W-10 — Line-cap / shape watch items (pre-analyzed)

- `src/app/run/mod.rs`: 182 → ~185 (the +3 lines: the derivation `let` after the prepared info
  event, the `GatePlan` field entry, and the `StreamingExecution` field entry); `run_runtime` body
  stays ≤ 40 lines (< 50 ✓). The 200-cap headroom ≈15 lines is a recorded WATCH ITEM: if any
  additional mod.rs line is needed beyond this design, split, do not squeeze.
- `gate.rs`: 114 → ~121; `broadcast.rs`: 37 → ~50; `streaming.rs`: 104 → ~110;
  `runtime_runner_fixture.rs`: 114 → ~118; `runtime_tests.rs`: 154 → ~156;
  `mod_tests.rs`: 17 → ~40; `gate_tests.rs`: 74 → ~76. All far under 200.
- No file approaches a fixed cap; no test-file split is required.

## 3. Derivation guarantee chain — single-source recap

```
official liveBroadcasts reference (last updated 2026-09-14)
  -> "id: string — broadcast ID (== the corresponding `video` resource ID)"
  -> transcribed in the binding Phase-05 Task-1 plan §1.3
  -> consumed here: https://www.youtube.com/watch?v=<broadcast_id>
  -> code constant WATCH_URL_PREFIX (single grep-audited definition site)
  -> emitted as the watch_url field of ONE info! event in gate.rs
```

No other surface may derive, format, or re-assemble the watch URL (grep-audit per D-T2W-4).

## 4. File plan (exact; deltas define the permitted diff surface)

| # | File | Action | Notes |
|---|---|---|---|
| W1 | `.kilo/plans/20261014-phase05-1-task2-watch-link-print.md` | NEW (this file) | tracked first (commit 1) |
| W2 | `src/app/run/broadcast.rs` | EDIT | `WATCH_URL_PREFIX` + `public_watch_url` (D-T2W-2; ~+13 lines) |
| W3 | `src/app/run/mod.rs` | EDIT | extend the existing `use broadcast::{prepare_broadcast, StageRuntime};` braces list with `public_watch_url`; derive `let watch_url = …` after the prepared info event; add `watch_url` to BOTH param objects (`GatePlan`, `StreamingExecution`) (D-T2W-2/D-T2W-3; order before `into_ingestion`) |
| W4 | `src/app/run/gate.rs` | EDIT | `GatePlan.watch_url`; `pub(super) const WATCH_LINK_MESSAGE`; the Ok-arm emission `info!(watch_url = %plan.watch_url, "{}", WATCH_LINK_MESSAGE);` immediately after the confirmation event; module-doc sentence noting the watch-link emission (one line) |
| W5 | `src/app/run/streaming.rs` | EDIT | `StreamingExecution.watch_url` field + destructured `_watch_url` + the ride-along doc comment (mirror of `stream_id`) |
| W6 | `src/app/run/runtime_runner_fixture.rs` | EDIT | `RecordedExecution.watch_url` + capture line |
| W7 | `src/app/run/mod_tests.rs` | EDIT | +2 pinned tests (D-T2W-5); doc-comment line extended to the constants class |
| W8 | `src/app/run/gate_tests.rs` | EDIT | `gate_plan` helper gains the derived fixture URL (mechanical) |
| W9 | `src/app/run/runtime_tests.rs` | EDIT | TD-1 gains the recorded watch-url assertion |
| W10 | `docs/production-run.md` | EDIT | log-table row 5 + manual step 5 event-list line (D-T2W-6) |
| W11 | `README.md` | EDIT | Production Run log-paragraph event list gains the watch-link event (D-T2W-6) |
| W12 | `.agent/project-structure.md` | EDIT | app/run/ map: broadcast/gate/streaming descriptions mention the watch-link derivation/emission/carriage; test count 17 → 19 on the `*_tests.rs` line; NO new-file entry (ruling: no new file) |

Explicitly NOT touched: `src/main.rs`, `src/app/cli.rs`, `src/app/error.rs`, `src/app/logging.rs`,
`src/app/mod.rs`, `src/app/auth/**`, `src/app/demo/**`, `src/app/pipeline/**`, `src/config/**`,
`src/chat/**`, `src/renderer/**`, `src/streaming/**`, `src/youtube/**`,
`src/app/run/{seams.rs, error.rs, credentials.rs, source.rs, streaming_stage_tests.rs,
error_tests.rs, credentials_tests.rs, runtime_test_fixtures.rs, runtime_credential_fakes.rs}`,
`Cargo.toml`, `Cargo.lock`, `Dockerfile`, `docker-compose.yml`, `scripts/`, `config/`,
`docs/configuration.md`, `docs/authentication.md`, `docs/demo.md`, `docs/build.md`,
`docs/build-windows.md`, `docs/google-cloud-prerequisites.md`,
`.agent/project-info/{tech.md, brief.md}` (context.md update is the 4.6 completion action),
`.gitignore`.

## 5. Drift guards (so reviews can adjudicate fast)

- The existing `broadcast prepared for streaming` line, its `broadcast_id` field, the two stop
  message lines, and the exit contract remain byte-stable (`mod_tests.rs` existing test unchanged).
- The confirmation event `stream confirmed active on YouTube` stays an inline literal ONE line
  above the new emission — no reordering, no rewording of that event (docs row 4 stays valid).
- The ONE `into_url` production call site count stays 1 (D-T2W-4 audit).
- The `debug`/`Display` surface deltas: ZERO (no derive, removed, or added).

## 6. Docs zone recap (single source list)

- production-run.md edits: ONLY the log-table row 5, the manual step 5 event-list sentence, and
  verify-only checks of the section intro and the standing secret-guarantees paragraph (which
  already hold and gain nothing new).
- README.md edit: ONLY the Production Run log paragraph.
- project-structure edit: ONLY the app/run/ map lines + test-count refresh (W12).

## 7. Redaction/regression cross-check vs the global plan

- D-GF1 honored: non-secret URL only; redaction guarantees untouched (D-T2W-4).
- D-GF5 honored: artifact rebuilds inside this task cycle; branch/version work stays at workflow
  steps 2–5 (none here).
- D-T3-7/D-T3-8/D-T3-11/D-T3-14 consumed unchanged (whitelist gains exactly one field; exit codes
  and stop messages frozen; the grep-audit list extends, D-T2W-4).

## 8. Implementation Steps (4.2; sequential; Alpine VM MCP; vm_status FIRST; never chained)

- S0 Preconditions: `git status --porcelain` empty; `git branch --show-current` returns
  `feat/phase05-1-operator-feedback`; `git log --oneline -3` shows HEAD at/after `6d3441f`;
  `Cargo.toml` reads version 0.9.0. NO version/dependency action. gitignore-compliance sanity
  (nothing in the planned set is ignored; `dist/`, `logs/checks/`, `videos/` stay untracked).
- S1 Commit this plan (§9 commit 1).
- S2 Code edit group (W2–W9) in one working pass:
  1. `broadcast.rs`: add the constant + `public_watch_url` (exact body, D-T2W-2).
  2. `gate.rs`: add `WATCH_LINK_MESSAGE`; add `GatePlan.watch_url`; add the Ok-arm emission line
     AFTER the confirmation `info!` (exact form, D-T2W-1); extend the module doc sentence.
  3. `streaming.rs`: add `StreamingExecution.watch_url` with the ride-along comment; destructure
     `_watch_url`.
  4. `mod.rs`: extend the `broadcast` import (it already imports `prepare_broadcast, StageRuntime`
      via `use broadcast::{prepare_broadcast, StageRuntime};` — extend THAT braces list, do not add
      a second `use` line); derive the URL after the prepared info event; thread `watch_url` into
      `GatePlan` and `StreamingExecution` (clone semantics: `watch_url.clone()` for the gate +
      move into the bundle; emission shape per D-T2W-1).
  5. Fixtures/tests: W6 recording capture; W7 two pinned tests; W8 gate_plan builder; W9 TD-1
     assertion.
- S3 Compile/behavior checkpoint via compose: `cargo clippy --locked -- -D warnings` and
  `cargo fmt --check` (if fmt fails: mechanical `cargo fmt` ONLY — never restructure). Unexpected
  compiler/clippy output → pause and return to the caller (planner ruling), per the
  no-improvisation rule.
- S4 Spot battery (separate invocations): `sh -c "cargo test --locked app::run"` (19 expected) and
  the FULL `sh scripts/dev-checks.sh` gate → record the log path + observed totals (357 predicted).
- S5 Artifacts: `sh scripts/build-linux.sh` then `sh scripts/build-windows.sh`; `ls -l` both
  dist entries; record exact observed sizes (D-T2W-8).
- S6 Docs group (W10, W11) exactly per §7; no other doc sentence is rewritten.
- S7 Hygiene: `git status --porcelain` shows ONLY the expected working set (W2–W11 + this plan);
  `git ls-files dist/` empty; no `config.json`/tokens/keys/MP4s staged; run the three
  watch-link grep audits (D-T2W-4) and record outputs.
- S8 Commit groups (§9) — never amend a landed commit; green-at-each-point binds (each commit
  compiles and keeps its touched suites green: the code commit precedes any docs commit for the
  same suite behavior, and the docs commits are markdown-only).
- S9 project-structure map (W12) — its own commit (§9 commit 4), after the tree is final.
- S10 `.agent/project-info/context.md` update is a 4.6 COMPLETION action (not 4.2).

## 9. Commit Plan (implementer; feature-branch commits only — NO push, NO merge, NO bump)

```text
1. docs: track the phase 05.1 task 2 watch-link plan            (S1)
2. feat: print the public watch link once the stream is confirmed active (S2–S5: code W2–W9;
   body notes: WATCH_LINK_MESSAGE + watch_url field; broadcast-id = video-id per the recorded
   Task-1 research; NO secret surface touched; artifacts rebuilt with observed sizes)
3. docs: document the watch link in the run guide and readme    (W10 + W11)
4. docs: map the watch-link additions in project structure      (W12)
```

Green-at-each-point binding: commit 2 holds the full gate evidence from S4/S5; commits 3–4 are
markdown-only (no re-run required unless a reviewer orders one — the standing gate log covers
compilation-neutral trees). Commit wording may adapt to the observed diff shape per the Task-3
precedent; the substance rules above do not.

## 10. Verification Evidence Ledger (what 4.5b/4.6 must show)

| Check | Expected | Source |
|---|---|---|
| Branch / HEAD / version | feat/phase05-1-operator-feedback; ≥ 6d3441f; 0.9.0 | S0 commands |
| Full gate | exit 0; four PASS; ALL CHECKS PASSED; observed total (≈357) | S4 log path |
| run-suite spot run | 19 (observed authoritative) | S4 |
| Linux artifact + size | dist/rust-youtube-streamer-service, exact observed bytes | S5 `ls -l` |
| Windows artifact + size | dist/windows/rust-youtube-streamer-service.exe, exact observed bytes | S5 `ls -l` |
| `git grep -n "watch?v" src/` | exactly 2 hits (WATCH_URL_PREFIX + the pinned-test literal) | S7 |
| `git grep -in "rtmps\\|stream key\\|ingest" src/app` | baseline hits only (fixtures/docs) | S7 |
| `git grep -n "println!\\|eprintln!" src/app/run/` | the two existing stop lines only | S7 |
| `git grep -n "derive(Debug" src/app/run/` | unchanged from baseline | S7 |
| `git ls-files dist/` | empty | S7 |
| CLI smoke of the confirmation path | NOT RUN (ruling D-T2W-8: offline pinned tests are the evidence; no live claim) | recorded statement |

## 11. Adherence Expectations (4.3–4.5b reviewers)

- Diff surface vs §5's W-table; every file listed "Explicitly NOT touched" has an EMPTY diff
  (especially `src/streaming/**`, `src/youtube/**`, `src/app/pipeline/source.rs`, `Cargo.*`).
- Emission point/ordering verified by source read of `gate.rs` (D-T2W-1 review-verified item):
  the watch-link `info!` sits in the `Ok(())` arm, immediately after the confirmation event, before
  `GateFinish::Confirmed`; the Cancelled/Fatal arms gain nothing.
- Constants pinned by the two new tests EXACTLY as D-T2W-5; the `"{}"`-format emission form present
  (single-source rule); no subscriber-capture machinery introduced.
- The `"{}"` + `WATCH_LINK_MESSAGE` form is reviewed for compile-fidelity at S3: if the compiler
  rejects it (not expected), pause and escalate — do not substitute a different macro form or an
  inline literal silently.
- Secrets audit re-executed (D-T2W-4 list); the ONE-grep rule for `watch?v` holds; `Debug`/`Display`
  deltas zero; `demo`/`auth` behavior byte-unchanged.
- Docs match the code (row text and README sentence reference the same constant values; the unlisted
  caution present; no official-URL-format citation invented, per D-T2W-9).
- project-structure map matches the tree; test counts reflect the observed gate.
- Observed-total rule: gate log test count is authoritative over the 357 estimate.

## 12. TODO Task-2 Bullet — Plan Mapping Self-Check

| TODO Task 2 bullet | Plan coverage |
|---|---|
| Print the PUBLIC watch URL `https://www.youtube.com/watch?v=<broadcast_id>` after the gate observes `streamStatus == active` as an operator-visible event; `broadcast prepared for streaming` keeps `broadcast_id` | D-T2W-1 (point/order/text), D-T2W-2 (derivation), D-T2W-3 (carriage); existing event untouched |
| NEVER print the stream key or the full RTMPS URL; no Display/Debug weakenings; no exit-code or demo/auth drift | D-T2W-4 (enumerated audit), D-T2W-7 (frozen contracts), §6 drift guards |
| Pinned offline test(s) for the exact line text | D-T2W-5 (+2 tests in mod_tests.rs; recorded-capture URL pin) |
| Update docs/production-run.md log table and README line list if it references events; both release artifacts rebuilt with `ls -l` sizes | D-T2W-6 (zones + exact rows), D-T2W-8 (rebuild + sizes + the no-smoke ruling) |
| Out of scope (none of: redaction changes, shutdown order, broadcast-fate policy, exit-code contract, live E2E) | §13 below; all frozen |

## 13. Explicitly Out-of-Task (restated for the implementer)

- NO Hardening implementation (bounded FFmpeg wait, ureq timeout, watchdog, supervisor) — future
  planning only (D-GF6; Task 3 owns the report addendum).
- NO Live Chat artifact; `src/app/pipeline/source.rs` stays byte-unchanged.
- NO redaction/shutdown/fate/exit-code change beyond what the watch-link print requires.
- NO live Google/YouTube call, NO claim of live success, NO operator E2E execution (on hold ~24 h —
  Task 1's recorded note is not re-verified here).
- NO version/lockfile action, NO push/merge/TODO-rename (workflow steps 5–6).
- NO tracing-subscriber test helper, NO `RunError` variant, NO stdout print of the watch link.

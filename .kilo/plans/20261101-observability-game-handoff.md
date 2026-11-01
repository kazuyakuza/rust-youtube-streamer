# Global Plan — Configurable Observability and Game-Foundation Handoff

Source TODO: `.agent/todos/20261031/20261031-todo-1.md` (Pattern C: 3 tasks under `## Tasks`).
Date: 2026-11-01. Base: `main` at v0.13.1 (`20c8a3d`), working tree clean.
TODO does NOT contain "Don't request me to approve plans" → user approval IS required (both for this global plan and per-task plans unless the user selects auto-approve).

## Workflow Step Mapping (steps 2–6; each task gets its own 4.1–4.6 cycle)

- Step 2: Git Feature Branch Setup => implementer (branch `feat/observability-game-handoff`)
- Step 3: Version Update => implementer (0.13.1 → 0.14.0, minor: new observability capability; commit `chore: bump version to 0.14.0`)
- Task 1: 4.1b Analysis & Planning => architector → 4.2 Implementation => implementer → 4.3 Code Review & Simplification => code-reviewer + code-simplifier (fixes => implementer) → 4.4 Documentation => docs-specialist → 4.5b Overall Plan Adherence => architector → 4.6 Task Completion => implementer
- Task 2: 4.1b => architector → 4.2 => implementer → 4.3 => code-reviewer + code-simplifier → 4.4 => docs-specialist → 4.5b => architector → 4.6 => implementer
- Task 3: 4.1b => architector → 4.2 => implementer → 4.3 => code-reviewer + code-simplifier → 4.4 => docs-specialist → 4.5b => architector → 4.6 => implementer
- Step 5: TODO File Completion => implementer (rename `-DONE`, clean tmp, merge to `main`, push `origin` ONLY)

Front-end determination: ALL THREE TASKS are non-front-end (Rust CLI/service; no UI). Sub-steps 4.1a/4.5a are therefore NOT executed for any task.

## Global Pre-Analysis (technical & architecture decisions)

- Current production path (locked facts from `.agent/project-structure.md` + context): chat worker (`src/app/run/chat_worker.rs` + `chat_stream_worker.rs`/`chat_stream_delivery.rs`, confined current-thread Tokio runtime, tonic 0.13.1 `streamList` gRPC, std::panic::catch_unwind containment) → bounded `std::sync::mpsc::sync_channel` capacity 64 (`try_send`-only delivery, full-channel retain-retry) → `ChatSource` (`src/app/run/source.rs`, non-blocking `try_recv` draining) → sync frame loop (`src/app/pipeline/runner.rs`, continuous frames at configured FPS, at most ONE new message consumed per frame) → `TextRenderer` → `FrameSink` → `FfmpegProcess` stdin (`src/streaming/`) → RTMPS (CBR-bearing locked 33/42-token argv vector).
- Architecture decision (constraint): instrumentation must be READ-ONLY observation layered onto the existing pipeline — no channel-capacity change, no unbounded queue, no silent drop, no shutdown-order change, no dependency addition unless the Task-1 plan proves a concrete need. Preferred stack: existing `tracing` + `std::time::Instant` (monotonic) + hand-written bounded aggregation structures.
- Configuration decision: new `diagnostics` config section (architector names exact fields), strictly validated under the existing `deny_unknown_fields` + named-predicate conventions (current count: 21 rules — architector will recut the count); disabled-by-default; update `config/config.example.json` + `docs/configuration.md` + test rosters.
- Testing environment decision: all automated checks run via the Alpine VM MCP (`alpine-vm_vm_status` first, then `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`); gate logs land in gitignored `logs/checks/`; release artifacts (if the plan requires rebuilds) via `scripts/build-linux.sh` / `scripts/build-windows.sh` in the same service with `ls -l` size cross-checks.
- Repo-host reality: git operations run on the Windows host working tree (`C:\repo\rust-youtube-streamer`); container checks see the same bind-mounted tree at `/rust-youtube-streamer`. Never create scratch dirs outside the workspace.
- Incremental TODO-file commit discipline (established precedent): per-task `chore: track ... plan` commits for plan files; 4.3 fix commits; 4.4 docs commits; 4.6 DONE-mark commits; step-5 rename/merge/push restricted to their owners.
- Version: bump once at step 3 to 0.14.0 (features); Task-2/3 do not bump again.
- Secret-safety invariant for all tasks: no chat body text, tokens, client secrets, RTMPS URLs, or stream keys in diagnostics/logs/docs; redaction and one-line CLI error contracts stay byte-stable.

## Per-Task Pre-Analysis

### Task 1 — Opt-in, low-overhead pipeline observability (non-front-end)

Scope: instrument the production chat-to-frame-to-FFmpeg path; operator-facing diagnostics config; bounded aggregation; optional separate diagnostic file with bounded rotation; unit tests (enabled/disabled/invalid/aggregation/backpressure); docs.

Technical decisions to encode at 4.1b:
- Instrumentation points (where events are technically available): (1) provider `publishedAt` wall-clock already parsed by `live_chat_rfc3339`/`live_chat_mapping` → carried on `ChatTextMessage`/`ChatMessage` correlation fields; (2) local `streamList` batch receipt in `chat_stream_delivery.rs` (monotonic); (3) enqueue acceptance/full-channel events in the worker's `try_send` sites; (4) dequeue in `ChatSource::next_message` / runner consumption; (5) render completion containing the message in `runner.rs`; (6) FFmpeg stdin frame-write start/completion around the runner's `write_frame`.
- Correlation: provider message id preferred (present on accepted text messages); local fallback only if the provider id is genuinely unavailable — for metrics summaries use COUNTS + aggregate durations, never message text; no new user-identifying data.
- Clock discipline: monotonic `Instant` for local durations; module must refuse cross-domain subtraction without recording the clock-skew caveat where wall-clock/provider timestamps are correlated; test with injected/controllable clock values, zero sleeps.
- Aggregation: bounded in-memory distribution records (count/min/max/mean + fixed histogram buckets or percentile arrays); periodic summary emission (interval configurable, e.g. seconds-range validated); high-water mark, enqueue wait/failure/full events, oldest-queued age, received/rendered per interval, reconnect count/duration; distinguish worker-retained-during-backpressure from enqueued.
- Frame pipeline: frame-loop interval/jitter, render duration, write duration; NO per-frame log lines in normal operation.
- Diagnostic file: configurable path, separate from `chat.log`, bounded size/rotation (explicit cap; rotation policy chosen by architector); disabled default ⇒ no file is ever created and non-summary high-frequency formatting is compiled out/guarded.
- Config validation conventions + example + docs; disabled-all ⇒ `auth`/`demo`/`run` behavior unchanged and no artifact created.
- Line caps (≤200/file, ≤2 args/method unless parameter-object, ≤50-line bodies, ≤2 nesting) may force rosters/sibling splits — precedent: caller-ruled formal adjudication of any STOP condition.

Acceptance criteria live in the TODO (pp. 57–64); Task-1 4.6 flips §Task-1 criteria only after evidence.

### Task 2 — Reproducible latency, throughput, backpressure test maps (non-front-end)

Scope: deterministic offline tests around the real queue/consumer seams; a repeatable offline load/stress harness injecting controlled rates/bursts (no credentials, no broadcast); operator-run live measurement procedure + results template + interpretation matrix in docs.

Technical decisions to encode at 4.1b:
- Test targets: `ChatSource` + worker channel delivery sites + frame-loop consumption (reuse existing offline doubles: `runtime_chat_fakes.rs`, `RunChatStreamLedger`, `stream_test_fixtures.rs` patterns; never Google).
- Harness shape: a `#[cfg(test)]`-only (or docs-supported binary-adjacent) injection utility pacing sends by configured FPS/capacity (never hardcoded 30); capacity/FPS read from config; scenario param sets (low-rate, burst-fill, sustained below/near/above nominal 1-message-per-frame capacity, recovery, bounded memory, queued-messages shutdown).
- Thresholds: diagnostic/regression guards ONLY when justified by measured behavior; synthetic ≠ YouTube end-to-end statement carried into every output/template.
- Operator docs: manual stopwatch / consistent-clock method, repeats, latency-mode + playback condition recording, message-id correlation with diagnostics events, observability on/off overhead comparison, results template (run/build id, date/time, FPS, diagnostics settings, rate/burst profile, queue metrics, internal timing summaries, external viewer delay, notes), interpretation matrix (upstream delivery / application queue / frame-render / FFmpeg+ingest / viewer playback — which the app measures directly vs externally inferred).

### Task 3 — Game-foundation handoff documentation (non-front-end)

Scope: docs + project-info only; NO game mechanics, NO code/behavior change.

Technical decisions to encode at 4.1b:
- New guide file under `docs/` (architector picks name, e.g. `docs/game-foundation.md`), ≤ appropriate 100-line TOC threshold rules; README + production/dev docs cross-links; no duplicated operator procedures (link canonical).
- Content per TODO pp. 109–150: executable-mode responsibilities (`auth`/`run`/`demo`), real source-module data flow with actual module paths, reusable vs extension-point components, explicit "game logic must not depend on YouTube API types/OAuth/broadcast ids/RTMPS/FFmpeg args", recommended future flow, separation lines, 10-item agent guidance, timing/scale constraints (35 s observation framed as measurement), decision checklist, "what not to change".
- Metadata: update `.agent/project-info/architecture.md`, `tech.md`, `context.md` when lasting facts/next steps emerge; keep `AGENTS.md` links accurate; update `.agent/project-structure.md` only if the tree changes (Task 3 should not change source layout).

## Risks (global)

- R1: Instrumentation must not perturb the locked argv vectors, shutdown order, exit-code contracts, or demo/auth behavior — Task-1 plan MUST carry a byte-stability ledger (precedent: stream-health Task 2).
- R2: File/line caps for new code — plan must pre-assign rosters and anticipate STOP→caller-ruling splits.
- R3: VM availability — sub-agents must call `alpine-vm_vm_status` before any compose command; on VM failure, pause and report to the user.
- R4: The 4.3 max-3-review-cycles rule; escalate to user on the third failure.
- R5: `deny_unknown_fields` means example/json fixtures used by `loader_tests.rs` (at its 200-line cap) must be re-cut when the diagnostics section lands — plan must name the exact fixture/split strategy.
- R6: live claims stay operator-run everywhere; no test may contact Google; secret-pattern audits (`ya29`/`AIza`/`key=`/`rtmps://`) must remain at pre-existing doc-context counts.

## Approval

Presented to the user via `question` tool: (a) Approve Global and Tasks Plans (auto-approve per-task plans), or (b) Approve Global Plan (per-task plans presented for approval).

# Global Plan — Run Streaming Diagnostics + Per-Stream Made-for-Kids Config (conditional)

Plan date: 2026-10-22 · Owner: Planner Agent (Critical Workflow)
TODO: `.agent/todos/20261022/20261022-todo-2.md` · Type: FIX+FEATURE (semver MINOR 0.9.1 → 0.10.0) · Front-end: NO

## Global Pre-Analysis

Incident chain (two operator run attempts, both native Windows, both operator-supplied log facts — never agent-observed live):

1. Attempt 1 (03:35, v0.9.0, `20261022-033554.log`): HTTP 411 at the bind — FIXED and closed earlier today (v0.9.1, `fix/http-411-bind-content-length`, merged `2317200`, pushed).
2. Attempt 2 (23:17, v0.9.1, `20261022-231740.log`): `stored authorization verified` → `broadcast prepared for streaming broadcast_id="xZTbF0bDRII"` (O1 insert + O2 insert + O3 bind + readiness wait SUCCEEDED — the 411 fix is confirmed on real Google; the stream is visible in the operator's YouTube Studio) → `FFmpeg started toward the configured streaming destination` → 120 s confirmation budget exhausted → `error: YouTube readiness wait timed out (stage 'ingestion-confirmation')` → exit 1. Studio shows "No data" / "Start sending us your video from your streaming software to go live" — YouTube never registered ingest.

Failure-shape deduction (code-verified):

- `app/run/gate.rs` records `GateFinish::Fatal` only when the wait runs to its full budget without cancellation; `app/run/streaming.rs` cancels the gate BEFORE the join on a pipeline fatal, and a cancelled gate lets the pipeline error become the reported error. The reported error IS the gate timeout → the frame pipeline never failed → FFmpeg stayed alive and consumed stdin frames for the entire 120 s while YouTube received nothing. FFmpeg-side instant failures (missing rtmps TLS support, missing libx264, publish rejection) all EXIT the process and would surface as pipeline errors — ruled out by this log shape.
- The app therefore lacks the observability to explain the remaining failure mode: (a) FFmpeg's captured stderr (which carries its per-frame progress lines and any RTMP/TLS diagnostics) and its exit code are captured (drain threads + 4096-byte tail + destination-URL redaction already exist in `streaming/ffmpeg.rs`/`pipe_drain.rs`) but surfaced ONLY inside an `ExitFailure` error payload — on the gate-outranking path the evidence is dropped; (b) the confirmation wait (`readiness_wait/confirmation_wait.rs`) discards the last observed `streamStatus` label at timeout — "status never left `ready`" is the key discriminator.
- Brief §11.1 explicitly requires logging "FFmpeg startup, exit, and diagnostic errors" — the exit is currently unlogged on run paths. This is an honest compliance gap, not gold-plating.
- Made-for-kids: the operator saw the Studio MFK popup (channel advanced setting "review every video") around the live stream section; the operator has since set the channel default to "not made for kids" but wants per-stream control. Candidate API field: `status.selfDeclaredMadeForKids` (bool) on the `liveBroadcasts` resource (settable on insert; `madeForKids` is the output-only mirror) — the architector MUST verify against the CURRENT official documentation before any implementation (user-ruled condition: only if it fits the start-stream request simply).
- Operator-side follow-up (communication, not agent work): both failed runs left orphaned `ready` broadcast/liveStream pairs in YouTube Studio (the 03:35 attempt and the 23:17 attempt); they must be deleted per `docs/production-run.md` §7 before further attempts. A cheap operator probe is also available: since the channel default is now "not made for kids", a plain re-run on v0.9.1 tests the MFK-holder hypothesis for free — but the diagnostics (Task 1) and the per-stream config (Task 2) are wanted regardless of that probe's outcome.

## Encoded Technical Direction (Task 1 — diagnostics)

- FFmpeg exit + stderr tail surfacing: plumb the already-captured, redacted stderr tail and the exit status from the streaming `finish()` path into the run-mode reporting (typed carrier through the stream completion type — the `CompletedOutput::Stream` variant currently carries no data), and emit info/warn tracing lines from the APP layer (keeps the streaming module's std+config-only boundary intact; tracing stays app-side). Non-zero exit during a run → a `warn` diagnostic line (never a second `error:` line — the single-`error:`-line contract holds); clean exit after a confirmed or cancelled session → `info`.
- Last observed streamStatus at gate timeout: in `app/run/gate.rs`, on the `Fatal(ReadinessTimeout)` path only, perform ONE additional tolerated stream lookup (the gate thread owns the `Arc` transport; a lookup failure is logged as a fact, never fatal) and log the observed label via a non-secret info line. Do NOT change the shared wait machinery's signatures or its 14 tests; do NOT add fields to `LiveApiError::ReadinessTimeout` (Display is pinned).
- Redaction: every surfaced tail passes the existing `exit_diagnostics` redaction (destination URL → `[redacted]`); no new secret surface is created; the diagnostics lines never name the destination or the stream key.
- Contract stability: exactly ONE `error:` line per failure; exit codes unchanged; shutdown order unchanged; `demo`/`auth` byte-stable; offline tests only.

## Encoded Technical Direction (Task 2 — made-for-kids, CONDITIONAL)

- Research gate (executed in 4.1b, official citations required): `status.selfDeclaredMadeForKids` settable on `liveBroadcasts.insert`? Behavior when unset (channel-default fallback)? Only proceed if BOTH hold: (a) documented settable, (b) implementation = optional config field + one optional field in the existing O2 request body + command plumbing. Otherwise record-and-skip (documented finding, no code, docs note that the channel-level setting is the workaround).
- If proceeding: config field `youtube.broadcast.made_for_kids` (OPTIONAL bool; absent = field not sent = channel default), mapped through `LiveBroadcastPreparer::from_config` → `CreateBroadcastCommand` → the O2 body (`selfDeclaredMadeForKids`, skipped when absent); validation needs no new rule beyond serde's typed Option; config example + docs updated; offline pins for absent/true/false serialization; no other request or lifecycle change.

## Execution Map (steps 2–6; each task gets its own 4.1–4.6 cycle)

- Step 2: Git Feature Branch Setup => implementer — branch `feat/run-streaming-diagnostics-mfk` from `main` (clean, at `06849c7`, synced with origin)
- Step 3: Version Update => implementer — 0.9.1 → 0.10.0, commit `chore: bump version to 0.10.0`, lockfile refresh verified `--locked`
- Task 1: 4.1b Analysis & Implementation Plan => architector (no front-end → no 4.1a)
- Task 1: 4.2 Implementation => implementer
- Task 1: 4.3 Code Review & Simplification => code-reviewer + code-simplifier concurrently; 4.3-fix => implementer
- Task 1: 4.4 Documentation => docs-specialist
- Task 1: 4.5b Overall Plan Adherence => architector (no front-end → no 4.5a)
- Task 1: 4.6 Task Completion => implementer
- Task 2: 4.1b Analysis & Implementation Plan => architector (includes the research gate + the implement-or-skip verdict with citations)
- Task 2: 4.2 Implementation => implementer (body per the 4.1b verdict; SKIP path = docs/context record only)
- Task 2: 4.3 Code Review & Simplification => code-reviewer + code-simplifier; 4.3-fix => implementer
- Task 2: 4.4 Documentation => docs-specialist
- Task 2: 4.5b Overall Plan Adherence => architector
- Task 2: 4.6 Task Completion => implementer
- Step 5: TODO File Completion => implementer — `-DONE` rename, tmp hygiene, merge to `main`, push `origin` ONLY

## Risks and Mitigations

- The diagnostics may NOT reveal the ingest blocker by themselves (e.g., if FFmpeg's stderr shows healthy progress and YouTube held the stream for a non-MFK reason) — acceptable: the log then proves FFmpeg's side is healthy and narrows the next investigation to the YouTube-side state; the plan's docs must frame Task 1 as evidence-gathering, never as a claimed root-cause fix.
- Streaming-layer boundary: the redacted diagnostics carrier must not widen `streaming`'s dependencies (app-layer logging only); the architector encodes the carrier shape.
- Task 2's API field: if the official docs place restrictions (e.g., field ignored without additional settings, or requires a different part list), the research verdict governs — record-and-skip stays a first-class outcome, never a forced implementation.
- Quota/orphans: repeated failing runs keep creating orphaned pairs (documented limitation); the docs remind the operator to clean up before every retry.

## Amendment 1 (2026-10-22, post-approval, user-supplied)

The operator confirmed the 23:17 run (`logs/run/20261022-231740.log`) was ALREADY the retry made after changing the channel's made-for-kids advanced setting to "not made for kids" — the earlier run is when the Studio MFK popup was observed. Consequences: the cheap re-run probe above is MOOT (already performed, same gate-timeout outcome), and the "MFK popup blocks ingest" hypothesis is WEAKENED as the sole blocker. Task 2 (per-stream made-for-kids config) remains WANTED by the user as a capability regardless; its conditional research gate is unchanged.

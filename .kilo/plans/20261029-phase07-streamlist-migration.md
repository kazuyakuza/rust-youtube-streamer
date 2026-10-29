# Global Plan — Phase 07: YouTube Live Chat streamList Migration

> TODO (source of truth): `.agent/todos/20261029/20261029-todo-1.md` (Pattern C: 4 tasks under `## Tasks`).
> Baseline: `main` @ `346d109` (TODO tracked), v0.12.0, working tree clean, pushed through `origin/main` @ `e1b8d44` (TODO commit local). Test baseline: 458 passed / 0 failed.
> Convention: every step below is a SEPARATE `task` tool invocation; no sub-agent executes two steps.

## Front-End Determination

| Task | Front-end? | Reason |
| --- | --- | --- |
| 1. Official streamList Client-Path Research and Design | NO | Rust backend research/design task |
| 2. Streaming Client and Async Bridge Implementation | NO | Rust backend module (`src/youtube/`) |
| 3. Production Run and Frame-Pipeline Integration Switch | NO | Rust backend wiring (`src/app/run/`) |
| 4. Offline Tests, Documentation, Builds, and Manual Verification | NO | Tests/docs/builds/report |

⇒ No task executes 4.1a / 4.5a; only 4.1b and 4.5b apply.

## Steps 2–6 (global execution order)

- Step 2: Git Feature Branch Setup => implementer (`feat/phase07-streamlist-retrieval`)
- Step 3: Version Update (0.12.0 → 0.13.0, minor — feature phase) => implementer
- Task 1: 4.1b Analysis & Planning => architector
- Task 1: 4.2 Implementation => implementer
- Task 1: 4.3 Review & Simplification => code-reviewer + code-simplifier; fixes => implementer
- Task 1: 4.4 Documentation => docs-specialist
- Task 1: 4.5b Overall Plan Adherence => architector
- Task 1: 4.6 Task Completion => implementer
- Task 2: 4.1b / 4.2 / 4.3 (+fix) / 4.4 / 4.5b / 4.6 (same cycle shape)
- Task 3: 4.1b / 4.2 / 4.3 (+fix) / 4.4 / 4.5b / 4.6 (same cycle shape)
- Task 4: 4.1b / 4.2 / 4.3 (+fix) / 4.4 / 4.5b / 4.6 (same cycle shape)
- Step 5: TODO File Completion => implementer (rename `-DONE`, tmp cleanup, merge to `main`, push `origin` ONLY)
- Step 6: Finish => planner (summary + handoff)

Branch creation/switch RESTRICTED to step 2; version bump RESTRICTED to step 3; `git push` RESTRICTED to step 5.

## Global Pre-Analysis (technical & architecture decisions)

1. **Superseded decision, user-ruled.** Phase 06's D-1 (polling via `liveChatMessages.list`) is superseded by the USER RULING of 2026-10-29 (motivation: message-appearance latency under server-directed pacing + quota burn of constant per-interval calls). Task 1 re-verifies official docs (access-dated 2026-10-29 or later) and implements the ruled direction unless official facts STOP it.
2. **Client path is adjudicated, not assumed.** Three candidates: (A) official gRPC server-streaming (generated stub from the published proto; Rust ⇒ `tonic`+`prost`+`tokio`), (B) HTTP streaming over the existing sync `ureq` stack IF an officially documented REST-mirror wire format exists, (C) neither officially implementable ⇒ STOP → caller. Path A is the working hypothesis (officially documented; the REST mirror's wire semantics were found undocumented on 2026-10-15). The third-party (Gemini-pasted) directives are INPUTS to adjudicate — several claims conflict with project rules or lack official backing (`reqwest`+`bytes_stream` against an undocumented REST shape; "API key query param" vs the Bearer-only rule; async `tokio::sync::mpsc` between worker and frame loop; SuperChat/ban visibility vs the text-only policy).
3. **Toolchain gate BEFORE implementation.** If Path A is chosen: verify `tonic`/`prost`/`tokio` MSRV + Windows-GNU compatibility against the pinned `rust:1.82` image. No compatible combination ⇒ STOP → caller (never silently bump the pinned image; a pin change is a caller decision affecting the whole build).
4. **Async bridge rules (if Path A).** Runtime confined to ONE dedicated worker thread; the existing bounded std `sync_channel(64)` + `try_recv` source + frame loop stay byte-unchanged (the async task forwards into the existing channel); Ctrl+C `StopFlag` propagation reaches the Tokio task (select/poll with 1 s stop-slices equivalent); bounded join at gate finish; `tokio`'s TLS story for the gRPC channel (rustls vs native-troots) selected in Task 1 with MSRV evidence; no second runtime instance anywhere else.
5. **Resumability + semantics.** Stream established when chat is first needed (worker start at `GateFinish::Confirmed`); establish → consume → on drop/timeout re-establish with last `nextPageToken` + bounded backoff (reuse the existing 60 s cap ↔ 10-failure budget shape unless research dictates different documented bounds); 512 dedup window and no-history-replay survive reconnects; verify officially how streamList's initial deliveries interact with the baseline-discard requirement (empty display at start).
6. **Quota semantics honesty.** Record ONLY documented billing facts (the 5-units floor; establishment/re-establishment costs; any documented per-event accounting); everything undocumented is marked unknown — the D-10 polling-burn table's replacement guidance must not invent math. `list`-polling retirement (or fallback retention) is a Task-1 decision with a caller-facing recommendation; the O6 chat-ID resolution and `liveBroadcasts.list` (1 unit) are unchanged.
7. **Message policy unchanged.** `textMessageEvent`-only mapping through the EXISTING mapping layer (author fallback, provider ID, RFC3339→epoch, 512 dedup); non-text/malformed events skipped per the documented tested policy; metadata never visible.
8. **Regression locks (byte-stability ledger).** demo schedule + local MP4 vector; RTMPS argv; redaction; clean Ctrl+C stop (exit 0); single-`error:`-line contract; exit-code table; ChatStore log-first; worker spawn-at-`Confirmed` + join-after-sink-EOF topology; `MessageSource` seam unchanged.
9. **Verification posture.** Container-only via Alpine VM MCP. Offline tests only: fakes + loopback (Path A may add a loopback gRPC server under a Tokio test runtime, in-container, test-only). The streaming-call quota-billing unknowns and latency expectations for the operator documented in `docs/production-run.md`; live E2E stays OPERATOR-RUN (the operator's 2026-10-29 polling-build confirmation is recorded, never agent evidence).
10. **Version** 0.12.0 → 0.13.0 (minor). New dependencies land via the tracked lockfile only after Task 1's MSRV verdict; the dependency policy (no `google-youtube3`; justification per brief §3.3) is recorded in the plan.

## Per-Task Pre-Analysis

### Task 1 — Official streamList Client-Path Research and Design (NOT front-end)

- Deliverable: research + design record inside `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md` — current official status of `streamList` (S3-equivalents), the gRPC guide/proto, the REST mirror entry, any new guidance; claim-by-claim adjudication of the user-pasted directives; client-path decision (A/B/C) with citations; MSRV/toolchain verdict; reconnect/backoff/resume policy; buffering/backpressure/cancellation/shutdown semantics; typed-error mapping; offline-test strategy (mockable seam shape); quota-billing semantics as documented; polling retire-vs-fallback ruling with recommendation.
- STOP conditions surface to the caller: toolchain incompatibility; neither path implementable.
- 4.2 (implementer): plan-tracking commit; minimal to no src edits (Phase-06 Task-1 precedent).

### Task 2 — Streaming Client and Async Bridge Implementation (NOT front-end)

- New `src/youtube/` modules (names per 4.1b plan): stream client seam + concrete gRPC (or documented-HTTP) transport, stream payload models (serde/proto-generated per path), stream-class typed errors where new classes are needed, resumable loop helpers. Bearer-only; secret-safety identical to the existing module.
- Bridge (in `src/app/run/` only if needed): runtime-confined worker thread; forward into the existing bounded channel; stop-flag propagation; bounded join. The `MessageSource` seam stays untouched.

### Task 3 — Production Run and Frame-Pipeline Integration Switch (NOT front-end)

- Swap the worker's retrieval mechanism per the Task-1/2 designs; spawn/join topology, one-message-per-frame pacing, log-first store contract, and funnel precedence byte-compatible; `ChatRetrievalError` classes preserved (new variant only if Task 1 defines one); latency profile verified offline (arrival → channel, no added interval); polling retirement executed exactly as ruled (no dead code).

### Task 4 — Offline Tests, Documentation, Builds, and Manual Verification (NOT front-end)

- Coverage audit first; new offline tests only for genuine gaps (establishment/baseline, reconnect+resume+dedup, push latency, bridge confinement, backpressure, regression pins). Gate + both builds + `ls -l` cross-checks. Docs: `docs/production-run.md` (stream-backed behavior, latency, reconnect diagnostics, recovery, privacy, documented quota semantics), scope/config drift-verdicts from research, README, structure map, tech.md (new deps + bridge), context.md (+ the operator's 2026-10-29 confirmation note); tracked `.agent/reports/` Phase 07 report; operator manual E2E with push-latency + reconnect-drop steps (operator-run, never agent evidence).

## Risks & Error Handling

- R1: toolchain incompatibility (`tonic`/`prost`/`tokio` MSRV > 1.82) — STOP → caller decides the pin question (options recorded in the research plan).
- R2: REST-mirror wire semantics remain undocumented → Path B rejected; Path A governs; if that also fails, STOP.
- R3: unknown long-lived-stream billing semantics → documented-unknowns posture; operator guidance conservative.
- R4: latency improvement is the user's motivating goal — the integration task pins the offline latency property (no added interval between arrival and channel delivery) and the docs set honest expectations (YouTube-side push latency is not agent-controlled).
- R5: regression in demo/RTMPS/redaction/stop contracts — pinned tests + byte-stability ledger per task.
- On any sub-step failure: pause, log, ask the user; max 3 review cycles then escalate.

# Global Plan — Phase 06: YouTube Live Chat Integration

> TODO (source of truth): `.agent/todos/20261025/20261025-todo-2.md` (Pattern C: 4 tasks under `## Tasks`).
> Baseline: `main` @ `b75d914`, v0.11.1, working tree clean, pushed to `origin/main`. Test baseline: 391 passed / 0 failed.
> Convention: every step below is a SEPARATE `task` tool invocation; no sub-agent executes two steps.

## Front-End Determination

| Task | Front-end? | Reason |
| --- | --- | --- |
| 1. Official Live Chat API Research and Design | NO | Rust backend research/design task |
| 2. YouTube Live Chat API Client and Message Mapping | NO | Rust backend module (`src/youtube/`) |
| 3. Production Run and Frame-Pipeline Integration | NO | Rust backend wiring (`src/app/run/`, `src/app/pipeline/`) |
| 4. Offline Tests, Documentation, Builds, and Manual Verification | NO | Tests/docs/builds/report |

⇒ No task executes 4.1a / 4.5a; only 4.1b and 4.5b apply.

## Steps 2–6 (global execution order)

- Step 2: Git Feature Branch Setup => implementer (`feat/phase06-youtube-live-chat`)
- Step 3: Version Update (0.11.1 → 0.12.0, minor — feature phase) => implementer
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
- Step 6: Finish => planner (summary + next-TODO handoff text)

Branch creation/switch is RESTRICTED to step 2; version update RESTRICTED to step 3; `git push` RESTRICTED to step 5.

## Global Pre-Analysis (technical & architecture decisions)

1. **Retrieval method.** The app is fully synchronous (std + ureq 2.12.1, no Tokio; brief §3.3 rejects `google-youtube3`; tech.md reserves Tokio only if justified). The research task (Task 1) must verify against CURRENT official docs whether `liveChatMessages.streamList` (gRPC server-streaming) is a currently supported, implementable option for a sync Rust service before choosing. Working hypothesis to VERIFY (not assume): `liveChatMessages.list` polling honoring the server's `pollingIntervalMillis` + `nextPageToken` is the selected production strategy; `streamList` requires gRPC/tonic + protobuf and is likely rejected as an unjustified dependency/complexity explosion unless official evidence proves it mandatory. One single production strategy; both implemented is prohibited unless evidence demands.
2. **Concurrency bridge.** The frame loop calls `next_message()` synchronously at the top of each frame iteration. Planned shape (Task 1 design decision, encoded in the per-task plan): a dedicated bounded worker thread polls the chat API on the server-instructed interval and feeds a bounded std channel (`mpsc` or bounded `crossbeam`-free construction — std-only preferred); the production source drains at most one message per frame call, never blocking the renderer; the worker is stopped by the existing Ctrl+C `StopFlag`/cancellation seam with bounded joins. Tokio stays OUT unless research proves `streamList` is the only viable method (then the architector must justify and bound it explicitly).
3. **Seam contract.** `src/app/pipeline/source.rs` is the documented Phase-06 widening point: `fn next_message(&mut self) -> Result<Option<ChatMessage>, ChatError>`. Chat/API/network failures are NOT `ChatError`-representable. Task 3 performs the smallest typed backward-compatible widening (e.g., a source-level error enum or an associated-error change) and updates BOTH the demo source and all test implementations; provider/network failures must not be collapsed into misleading chat-log errors.
4. **Chat ID resolution.** The live chat id comes from the API response of the broadcast created by Phase 05 (`snippet.liveChatId` on the broadcast resource — Task 1 research pins the exact field + response shape). Absent/disabled chat ⇒ explicit typed error with actionable diagnostic; never hardcoded, never assumed present. Resolution reuses the already-authorized transport and the Phase-05 `PreparedBroadcast` flow — NO second broadcast creation, NO lifecycle change.
5. **Scopes & credentials.** Reuse Phase-04 credential service + `REQUIRED_SCOPE = youtube.force-ssl` verbatim. Task 1 research confirms the scope sufficiency for the selected chat method; if research proves an additional scope is required, that is a minimal documented integration fix (config/docs/auth note — and operator must re-run `auth`; record it in Task 4's manual procedure).
6. **Message mapping.** Provider message → `ChatMessage` via the existing model: author = `authorDetails.displayName` (normalized single-line), text = `snippet.textMessage.messageText` (normalized), provider message ID preserved, `publishedAt` RFC3339 → epoch seconds (the reserved `ChatMessage::with_timestamp` helper's documented consumer — retire its `#[allow(dead_code)]`). Non-text/system/malformed events skipped ONLY per a documented + tested policy (never silently as ordinary chat). Metadata never becomes visible text (`Display` = `author: text` stays).
7. **Store contract.** `ChatStore::accept` (log-first, reject-on-log-failure) and the bounded visible FIFO stay untouched; no history restore into the visible queue at startup; dedup/ordering policy defined in Task 1 (provider ID last-seen tracking is the working design; replayed/old IDs dropped).
8. **Retry/backpressure/cancellation.** All network waits/retries bounded (documented backoff limits, quota-aware: honor `pollingIntervalMillis`, no invented fixed intervals); worker errors funnel into the run error path without masking the primary FFmpeg/gate failure; shutdown ordering keeps Ctrl+C clean-stop (exit 0) intact.
9. **Regression locks (byte-stability ledger).** `demo` schedule + local MP4 vector; production RTMPS argument vector (33/42-token CBR shapes); secret redaction (tokens/headers/raw bodies/stream keys never logged); clean Ctrl+C stop; single `error:` line contract; exit-code table; `ChatStore` log-first acceptance. Phase 05 stream-health settings are NOT touched.
10. **Verification posture.** Container-only via Alpine VM MCP (`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` + build scripts). All tests offline (fake transports/sources, loopback servers, no Google contact, no real credentials/broadcasts). The real-broadcast manual E2E stays OPERATOR-RUN and documented — never claimed as agent evidence.
11. **Known stream-health limitation.** Chat content addresses the pure-black limitation when messages arrive; the TODO explicitly does NOT promise Studio Stream Health clears — docs must keep the existing known-limitation section truthful.
12. **Housekeeping.** TODO's Task-2 note: run the requested research gates before the chat-client code; Task 1's research record lives in its per-task plan file (Phase-05 Task-1 precedent). Version bump: minor (feature) → 0.12.0.

## Per-Task Pre-Analysis

### Task 1 — Official Live Chat API Research and Design (NOT front-end)

- Deliverable: research record in `.kilo/plans/20261025-phase06-task1-live-chat-research-design.md` — official URLs + access dates, `liveChatId` resolution fields (incl. how Phase-05-created broadcasts expose it and what indicates chat disabled/unavailable), `streamList` support/status + transport requirements vs `liveChatMessages.list` continuation semantics, scope/quota/rate-limit/retry facts for BOTH (with the quota math), message kinds + which carry user-visible text, chat lifecycle on broadcast end/disabled chat/revoked permissions.
- Decisions to encode: ONE production retrieval strategy + deterministic fallback/error policy; buffering/dedup/ordering/retry/backoff/cancellation/typed-error mapping; the `MessageSource` seam decision (bounded worker + channel vs minimal trait change) with its effects on `demo`.
- 4.2 (implementer): research is plan-tracked; docs/src edits only what this task's plan itself requires (Phase-05 Task-1 precedent: minimal).
- Risk: API currency — all claims must carry official citations with access dates; community sources labeled as such.

### Task 2 — YouTube Live Chat API Client and Message Mapping (NOT front-end)

- New `src/youtube/` modules (names per 4.1b plan): chat models (provider-side structs, serde), chat typed error, mockable chat transport/client seam over the existing ureq transport conventions (Bearer-only, redirects disabled, no token/header/body/raw-response logging), request builders + response decoding with correct params (part, liveChatId, maxResults, pageToken per researched spec).
- Mapping layer: supported text events → `ChatMessage`; skip-policy documented/tested; empty responses/duplicate IDs/ordering/continuation/ended-chat/permission/quota/transient/malformed/cancellation all have defined behavior (Task 1 design).
- Module boundary: YouTube types never exported to `chat`/`renderer`/`streaming`/`app` beyond `ChatMessage` + existing typed error contracts.

### Task 3 — Production Run and Frame-Pipeline Integration (NOT front-end)

- Replace `ChatFreeSource` with the real chat-backed source wired to the resolved chat ID + authorized transport; `demo` untouched.
- Seam widening per the Task-1/Task-3 plan decision; bounded cancellable worker; startup/shutdown order defined (worker start after broadcast prep, stop before/at frame-loop stop; blocked network ops and channel sends cannot block shutdown); one-message-per-frame pacing + log-first contract preserved; frame loop continuous during empty/slow/reconnecting chat; chat failures reach the `RunError` funnel without masking the primary failure.

### Task 4 — Offline Tests, Documentation, Builds, and Manual Verification (NOT front-end)

- Offline deterministic tests covering the TODO §4 bullets (chat ID extraction, wire params/decoding via loopback, mapping/normalization/policy, ordering/dedup/continuation/retry, cancellation/bounded queues/shutdown, source→store→renderer integration + log-write failure + idle frames, failure→exit path, regression pins for demo/MP4/redaction/stop).
- Standard gate + Linux/Windows rebuilds on the final tree with `ls -l` cross-checks.
- Docs: `docs/production-run.md` (Live Chat prerequisites/behavior/diagnostics/recovery/privacy + the manual E2E procedure), scope-driven updates to `docs/authentication.md`/`docs/google-cloud-prerequisites.md` only if researched scopes require, `docs/configuration.md` only if a real new config setting exists, README links; `.agent/project-structure.md`, `tech.md`, `context.md`; tracked Phase 06 completion report under `.agent/reports/`.
- The operator manual E2E is documented, NOT executed by the agent.

## Risks & Error Handling

- R1: streamList research verdict may force a dependency decision — pause and surface to the user before adopting any async/gRPC stack.
- R2: quota exhaustion from mis-paced polling — server-interval compliance + quota math recorded; operator guidance in docs.
- R3: operator-side scope gap (existing grant missing a new scope) — documented re-auth step; never improvised in code.
- R4: regression in demo/RTMPS/redaction/stop contracts — pinned tests + byte-stability ledger per task.
- On any sub-step failure: pause, log, ask the user; max 3 review cycles then escalate.

# Global Plan — Stream Establishment Panic Fix (Readiness Reserve + Worker Panic Funnel)

> TODO (source of truth): `.agent/todos/20261030/20261030-todo-1.md` (Pattern C: 2 tasks under `## Tasks`).
> Baseline: `main` @ `acdf8b5` (== `origin/main`, tree clean), v0.13.0, test baseline **494 passed / 0 failed** (closing log `logs/checks/20261029T222610Z.log`).
> Convention: every step below is a SEPARATE `task` tool invocation; no sub-agent executes two steps.

## Front-End Determination

| Task | Front-end? | Reason |
|---|---|---|
| 1. Readiness-Reserve Fix, Worker Panic Funnel, and Offline Contract Pins | NO | Rust backend (`src/youtube/` + `src/app/run/`) |
| 2. Documentation, Builds, and Tracked Fix Record | NO | Docs/builds/report |

⇒ No task executes 4.1a / 4.5a; only 4.1b and 4.5b apply.

## Steps 2–6 (global execution order)

- Step 2: Git Feature Branch Setup => implementer (`fix/stream-establishment-readiness`); the implementer commits the TODO + this global plan as the step-2 clean-tree step (context origin: the files are untracked planner artifacts).
- Step 3: Version Update (0.13.0 → 0.13.1, patch — fix semantics) => implementer
- Task 1: 4.1b Analysis & Planning => architector
- Task 1: 4.2 Implementation => implementer
- Task 1: 4.3 Review & Simplification => code-reviewer + code-simplifier; fixes => implementer
- Task 1: 4.4 Documentation => docs-specialist (plan-record scope only)
- Task 1: 4.5b Overall Plan Adherence => architector
- Task 1: 4.6 Task Completion => implementer
- Task 2: 4.1b / 4.2 / 4.3 (+fix) / 4.4 / 4.5b / 4.6 (same cycle shape)
- Step 5: TODO File Completion => implementer (rename `-DONE`, tmp cleanup, merge to `main`, push `origin` ONLY)
- Step 6: Finish => planner (summary + handoff)

Branch creation/switch RESTRICTED to step 2; version bump RESTRICTED to step 3; `git push` RESTRICTED to step 5.

## Global Pre-Analysis (technical & architecture decisions)

1. **Root cause is already pinned (Planner, 2026-10-30, source-verified):** tonic 0.13.1 `Grpc::streaming` (under `server_streaming`) calls the tower-Buffer-backed `Channel` WITHOUT `ready()`; the tower `Service` contract requires `poll_ready` on the same handle before `call`; tonic issue #2555 + community client code confirm the exact panic. Our Phase-07 `open()` skeleton omitted the reserve step (plan defect in the Task-2 plan §C.4, resolved here). No version reroute exists (tonic 0.13.1 requires `tower ^0.5`; no 0.5.4+ release as of 2026-10-30) — the fix is our call shape.
2. **Fix 1 shape (Task 1 decides exact code):** inside `open()`, after the channel connect and before `server_streaming`: `grpc.ready().await` with the failure mapped to a NEW fixed reason constant (`STREAM_READY_FAILURE_REASON`, `Network` class → the CLOSED transient list → bounded re-establishment; never embed `T::Error` text; secret-safety identical).
3. **Fix 2 shape:** `run_chat_worker` wraps the `block_on` body in `std::panic::catch_unwind(AssertUnwindSafe(...))`; on catch → `record_terminal` with a FIXED detail constant (no panic payload text; the default panic hook keeps printing the backtrace to the console; the detail points the operator to stderr). The existing funnel then surfaces one `error:` line + exit 1 (terminal-class policy unchanged; ChatEnded/`offlineAt` graceful paths untouched). The panic payload is never logged (fixed-string detail only).
4. **Offline pins (decided here):** (a) incident anchor — the pre-fix call shape (no `ready()`) against a plaintext `http://127.0.0.1:<dead-port>` `Endpoint` panics locally (asserted via `catch_unwind(AssertUnwindSafe)` in the test); (b) fix shape — the EXACT production call construction (including the new `ready()`), endpoint parameterized for tests, against a dead loopback endpoint: NO panic, typed `Network`-class error; (c) funnel — a `#[cfg(test)]` fake stream transport that panics inside `open`; the worker catches and writes the typed terminal; `ChatSource` surfaces `ChatRetrievalError`. No listener, no Google, no external address; the connecting-pin laziness of tonic makes the dead-port endpoint safe (no network I/O completes before the contract point; the call never leaves the process shape asserted).
5. **Clippy posture:** `catch_unwind` may trigger clippy lints (e.g. `panic_in_result_fn`/unwind-family lints are possible) — preferred handling is a rule-following re-shape; a compiler-forced single adaptation follows the established `Box<ureq::Error>` precedent, documented in the plan with the lint name. Zero unconditional `#[allow(` additions; no commented-out code; file caps ≤ 200 lines.
6. **Version** 0.13.0 → 0.13.1 (patch). No dependency changes beyond the standard lockfile refresh; the Phase-07 pin set stays (tokio 1.53.1 / tonic 0.13.1 / prost 0.13.5 + transitive pins as recorded).
7. **Verification posture:** container-only via the Alpine VM MCP; the standard gate + windows-gnu check-only in Task 1; BOTH release builds + `ls -l` cross-checks in Task 2 (recorded sizes). Operators may keep using `RUST_BACKTRACE` for diagnostics; the docs record the panic-print + terminal-line behavior.
8. **Incident attribution stays operator-supplied** (`logs/run/20261030-211225.log`); the agent never observed Google live; the tracked Phase-07 fix report records both attributions cleanly. Studio hygiene pointer for the operator (autoStop completed both broadcasts; glance per the fate policy) recorded in docs — never agent-verified.

## Per-Task Pre-Analysis

### Task 1 — Readiness-Reserve Fix, Worker Panic Funnel, and Offline Contract Pins (NOT front-end)

- Deliverable: `.kilo/plans/20261030-fix-task1-remedy-and-pins.md` — the exact code edits (`stream_concrete.rs` open-shape + the new reason constant's home in `stream_client.rs`; `chat_worker.rs` containment), the exact three test pins (file names/test names/assertions), the clippy posture, the gate/check verification commands.
- 4.2 (implementer): plan-tracking commit → code+tests → targeted runs → gate → windows-gnu check → §L addendum; NO builds.

### Task 2 — Documentation, Builds, and Tracked Fix Record (NOT front-end)

- Deliverable: `.kilo/plans/20261030-fix-task2-docs-builds-report.md` — coverage audit; the doc-change set (`production-run.md` diagnostics/recovery, limitation-wording revision scope, README, structure map, tech.md, context.md); the tracked completion report skeleton (incident facts + root-cause citations + fix evidence + builds + audit); the gate/build/audit command list; commit list.

## Risks & Error Handling

- R1: the fix-shape pin against a dead endpoint could connect for real network I/O before the contract point — mitigated: the endpoint is plaintext `127.0.0.1` on a dead port; the call shape (non-TLS) is pinned never-to-reach external hosts; the container has no route ambiguity (the assertion targets the typed error, and the test asserts NO panic deterministically).
- R2: `ready()`'s `T::Error` type conversion — the plan-fixes the mapping construction (via the established status-first classification entry points) instead of inventing a new funnel class.
- R3: catch_unwind semantic risk (residual lantern state after unwind inside a runtime) — the plan documents the accepted behavior: the runtime is per-run and thread-confined; after the terminal write the thread returns; no reuse.
- R4: regression risk in the funnel — the ChatEnded/offlineAt graceful shape must be re-pinned green by the existing rosters (worker/flow rosters).
- R5: docs drift risk (stale "handshake untested offline" wording) — Task 2 owns the precise revision scope.
- On any sub-step failure: pause, log, ask the user; max 3 review cycles then escalate.

## User Rulings (2026-10-30)

- Global plan APPROVED via the workflow `question` step; per-task plans AUTO-APPROVED (option: "Approve Global and Tasks Plans").
- Containment semantics ruling (Task 1 USS-2): a retrieval-thread panic maps to the TYPED FATAL policy — the worker catches the unwind, writes the existing terminal (fixed detail), and the run surfaces the funnel's single redacted `error:` line and EXITS 1 promptly (pre-streaming/live-stop semantics per the existing `RunError` mapping). The "keep streaming + warn" alternative was rejected; the mutex-risk alternative is forbidden.
- Amendment appended below the original content; the steps/pre-analysis above are never overwritten.

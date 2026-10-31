# Stream Establishment Readiness Fix — Completion Report

> Branch `fix/stream-establishment-readiness` — version v0.13.1 (pre-merge at the time of writing).
> Authority: TODO `.agent/todos/20261030/20261030-todo-1.md`; global plan `.kilo/plans/20261030-establishment-readiness-fix.md`; per-task plans `.kilo/plans/20261030-fix-task1-remedy-and-pins.md` and `.kilo/plans/20261030-fix-task2-docs-builds-report.md`; Task-1 adherence `.kilo/plans/20261030-fix-task1-adherence.md`.
> Every number in this report is an OBSERVED value from a command run inside this pass (or a committed record cited by path); no prediction is restated as a result, and no live Google observation is claimed by the agent.

## Table of Contents

- [Overview](#overview)
- [Incident (operator-supplied facts)](#incident-operator-supplied-facts)
- [Root Cause (pinned from the locked source)](#root-cause-pinned-from-the-locked-source)
- [The Two Fixes](#the-two-fixes)
- [The Three Offline Pins (evidence)](#the-three-offline-pins-evidence)
- [Files Changed / Commits](#files-changed--commits)
- [Test Coverage Audit](#test-coverage-audit)
- [Exact Commands and Exit Statuses](#exact-commands-and-exit-statuses)
- [Build Results](#build-results)
- [Hygiene/Secret Audit](#hygienesecret-audit)
- [Manual Verification Pointer (operator-run)](#manual-verification-pointer-operator-run)
- [Checks Actually Performed vs NOT Performed](#checks-actually-performed-vs-not-performed)
- [Known Limitations](#known-limitations)
- [AC Closing Matrix](#ac-closing-matrix)
- [Evidence Index](#evidence-index)

## Overview

The v0.13.0 streamList build (`fix/stream-establishment-readiness`, v0.13.1) fixes two defects exposed by the 2026-10-30 operator runs: (1) the establishment call omitted the tower `Service` readiness reserve required by the tonic transport `Channel`, panicking the chat worker thread before any server contact; (2) the worker's panic was discarded at the gate join, so the run kept streaming with dead retrieval and exited 0 without any `error:` line. The fix chain splits into Task 1 (code + offline pins; commit chain below) and Task 2 (this pass: docs, builds, hygiene audit, tracked record). The report's own commit hash is unknowable from inside the record (predecessor pass convention); the containing commit's identity is read from `git log` by the caller.

## Incident (operator-supplied facts)

All facts in this section are operator-supplied from `logs/run/20261030-211225.log` (a plain gitignored path citation — never staged, never read into any doc beyond the path) and were never agent-observed live.

- Two v0.13.0 `run` sessions (broadcasts `TUcJ71xiBqM`, `WebHLRIOlvo`) both started, confirmed active, and resolved the live chat id, then the worker thread panicked at the `.open()` call.
- VERBATIM panic line: `thread '<unnamed>' panicked at .../tower-0.5.3/src/buffer/service.rs:122:23: `send_item` called without first calling `poll_reserve``.
- Run 2's `RUST_BACKTRACE=1` frame chain: `PollSender::send_item` → `tower::buffer::Buffer::call` → `<tonic Channel as tower_service::Service<...>>::call` → `<GoogleLiveChatStreamTransport as ChatStreamTransport>::open::{{closure}}` → `run_stream_loop::{{closure}}`.
- Mute outcome: the stream kept working (video healthy), zero chat messages rendered, the run exited 0, and NO `error:` line was ever printed.
- The "panicked" quotes above are the operator log's only terminal evidence.
- Secondary shaping facts from the same log: `enableAutoStop` auto-completed both broadcasts (no live orphan remains), and both video encodes were healthy (CBR ~5989/5965 kbit/s, clean exits).

## Root Cause (pinned from the locked source)

- tonic 0.13.1 `Grpc::streaming` (`src/client/grpc.rs:282–319`) calls `self.inner.call(request)` WITHOUT first calling `Grpc::ready()` (which exists at `grpc.rs:199–204`).
- The transport `Channel` is backed by `tower::buffer::Buffer`; its `call` MUST follow a `poll_ready` on the same handle — `tower` 0.5.3 `service.rs:122:23` is the exact panic site (`poll_ready` → `poll_reserve`).
- Independent corroboration: tonic issue #2555 (identical panic before server contact) and community client documentation ("reserve with `client.ready().await` before `call()`"); generated tonic clients do this too.
- **Plan-defect admission:** the Phase-07 Task-2 plan's `open()` skeleton omitted the reserve — our call shape, never a crate defect. tonic 0.13.1 requires `tower ^0.5` and no 0.5.4+ patch exists on crates.io as of 2026-10-30 to route around it.
- The eager-`connect` mechanics correction recorded during Task-1 research (Task-1 plan §A.0 item 4) stands as the research record it was.

## The Two Fixes

- **Fix 1 — the readiness reserve.** `open_with_endpoint` now awaits `grpc.ready()` before `server_streaming`; a reserve failure maps to the FIXED non-secret reason `STREAM_READY_FAILURE_REASON` = `the stream endpoint readiness reserve failed` (Network/transient class; the error value is discarded with `|_|`, so secret-safety is unchanged).
- **Fix 2 — the worker containment.** The worker's `block_on` body runs inside `std::panic::catch_unwind(AssertUnwindSafe(...))`; a caught panic writes the EXISTING typed terminal (`record_terminal` → `ChatRetrievalError::Transport`) with the FIXED `CHAT_WORKER_PANIC_DETAIL` detail — the panic payload text is never read. The two constants/factory live in `chat_stream_delivery.rs` beside `record_terminal` per the Caller's Option-A ruling; `gate.rs`'s `let _ = worker.join()` is untouched, and the ChatEnded/`offlineAt` graceful paths are untouched.
- Operator-visible composed line (verbatim, one redacted `error:` line, then exit 1): `error: live chat transport failed after repeated attempts: the chat streaming worker panicked; its backtrace is on the standard error stream and the run fails with exit code 1`.

## The Three Offline Pins (evidence)

- `stream_concrete_tests.rs::establishment_anchor_unread_shape_panics_locally` — the PRE-FIX reserve-less shape (via `connect_lazy`, zero I/O) panics locally against the dead plaintext loopback endpoint.
- `stream_concrete_tests.rs::establishment_fixed_shape_reserves_readiness_and_yields_typed_error` — the EXACT production composition (through `open_with_endpoint`) yields NO panic and a typed Network-class error.
- `chat_worker_panic_tests.rs::worker_panicking_transport_writes_the_terminal` — a faked panicking transport produces the typed terminal with the exact fixed detail, and `ChatSource` surfaces `Retrieval(Transport)` once then `Ok(None)`.
- Offline posture: the dead plaintext endpoint `http://127.0.0.1:1` only; the anchor uses `connect_lazy` (zero I/O); no listener runs and no test contacts Google.
- OBSERVED fix-shape reason: `the stream endpoint connection failed` — a member of the pinned 3-reason set (the eager connect is refused before the reserve arm can fire, so `STREAM_READY_FAILURE_REASON` is a lawful member but not the observed one).
- Task-1 targeted-run counts, as recorded in `.kilo/plans/20261030-fix-task1-remedy-and-pins.md` §L.4: `stream_concrete` 2 passed; `chat_worker` 24 passed; `chat_stream` 12 passed.

## Files Changed / Commits

Observed in `git log` (lineage on `fix/stream-establishment-readiness`):

| Step | Commit | Message |
| --- | --- | --- |
| version bump | `6151fa0` | chore: bump version to 0.13.1 |
| Task-1 plan | `cf258f2` | chore: track stream establishment readiness fix Task 1 plan |
| Task-1 code + pins | `5190145` | feat: reserve stream endpoint readiness and contain chat worker panics |
| Task-1 addendum | `2e89cc3` | docs: record the readiness-fix task-1 observed evidence |
| Task-1 fold | `2873692` | refactor: fold the reestablishment terminal literal and track the task1 simplification plan |
| Task-1 alignment | `fee9d44` | docs: align the task1 remedy plan record |
| Task-1 4.6 marks | `a4eb832` | docs: mark establishment fix task1 done and track its adherence report |
| Task-2 plan | `a7db8dc` | chore: track stream establishment readiness fix Task 2 plan |
| Task-2 user docs | `9604bbd` | docs: record the stream establishment readiness fix for operators |
| Task-2 metadata + report | this file's commit | docs: record the readiness-fix metadata and completion report |

Code files changed by Task 1: `src/youtube/stream_concrete.rs` (the reserve composition), `src/youtube/stream_client.rs` (the fixed reason constant), `src/app/run/chat_worker.rs` (the catch boundary), `src/app/run/chat_stream_delivery.rs` (the moved constants/factory), plus the two new rosters `src/youtube/stream_concrete_tests.rs` and `src/app/run/chat_worker_panic_tests.rs`.

Task-2 changed docs only: `docs/production-run.md`, `README.md`, `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md`, and this report. No code, test, lockfile, or version change.

## Test Coverage Audit

**Verdict: ZERO new automated tests.** The TODO Task-2 bullets define no test expectations: the only coverage-bearing words in Task 2 are the audit bullet itself. The three offline pins that the fix-phase needed were landed and gated by Task 1 (`stream_concrete_tests.rs` 2 + `chat_worker_panic_tests.rs` 1; grep-verified rosters exist and are registered via the `#[cfg(test)] #[path]` pattern). The run-side funnel (rendering of the single redacted `error:` line + exit 1) is composition-proven by the pre-existing pin `error_tests.rs::chat_retrieval_failure_maps_to_the_chat_funnel_without_masking_the_gate` (zero diff across the whole fix chain, per Task-1 adherence §7/AC-3) — COVERED-BY-COMPOSITION, no widening performed.

Audit note (recorded, not argued): the 4.2 verification for this task is the FULL gate on the final tree, not new tests; the honest coverage statement is "the 3-pin offline proof from Task 1 + the pre-existing funnel/display pins close every AC claim Task 2 must flip (AC-5/AC-6/AC-7)". Roster arithmetic: 494 baseline tests + Task-1's 3 pins = the fresh gate total **497**.

## Exact Commands and Exit Statuses

All container commands run through the Alpine VM MCP against `/rust-youtube-streamer/docker-compose.yml`; the host-side audit rows are plain git commands. Batches and filters follow the per-task plan §B.

| # | Command | Exit | Observed result |
| --- | --- | --- | --- |
| 1 | `git status --porcelain` (pre-flight) | 0 | clean except the untracked Task-2 plan file |
| 2 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` | 0 | four PASS lines (fmt-check/check/test/clippy); `ALL CHECKS PASSED`; **497 passed / 0 failed**; log `logs/checks/20261030T224512Z.log` |
| 3 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked --target x86_64-pc-windows-gnu` | 0 | `Finished dev profile` (no release relationship) |
| 4 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh` | 0 | `dist/windows/rust-youtube-streamer-service.exe` (12334184 bytes) |
| 5 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` | 0 | `dist/rust-youtube-streamer-service` (7654688 bytes) |
| 6 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows` | 0 | both files non-empty; sizes match the scripts exactly |
| 7 | `git ls-files dist videos` | 0 | EMPTY |
| 8 | `git ls-files credentials` | 0 | `credentials/.gitkeep` only |
| 9 | `git check-ignore -v dist/rust-youtube-streamer-service dist/windows/rust-youtube-streamer-service.exe logs/run/20261030-211225.log` | 0 | `.gitignore:34:dist/` (both artifacts); `.gitignore:18:logs/*` (the cited log) |
| 10 | `rg -n "ya29\|AIza" src docs README.md` | 1 (no matches) | ZERO hits |
| 11 | `rg -c "#\[allow\(" src --glob "*.rs"` | 0 | 9 attribute lines (8 `dead_code` + 1 `clippy::manual_clamp`) |

The order is recorded, not silently swapped: Windows artifact first, then Linux (the Phase-07 TODO wording precedent).

## Build Results

- Windows: `dist/windows/rust-youtube-streamer-service.exe` — OBSERVED **12,334,184 bytes** (script print and `ls -l` agree exactly; non-empty).
- Linux: `dist/rust-youtube-streamer-service` — OBSERVED **7,654,688 bytes** (script print and `ls -l` agree exactly; non-empty).
- Both rebuilt on the final tree, `ls -l`-cross-checked; the Windows build ran before the Linux build.
- Caveats: cross-compilation is not native runtime validation (the `.exe` must be tested separately on Windows); building validates no live chat behavior — the real handshake stays the operator E2E.

## Hygiene/Secret Audit

| Check | Result |
| --- | --- |
| `git status --porcelain` | only the intended paths of the Task-2 docs/report set (plus the plan file already committed) |
| `git ls-files dist videos` | EMPTY both |
| `git ls-files credentials` | `.gitkeep` only |
| `git check-ignore -v dist/rust-youtube-streamer-service` | `.gitignore:34:dist/` |
| `git check-ignore -v dist/windows/rust-youtube-streamer-service.exe` | `.gitignore:34:dist/` |
| `git check-ignore -v logs/run/20261030-211225.log` | `.gitignore:18:logs/*` (cited as a plain path only) |
| `ya29` / `AIza` in `src docs README.md` | ZERO |
| `key=` | the pre-existing "never an API key" doc-comment/test-assertion set, count unchanged |
| `rtmps://` | pre-existing `.example.test` fixtures and doc-context mentions only, count unchanged |
| `#[allow(` in `src/` | EXACTLY 9 attribute lines (8 `dead_code` + 1 `clippy::manual_clamp`), unchanged — zero added by the fix chain |
| Report self-check | no secret material (stack text only, no tokens/keys/URIs); no literal newline-escape sequences; every numeric claim observed this pass or path-cited |

## Manual Verification Pointer (operator-run)

The real-Google E2E is operator-run and was NOT executed by the agent: follow `docs/production-run.md` §Manual Verification (unlisted broadcast → video before messages → separate-account text renders PROMPTLY → idle periods → optional network-interruption re-establishment check → clean Ctrl+C exit 0 → broadcast auto-completes). On the fixed build the incident's dead-delivery shape is EXPECTED FIXED: messages from the separate account must render. No agent-observed live claim is made.

## Checks Actually Performed vs NOT Performed

Performed (this pass): the full container gate on the final tree (497/0), the windows-gnu `cargo check`, both release builds, the `ls -l` cross-check of both artifacts, and the host-side hygiene/secret audit rows above.

NOT performed: live Google verification (no call to Google), native Windows/Linux runtime execution of the artifacts, and any YouTube Studio observation. The one cited operator log (`logs/run/20261030-211225.log`) is operator evidence, not agent evidence.

## Known Limitations

- (a) The real-Google TLS/handshake and cloud-side establishment behavior remain operator-verified ONLY; the offline pins cover the call-shape contract locally (the readiness reserve and the containment), never the real handshake.
- (b) The manual E2E (push latency, re-establishment) remains pending on the streaming build.
- (c) Carried Phase-07 facts are unchanged: the baseline-session loss window, the quota unknowns, the `LIVE_CHAT_DISABLED`/`LIVE_CHAT_ENDED` code ambiguity, and the ≤64-message shutdown loss window.

## AC Closing Matrix

TODO `.agent/todos/20261030/20261030-todo-1.md` lines 56–62; AC-1..AC-4 flipped at Task-1 4.6, AC-5..AC-7 close with this Task-2 evidence set.

| AC | Line | State | Evidence |
| --- | --- | --- | --- |
| AC-1 | 56 | `[x]` (Task-1 4.6) | Fully proven offline: `STREAM_READY_FAILURE_REASON` verbatim (`stream_client.rs:32`), reserve arm exactly `.map_err(|_| stream_ready_failure_error())?`, `LiveApiError::Network` (transient-list member — `chat_failure.rs` untouched); no runtime text embedded anywhere. |
| AC-2 | 57 | `[x]` (Task-1 4.6) | Both pins exist and PASS: 2/2 in the Task-1 targeted run (§L.4) and green in both full gates (497/0 incl. rosters); the observed fixed-shape reason is recorded as a lawful set member. |
| AC-3 | 58 | `[x]` (Task-1 4.6) | Every link pinned offline: pin (c) proves worker-catch → `Transport` terminal with the EXACT `CHAT_WORKER_PANIC_DETAIL` → `ChatSource` surfaces `Retrieval(Transport)` once then `Ok(None)`; the run-side rendering + exit 1 is pre-existing machinery pinned by `error_tests.rs::chat_retrieval_failure_maps_to_the_chat_funnel_without_masking_the_gate` (zero diff across the chain). |
| AC-4 | 59 | `[x]` (Task-1 4.6) | `demo` + pipeline paths: zero diff across the whole chain; mapper/dedup/convert/store/pacing pins and worker flow/budget/gate rosters all green in the 497/0 gates. |
| AC-5 | 60 | `[ ]` (pending 4.6; evidence recorded this pass) | Regression locks + final commit-set hygiene: the §Hygiene/Secret Audit table above (9-allow attribute-exact, zero `ya29`/`AIza`, `dist/`/`videos/` empty, no secret material) plus the unchanged RTMPS/redaction/exit/ChatEnded surfaces in the green gate. |
| AC-6 | 61 | `[ ]` (pending 4.6; evidence recorded this pass) | Full gate on the final tree: 497 passed / 0 failed, `ALL CHECKS PASSED`, log `logs/checks/20261030T224512Z.log`; both artifacts rebuilt and `ls -l`-cross-checked — Windows 12,334,184 B, Linux 7,654,688 B. |
| AC-7 | 62 | `[ ]` (pending 4.6; evidence recorded this pass) | Docs/records/report: `docs/production-run.md` (contained-panic diagnostics/recovery + revised limitation wording), README (v0.13.1 + observed count), `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md` (operator attribution kept explicit throughout), and this tracked completion report. |

## Evidence Index

- TODO: `.agent/todos/20261030/20261030-todo-1.md` (Task 2 bullets; Pre-Diagnosis binds the incident facts).
- Global plan: `.kilo/plans/20261030-establishment-readiness-fix.md` (User Rulings; containment = TYPED FATAL exit 1).
- Task-1 plan: `.kilo/plans/20261030-fix-task1-remedy-and-pins.md` (§L holds the OBSERVED values, targeted-run counts, terminal detail string, line counts).
- Task-1 adherence: `.kilo/plans/20261030-fix-task1-adherence.md` (ADHERENT; 9-allow canonical figure; the AC flip list 4 HOLD → Task 2).
- Task-2 plan: `.kilo/plans/20261030-fix-task2-docs-builds-report.md` (this pass's binding plan; §L addendum records the observed evidence).
- Gate log (VM-side, gitignored): `logs/checks/20261030T224512Z.log`.
- Task-1 gates: `logs/checks/20261030T220937Z.log` (pre-fold) and `logs/checks/20261030T222042Z.log` (post-fold), both 497/0.
- Build commands: `sh scripts/build-windows.sh`, then `sh scripts/build-linux.sh`; `ls -l` cross-check on `/rust-youtube-streamer/dist` and `/rust-youtube-streamer/dist/windows`.
- Log citations (plain paths, gitignored): `logs/run/20261030-211225.log` (operator incident log).
- Prior report precedents: `.agent/reports/20261026-phase-06-youtube-live-chat-integration.md`, `.agent/reports/20261029-phase-07-streamlist-migration.md`.

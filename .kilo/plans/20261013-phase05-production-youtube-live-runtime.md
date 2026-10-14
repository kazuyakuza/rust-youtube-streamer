# Phase 05 — Production YouTube Live Runtime — Global Plan

- Date: 2026-10-13
- TODO file: `.agent/todos/20261013/20261013-todo-1.md` (Pattern C: 4 tasks under `## Tasks`)
- Status: APPROVED (2026-10-13) — "Approve Global and Tasks Plans" (per-task plans auto-approved)
- Base commit: `main` @ `7769efd` (clean tree, synced with `origin/main`)
- Base version: 0.7.1 (`Cargo.toml`)
- Planned feature branch: `feat/phase05-production-live-runtime`
- Planned version bump: 0.7.1 → 0.8.0 (minor; new production feature)

---

## 1. Global Pre-Analysis

### 1.1 Current repository state (verified 2026-10-13)

- Phase 04 (OAuth 2.0 / credential management) is complete and merged; `auth` implements the
  installed-app flow with PKCE S256, loopback callback, minimum-scope verification, and atomic
  credential persistence. User-reported manual Windows OAuth test succeeded (context.md).
- 254 unit tests pass; the standard gate (`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`) is green; Linux and Windows release artifacts build cleanly via the pinned container toolchain.
- The reusable credential service is **staged but unwired**: `src/youtube/credential_service.rs`
  and `src/youtube/google_authorization_verifier.rs` carry module-level `#[allow(dead_code)]`
  declarations in `src/youtube/mod.rs` with an explicit contract: both allows retire when the
  production `run` phase wires them (this phase).
- `src/app/modes.rs` still holds the `run` exit-3 placeholder; `src/streaming/video_output.rs`
  holds the reserved `OutputKind::Session` variant (`session-` prefix) as dead code.
- Verification convention: all automated checks run inside the Compose `rust` service on the
  Alpine VM via the `alpine-vm` MCP (commands pattern
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cmd>`); native
  Windows/Linux runtime claims remain prohibited without recorded user-run evidence.

### 1.2 Scope summary (task extraction)

Per Pattern C, the four `###` headings under `## Tasks` are the tasks, in file order:

1. **Official API Research and Runtime Design** — research current official Google/YouTube
   documentation (scopes, broadcast/stream lifecycle, RTMPS ingestion fields, failure cases),
   defer Live Chat to Phase 06, select minimal dependencies, define typed API models/transport
   seams/errors/lifecycle transitions, and document the orchestration/shutdown sequence.
2. **Broadcast and Ingestion Lifecycle** — implement create/bind/readiness YouTube API
   operations behind project interfaces, bounded cancellable readiness waits, RTMPS destination
   assembly, typed actionable failures, fake-transport tests incl. secret redaction.
3. **Production `run` Orchestration and FFmpeg Output** — replace the exit-3 placeholder with
   the real startup path consuming the Phase 04 credential service; explicit startup ordering;
   minimal streaming-output extension so FFmpeg can target RTMPS in production while `demo`
   keeps local MP4; secret-safe spawn/error paths; Ctrl+C/failure shutdown ordering; CLI/usage
   text and tests updated.
4. **Tests, Documentation, Builds, and Manual Verification Procedure** — offline test suite
   (fake transports/chat sources/FFmpeg), docker dev checks, Linux + Windows rebuilds, README +
   docs + project-structure/tech/context updates, tracked Phase 05 completion report under
   `.agent/reports/`, safe manual verification procedure for a real unlisted broadcast, and a
   final secret/hygiene audit.

### 1.3 TODO-internal inconsistency note (RESOLVED by user, 2026-10-13)

The `## Acceptance Criteria` list still contains chat-related lines (chat fetch/render criteria
and "chat ingestion" test coverage) even though the Objective/Scope (and commit `7769efd`)
explicitly defer Live Chat to Phase 06. **User ruling at approval: those Live Chat acceptance
criteria are an error; Live Chat is out of scope of Phase 05.** Handling: Phase 05 work,
checklists, and reports treat the chat-related criteria lines as out-of-scope/Phase-06 defects
in the TODO (annotated as such, never claimed done, never implemented); the chat-source seam
stays mockable so Phase 06 can fill them later.

### 1.4 Front-end determination

No task is front-end related (pure Rust backend/service work). Sub-steps 4.1a and 4.5a are
**omitted for every task**; each task runs 4.1b → 4.6 only.

---

## 2. Global Technical & Architecture Decisions (pre-analysis; per-task 4.1 plans refine)

- **D-G1 — Direct API calls, no generated client.** `google-youtube3` stays rejected (brief
  §3.3). YouTube Data API v3 operations are called directly over HTTPS JSON using the already
  pinned `ureq 2` + `serde_json` stack, behind the project's own `src/youtube/` interfaces.
  No broad generated API client is added.
- **D-G2 — Synchronous, no Tokio.** The project is sync-std with a reserved Tokio decision;
  Phase 05 keeps synchronous HTTP/process orchestration (ureq, std threads for drain), consistent
  with Phase 03/04. Blocking readiness waits are bounded and cancellable via the existing
  stop-flag seam plus a deadline loop (exact mechanism decided in Task 1/2 plans).
- **D-G3 — Live Chat fully deferred.** No chat scopes, retrieval APIs, gRPC/prost deps, chat
  ingestion, or chat rendering land in Phase 05. The frame loop runs chat-free; the chat-source
  seam (`app/pipeline/source.rs`) is preserved so Phase 06 can attach without redesign.
- **D-G4 — Credential service reuse, no duplication.** `run` consumes
  `youtube::credential_service` (load → verify-first probe → refresh-on-rejection → persist →
  re-verify) and `google_authorization_verifier` exactly as staged; both module-level
  `#[allow(dead_code)]` declarations retire at this wiring. Secret-wrapper token types and
  redacted error contracts from Phase 04 are reused untouched.
- **D-G5 — New secrets get the Phase 04 secret-wrapper treatment.** The stream key and any
  full RTMPS destination are held in redacting wrapper types (Debug never reveals, no Display);
  errors, logs, tracing fields, spawn argv records, and panic messages never embed them.
- **D-G6 — Broadcast lifecycle.** One new broadcast + one new live-stream resource per `run`
  invocation from validated config (`youtube.broadcast.*`); bind them; poll/readiness-check
  with bounded, cancellable waits until the API reports a streamable state; extract ingestion
  address + stream name/key from API responses; assemble the RTMPS destination for FFmpeg.
  Exact endpoints, fields, and legal transitions are verified against current official docs in
  Task 1 and recorded in the per-task plan and completion report — no hardcoded undocumented
  behavior.
- **D-G7 — FFmpeg output abstraction extension (minimal).** The existing locked argument
  vector and process supervision are preserved; the output target becomes parameterizable:
  local recording (`demo`, unchanged collision-safe MP4) vs RTMPS destination (production).
  The reserved `OutputKind::Session` variant is either repurposed or replaced by an explicit
  RTMPS-carrying variant — Task 1's plan picks one and documents why (the dead-code allow then
  retires accordingly). No-shell spawn, drain threads, frame-write errors, child cleanup, and
  `demo` behavior must not regress.
- **D-G8 — Shutdown policy.** Defined deterministic order: stop chat polling (n/a in 05,
  reserved), request frame-loop stop, close FFmpeg stdin, bounded wait/kill, report cleanup
  failures without masking the primary error, cancel in-progress API waits. Broadcast/stream
  resource fate on shutdown and startup failure is documented (default: do not auto-delete;
  transition out of `live` when a clean documented transition exists and is required) — exact
  policy fixed in Task 1/3 plans from official docs.
- **D-G9 — Exit-code contracts.** `auth`/`demo` behavior and exit codes unchanged; `run`
  keeps non-interactive behavior with typed startup-failure mapping (usage 2 / config+startup
  failures 1 / placeholder exit 3 removed when the path is fully wired); success path exit 0.
- **D-G10 — Offline testing.** All automated tests use injected fake API transports, fake chat
  sources, and fake FFmpeg executables; no test contacts Google or creates a real broadcast.
  Manual end-to-end verification is a documented, operator-run procedure recorded in the report.
- **D-G11 — Version/branch handling.** Branch/merge restricted to workflow step 2; version
  bump restricted to step 3; push restricted to step 5.

---

## 3. Workflow Mapping (Critical Workflow steps 2–6)

| Step | Sub-agent | Notes |
|---|---|---|
| 2. Git Feature Branch Setup | implementer | commit any unstaged work (none expected: tree clean); ensure on `main`; create `feat/phase05-production-live-runtime`; switch |
| 3. Version Update | implementer | `Cargo.toml` 0.7.1 → 0.8.0; refresh `Cargo.lock` via container `cargo check`; commit `chore: bump version to 0.8.0` |
| Task 1: 4.1b Analysis & Planning | architector | official-API research + runtime design plan; save `.kilo/plans/20261013-phase05-task1-*.md` |
| Task 1: 4.2 Implementation | implementer | design deliverables are docs/types as the plan directs |
| Task 1: 4.3 Review & Simplification | code-reviewer + code-simplifier → implementer (fixes) | |
| Task 1: 4.4 Documentation | docs-specialist | |
| Task 1: 4.5b Plan Adherence | architector | |
| Task 1: 4.6 Completion | implementer | `[DONE]` mark + commit |
| Task 2: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 | architector / implementer / reviewers / docs-specialist | same cycle |
| Task 3: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 | same cycle | retires `modes.rs` placeholder |
| Task 4: 4.1b → 4.2 → 4.3 → 4.4 → 4.5b → 4.6 | same cycle | gates, builds, report, manual-procedure docs |
| 5. TODO File Completion | implementer | rename TODO with `-DONE`; tmp cleanup; merge to `main`; push `origin` ONLY |
| 6. Finish | planner | summary + next-TODO handoff text |

Rules in force for every sub-task invocation: the standard sub-agent preamble (single discrete
step, TODO-file primacy, tool-selection priority, gitignore compliance, completion signal),
context passing (TODO path, task text, plan path, constraints), and no assumption-making.

---

## 4. Per-Task Pre-Analysis

### Task 1 — Official API Research and Runtime Design

- **Goal:** before any API code, produce an evidence-backed design: required scopes
  (`https://www.googleapis.com/auth/youtube.force-ssl` already pinned in Phase 04 — verify
  sufficiency), broadcast/stream create/bind/readiness operations and fields, legal lifecycle
  transitions, RTMPS ingestion response fields and exact FFmpeg destination format, and failure
  mappings (disabled live streaming, channel eligibility, revoked auth, quota, transition
  conflicts).
- **Inputs:** brief, architecture/tech docs, Phase 03/04 reports, staged
  `src/youtube/` interfaces, official Google documentation (current, cited).
- **Outputs (expected):** per-task implementation plan (4.1b) capturing selected API operations,
  typed model/transport/error design, lifecycle state machine, orchestration + shutdown
  sequence, and dependency ruling (expected: no new dependencies beyond `ureq`/`serde_json`;
  any addition requires documented justification). Implementation of this task is expected to
  be mostly design artifacts/types that later tasks consume — 4.1 must define exactly which
  files (if any) change in this task.
- **Non-goals:** chat anything (Phase 06).

### Task 2 — Broadcast and Ingestion Lifecycle

- **Goal:** concrete YouTube API layer in `src/youtube/` (broadcast/stream operations per brief
  structure; refine layout only with justification): create stream, create broadcast, bind,
  status/readiness inspection with bounded+cancellable waits and typed readiness timeout,
  RTMPS destination extraction (secret-wrapped), typed actionable failures for every mapped
  case, redaction at response boundaries.
- **Design constraints:** mockable transport seam (fake transport in tests); documented fields
  only; no continuous spinning; no unbounded waits; malformed-response handling; quota/rate
  classification; secret redaction tests.
- **Expected shape:** new `src/youtube/` source + test files (names fixed in 4.1), reusing
  `transport.rs` patterns; strict file-size/method-size/depth rules.

### Task 3 — Production `run` Orchestration and FFmpeg Output

- **Goal:** wire the real `run`: load+refresh credentials via the staged service, verify
  permissions, create/bind/readiness-check resources, assemble RTMPS destination, extend the
  streaming output minimally for the RTMPS target, start FFmpeg, run the continuous frame
  pipeline, handle Ctrl+C/fatal failures with the documented shutdown order, and remove the
  exit-3 placeholder only when the path is complete.
- **Design constraints:** startup order as TODO §3 (refinements only if official API constraints
  demand, documented); no chat source dependency; secret-safe spawn/error paths (test
  redaction around spawn/exit failures); broadcast/stream fate on shutdown/startup-failure
  documented; `auth`/`demo` untouched; CLI usage text and `run` tests updated; exit contracts
  preserved.
- **Risks:** `modes.rs` retirement ripple (CLI tests), `OutputKind` change ripple in
  `video_output.rs`/`args.rs`/`sink.rs` — all enumerated in the 4.1 plan.

### Task 4 — Tests, Documentation, Builds, and Manual Verification Procedure

- **Goal:** full offline coverage audit (mapping every TODO bullet to tests), docker dev-checks
  gate, Linux + Windows release rebuilds with observed sizes, README/docs/configuration/
  authentication updates, `.agent/project-structure.md` + `tech.md` + `context.md` refresh,
  tracked completion report under `.agent/reports/` (API operations/scopes, dependencies,
  lifecycle/shutdown decisions, redaction review, changed files, exact commands/exit statuses,
  test counts, artifact sizes, manual verification procedure, limitations), a safe manual test
  plan (real unlisted broadcast, video arrival, Ctrl+C, post-shutdown broadcast status, privacy
  reminders, user-run vs container-run distinction), and the final secret/hygiene audit
  (nothing secret or artifact committed).
- **Note:** "Run `docker compose … dev-checks.sh`" and both build commands in the TODO are
  executed inside the Alpine VM via the `alpine-vm` MCP using the documented Compose paths —
  recorded verbatim with exit statuses in the report.

---

## 5. Approval & Execution Notes

- The global plan is never overwritten; per-task plans are produced in each 4.1 and saved under
  `.kilo/plans/20261013-phase05-task<N>-*.md`.
- Per the Critical Workflow, on approval each TODO task runs its own 4.1–4.6 cycle with separate
  `task` invocations; failures pause and escalate to the user.
- The acceptance-criteria chat deferral ruling in §1.3 stands unless overridden at approval.

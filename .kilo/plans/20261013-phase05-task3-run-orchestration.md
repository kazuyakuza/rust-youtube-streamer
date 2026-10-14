# Phase 05 — Task 3 Implementation Plan — Production `run` Orchestration and FFmpeg Output

- Date: 2026-10-14
- TODO authority: `.agent/todos/20261013/20261013-todo-1.md`, Task 3 (`### 3. Production run
  Orchestration and FFmpeg Output`) + the global Scope/Boundaries + Explicitly-Out-of-Scope
  sections. User ruling (global plan §1.3): the TODO's Live-Chat acceptance-criteria lines are a
  TODO defect — Live Chat is Phase 06; Task 3's run path is chat-free by design.
- Global plan (binding): `.kilo/plans/20261013-phase05-production-youtube-live-runtime.md`
  (D-G1..D-G11)
- Foundation plans (binding, consume verbatim): Task 1
  `.kilo/plans/20261013-phase05-task1-official-api-research-design.md` (esp. §1 research record,
  D-T1-8 orchestration, D-T1-9 shutdown/resource fate, D-T1-10/D-T1-11 streaming extension +
  secret wrapper, D-T1-12/D-T1-13 waits/predicates) and Task 2
  `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md` (+ Amendments 1–2,
  D-T2-1..D-T2-16). This plan may not contradict them; refinements below are the bindings
  Task 3 owns.
- Task-2 adherence carry-forwards addressed: E1 resolved per D-T3-15; E2/E3 record-only (§13).
- Status: APPROVED (per-task plans auto-approved by the user's "Approve Global and Tasks Plans"
  ruling)
- Branch: `feat/phase05-production-live-runtime`; version 0.8.0; base HEAD `a37cb4f`, clean tree,
  332 tests green (Task 2 landed; gate `logs/checks/20261014T173234Z.log`)
- Phase reports referenced: `.agent/reports/20261011-phase-03-ffmpeg-local-pipeline.md`,
  `.agent/reports/20261012-phase-04-oauth-credentials.md`
- This sub-step: 4.1b (Analysis & Planning). The implementer executes §9 in 4.2; reviewers use
  §11 in 4.3–4.5b.

---

## 1. Verified starting state (fresh reads, 2026-10-14)

- `src/app/modes.rs` holds the `run` exit-3 placeholder (`NOT_IMPLEMENTED_EXIT = 3`) with its own
  4-test module; `src/app/mod.rs` dispatches `cli::Mode::Run => modes::run(app_config)`.
- `src/app/` seams today: `pipeline::{install_interrupt_handler, StopFlag,
  FramePipelineError, prepared_pipeline, PipelineConstruction, MessageSource}` (chat → renderer
  → streaming, generic over `S: FrameSink`); `demo::run_demo` wires `OutputKind::Demo`.
- `src/streaming/`: locked arg vector (`args.rs`), `FfmpegProcess::start(config, output_path)`
  (piped spawn, drain threads joined before `wait`, `finish()` = stdin EOF → drains → exit check
  → non-empty-file check, `Drop` kill), `FrameSink { write_frame, finish }` with
  `CompletedVideo`, `resolve_output_path/OutputKind` (demo only since Task 1 deleted `Session`).
- `src/youtube/` Task-2 layer is complete and staged: `LiveBroadcastPreparer::prepare`
  (O1→O2→O3→readiness→`RtmpsIngestion`), `ReadinessWait` (36×5 s) and
  `wait_until_stream_active` (24×5 s, item-staged), `WaitCanceller` (`new`/`request_cancel`
  item-staged), `RtmpsIngestion::into_url` (the single reveal point, item-staged),
  `GoogleLiveApiTransport::new(AccessToken)`, plus the Phase-04
  `CredentialService`/`GoogleAuthorizationVerifier` staging and the frozen staged allows listed
  in `.agent/project-structure.md` (22 attribute lines + 2 comment hits).
- `VerifiedCredentials { access_token, authorization: VerifiedAuthorization }` is the Phase-04
  service outcome; `VerifiedAuthorization::granted_scopes()` (field + accessor, item-staged) is
  the scope-evidence reader reserved for this wiring.
- `PreparedBroadcast` exposes `broadcast_id()` and `into_ingestion()` but NOT the stream id —
  Task 3's post-start O4 confirmation wait needs it (§4 D-T3-4).

---

## 2. Binding Design Decisions (D-T3-1 .. D-T3-15)

Refinements of D-T1/D-T2 under Task 3's authority. Any deviation requires a planner adjudication.

### D-T3-1 — Module shape: `src/app/run/` directory; `modes.rs` deleted

The placeholder file `src/app/modes.rs` is REPLACED by a directory mode module, mirroring the
`app/auth/` and `app/demo/` precedents:

```text
src/app/run/
├── mod.rs             — run_command exit funnel, ordered orchestration glue, seam parameter objects
├── error.rs           — RunError (8 variants), Display (helper-fn split), exit mapping
├── credentials.rs     — client-file read+parse, production stop construction, service call,
│                        run-side scope-evidence gate
├── broadcast.rs       — preparer construction + StageRuntime param object + clock epoch helper
├── gate.rs            — SpawnedGate: confirmation-wait thread, shared outcome state, join
├── streaming.rs       — StreamingRunner seam + PreparedStreaming (production) + shutdown rules
├── source.rs          — ChatFreeSource (MessageSource impl yielding None always)
└── *_tests.rs (+ runtime_test_fixtures.rs, #[cfg(test)]) — offline test suite
```

`src/app/mod.rs`: `mod modes;` → `mod run;`; dispatch arm `cli::Mode::Run => run::run_command(app_config)`.
The module-level doc exit-code sentence drops the "3 = run not implemented" clause and gains
"run-origin fatal failures" in the exit-1 class. The function/module name overlap (`app::run`
funnel vs `app::run` module) is legal Rust (different namespaces) and disambiguated by naming the
entry `run_command`.

### D-T3-2 — Startup sequence (D-T1-8 verbatim, with ONE documented refinement)

The production order implemented by `run_outcome` / `run_runtime`:

1. **Ctrl+C pair install** (refinement, see below): the app's single `ctrlc` handler flips BOTH
   the frame-loop `StopFlag` AND a `WaitCanceller` clone (D-T2-10's Task-3 contract).
2. **Credentials/authorization**: read+parse `oauth.client_config_path` (first — the auth flow
   D5 order), construct the concrete stops (`FileCredentialStore`, `GoogleTokenEndpoint`,
   `GoogleAuthorizationVerifier`, `ClientConfig`), then
   `CredentialService::establish_authorization()` — load → offline persisted-scope gate →
   verify-first probe → refresh-once → atomic persist → one re-verify (Phase 04 service
   consumed VERBATIM; no OAuth duplication, D-G4).
3. **Run-side permission gate BEFORE any resource creation**: consume
   `VerifiedAuthorization::granted_scopes()` and assert `REQUIRED_SCOPE` membership
   (`GrantedScopes::grants`); failure → `RunError::InsufficientScope` (exit 1). This consumes and
   retires the two `verify.rs` item-level staging allows.
4. **Transport construction**: `GoogleLiveApiTransport::new(verified access_token)` via the
   `TransportFactory` seam (tests inject a fake; §6/D-T3-9), held as an
   `Arc<dyn LiveApiTransport + Send + Sync>` (`BroadcastTransport`) so it can cross into the
   confirmation gate thread.
5. **Broadcast/stream resources → readiness → RTMPS destination** in ONE call:
   `LiveBroadcastPreparer::from_config(config, epoch_now)` + `prepare(&transport, &canceller)` —
   O1 insert stream → O2 insert broadcast → O3 bind (`boundStreamId` asserted) → bounded
   readiness wait (36×5 s, cancellable) → destination extraction. Steps run exactly once each
   (no request duplication; brief §12.2).
6. **Destination reveal (the ONE reveal point)**:
   `RtmpsDestination::new(prepared.into_ingestion().into_url())` — the plain `String` exists only
   inside this expression and is immediately re-wrapped (D-T2-12 boundary ruling). Consumes and
   retires the `into_url` item allow.
7. **FFmpeg start targeting the destination**: `prepared_stream_pipeline(config, StreamParts)`
   → `FfmpegProcess::start_stream(config, destination)` (locked stream vector, D-T3-5 F-S2).
   Spawn failure → `RunError::Pipeline(FramePipelineError::Sink(StreamingError::Spawn))`.
8. **Post-start confirmation gate (the refinement, THREAD-PARALLEL)**: D-T1-8 step 8's
   O4-only `wait_until_stream_active` (24×5 s, cancellable) starts on a dedicated std thread (
   `SpawnedGate`) RIGHT AFTER FFmpeg start, and the frame pipeline (step 9) starts immediately
   after; the gate requests the frame-loop stop and records its typed error if it observes a
   fatal state. The literal sequential reading of D-T1-8 (confirmation fully completes BEFORE
   the frame loop) is REJECTED because the existing pipeline contract requires video to flow
   before YouTube flips `streamStatus` to `active` (FFmpeg opens the RTMPS output connection
   only once stdin frames arrive; `rawvideo` demuxer params are explicit, so no data flows
   before the paced loop writes). Justification per the TODO's own refinement clause
   ("Refine this sequence only if official API constraints or the existing pipeline contract
   require it, and document why"). Everything else in D-T1-8 is byte-consumed.
9. **Continuous chat-free frame loop**: `FramePipeline` with `ChatFreeSource` (`MessageSource`
   always `Ok(None)`); the seam `src/app/pipeline/source.rs` stays BYTE-UNCHANGED for Phase 06;
   no chat source is constructed anywhere.
10. **Ordered shutdown + exit** (D-T3-6 below).

"Startup success" remains D-T1-13's predicate (`streamStatus == active` while FFmpeg is
running): it is MARKED by the gate's `info!` event, not by a blocking barrier — consistent with
"do not mark startup successful until the broadcast is in a streamable state and FFmpeg has
started" (event fires only after both conditions held; frames flowed to make the stream
active). The D-T1-8 note about logging a `lifeCycleStatus == live` observation is SATISFIED AS
VOID for Phase 05: the confirmation wait is O4-only by design (D-T2-9), no O5 polling happens
post-start, and the broadcast reaches `live` via autoStart outside our poll window; nothing
claims otherwise.

### D-T3-3 — Ctrl+C pairing installer (pipeline boundary untouched)

`src/app/pipeline/interrupt.rs` gains ONE new function; `install_interrupt_handler` (demo's)
and its module docs stay byte-unchanged:

```rust
/// Generic stop-request installer used by the run mode (the auth/demo
/// precedent handles keep their own entry): the app's SINGLE ctrlc handler
/// runs the caller-built flag flips. Inherits the documented limitations of
/// this module (single handler, pacing-sleep latency, mid-frame latency, no
/// background-signal claim, unbounded finish wait).
pub(crate) fn install_stop_request_handler(
    request: Box<dyn Fn() + Send>,
) -> Result<(), InterruptHandlerError>
```

The run-mode closure flips `stop_flag.request_stop()` and `canceller.request_cancel()` (both
flags, one handler — D-T2-10's contract, no youtube types in `pipeline`). The
`pub(crate) use interrupt::{install_interrupt_handler, ...}` line in `pipeline/mod.rs` gains the
new name in the same commit as its consumer (an unused crate re-export warns).

### D-T3-4 — `PreparedBroadcast` gains the stream id

`src/youtube/live_ingestion.rs`: add a private `stream_id: String` field set in `prepare` plus
`pub(crate) fn stream_id(&self) -> &str`, because the post-start O4 confirmation wait
(`wait_until_stream_active` via `StreamObservationRequest { stream_id, cancellation }`) needs
the id and `prepare` currently drops it. Debug stays derived and secret-safe (the stream id is
non-secret operator-diagnostics, same class as `broadcast_id`; the redacting `RtmpsIngestion`
field is unchanged). One new fixture test (§8 TU-1).

### D-T3-5 — Streaming output extension (D-T1-10 items (i)–(iv) implemented)

Five streaming files:

- **F-S1 `src/streaming/rtmps_destination.rs` (NEW, ~60 lines)** — the streaming-owned secret
  wrapper (same shape as `RtmpsIngestion`; two wrappers exist because neither module may depend
  on the other): private `url: String`, `pub(crate) fn new(url: String) -> Self`,
  `pub(super) fn as_url(&self) -> &str` (consumed ONLY by the `args`/`ffmpeg` siblings),
  `Debug` = `RtmpsDestination(<redacted>)`, NO `Display` ever, no secret-typed fields beyond
  `url`.
- **F-S2 `src/streaming/args.rs` (EDIT)** — add
  `pub(super) fn build_stream_arguments(config: &AppConfig, destination: &RtmpsDestination) ->
  Result<Vec<String>, StreamingError>`:
  input segment byte-identical to the locked local vector
  (`-nostdin -n -f rawvideo -pix_fmt rgb24 -s <w>x<h> -framerate <fps> -i pipe:0`),
  output group EXACTLY
  `-c:v <video_codec> -preset <preset> -b:v <bitrate> -g <2 × fps> -pix_fmt yuv420p -f flv
  <destination_url>`:
  (i) the destination URL is the final argument; (ii) `-movflags +faststart` REMOVED
  (MP4-only container rewrite; meaningless for the FLV muxer); (iii) `-f flv` ADDED (explicit
  muxer selection — no output extension exists); (iv) `-g <2 × fps>` ADDED (official
  keyframe-interval guidance: target 2 s, never > 4 s; derived from configured FPS).
  The non-rgb24 pixel-format guard (`UnsupportedPixelFormat`) is kept. The local
  `build_arguments` and its vector are byte-unchanged (demo never regresses). Constants:
  `STREAM_MUXER = "flv"`, `GOP_MULTIPLIER: u32 = 2` (named, no magic numbers).
- **F-S3 `src/streaming/ffmpeg.rs` (EDIT)** —
  - struct field rework: `output_path: PathBuf` → `local_output: Option<PathBuf>` plus
    `redaction_target: Option<String>`; NO `#[derive(Debug)]` is added to `FfmpegProcess` (the
    destination lives inside the struct and must never become formattable).
  - `start(config, output_path)` keeps its exact signature/behavior (local spawn, finalize file
    check).
  - NEW `pub fn start_stream(config: &AppConfig, destination: RtmpsDestination) ->
    Result<Self, StreamingError>`: builds the stream args, spawns with piped stdin/stdout/stderr
    and no shell (shared private `spawn_with(config, arguments, local_output, redaction_target)`),
    drains identical. Carries an item-level `#[allow(dead_code)]` with the "consume at the Task-3
    run wiring; retire there" comment until the wiring commit.
  - `finish()` target-aware: stdin EOF → drains joined → `wait()` →
    `report_exit_failure(&status, &captured_stderr, self.redaction_target.as_deref())` →
    finalize check ONLY when `local_output` is `Some` (`OutputFinalize` path extracted into
    `verify_local_output`); success returns `Ok(CompletedOutput::Recording(CompletedVideo))
    ` for local or `Ok(CompletedOutput::Stream)` for RTMPS. Drop kill/reap unchanged.
  - **Mandated redaction pass (D-T1-11/D-T1-13)**: FFmpeg echoes the output URL in its stderr
    (`Output #0, flv, to 'rtmps://…/<key>'`); the 4096-byte trailing-diagnostics path runs
    `diagnostics.replace(destination_url, "[redacted]")` BEFORE any `ExitFailure` payload is
    built. `REDACTED_TEXT = "[redacted]"` named constant; the replace helper replaces every
    occurrence; the read-failure note (`(read failure: …)`) is app-generated text and can never
    carry the URL.
- **F-S4 `src/streaming/sink.rs` (EDIT)** — the finish outcome becomes target-aware and HONEST:
  ```rust
  /// Verified finalization proof of a finished frame sink (D-T1-10): local
  /// recordings verify the MP4 on disk; RTMPS transmission verifies only a
  /// clean FFmpeg exit (no local file exists to check).
  pub enum CompletedOutput {
      /// A verified local MP4 recording (`demo`; the prior `CompletedVideo`
      /// semantics, unchanged).
      Recording(CompletedVideo),
      /// FFmpeg exited cleanly streaming to the RTMPS destination.
      Stream,
  }
  ```
  `FrameSink::finish(self) -> Result<CompletedOutput, StreamingError>`; `CompletedVideo` and its
  docs stay as-is. **This ripples (all mechanical, zero behavior change)**:
  `FramePipeline::run -> Result<CompletedOutput, FramePipelineError>`; `demo/mod.rs`
  `print_final_path` guards on `CompletedOutput::Recording` (same success line byte-for-byte);
  `demo/mod_tests.rs` composition test adapts one line via `let … else` unwrap of the
  `Recording` variant (all timeline/path/byte assertions identical);
  `pipeline/test_helpers.rs` `RecordingSink::finish`/`FailingWriteSink::finish` wrap their
  `CompletedVideo` in `Recording`; `pipeline/runner_tests.rs` sites consuming a `run()` Ok value
  unwrap the same way. The demo MODE's inputs, outputs, exit codes, timeline, and test
  assertions are unchanged — "demo behavior byte-unchanged" (D-G7) is honored; the trait
  signature change is the minimal honest specialization of "finish verification target-aware".
- **F-S5 `src/streaming/mod.rs` (EDIT)** — extend the re-export block ONLY in the same commits
  as consumers appear (an unused crate re-export warns): `rtmps_destination::RtmpsDestination`
  (with its `sink`-change consumer) and nothing else. `video_output.rs` is NOT edited (its
  existing doc sentence already states production uses the RTMPS destination with no local
  file; `OutputKind::Demo` single variant stands).
- **`FramePipeline::run` demonstrates the sink handoff without naming FFmpeg**: the
  `StreamingExecution`/`StreamParts` types below carry `RtmpsDestination` only.

### D-T3-6 — Shutdown order, masking rule, and resource fate

Implementing TODO Task 3's shutdown bullet + D-T1-9, with the Phase-03 single-handler/StopFlag
mechanics (NO chat polling exists — step 1 is a documented structural no-op reserved for
Phase 06):

1. **Stop chat polling**: structural no-op (the run pipeline never constructed a chat source;
   `ChatFreeSource.yields None`). Documented, not a gap.
2. **Cancel in-progress API waits**: the Ctrl+C handler flips `WaitCanceller` TOGETHER with the
   `StopFlag` (one handler, both flags, D-T3-3). On a frame-pipeline FATAL, `run` calls
   `canceller.request_cancel()` BEFORE joining the gate, so a mid-interval wait still ends on
   its next attempt check.
3. **Request the frame-loop stop** (`StopFlag`): the paced loop breaks within ≤ one frame
   interval (existing documented latency).
4. **Close FFmpeg stdin → wait for exit**: the loop's exit path calls `sink.finish()` = stdin
   EOF → drain joins → `wait()` → exit-status check (+ redaction on diagnostics). A frame-write
   fatal skips `finish()` and falls to `Drop` (kill + reap) — existing semantics unchanged.
5. **Bounded wait/kill DECISION**: the Phase-03 documented unbounded-`finish()` limitation
   STANDS UNHARDENED (interrupt.rs module-doc limitation 5 carries; TODO bullet 66 does not
   mandate a timeout field; D-T1-9 already recorded "existing, unchanged"). One NEW bounded-latency
   fact: joining the gate thread costs ≤ one poll interval (5 s) plus any in-flight ureq call
   (no request-level timeout — D-T2-16(a) carried and restated in gate.rs docs).
6. **Cleanup-failure reporting without masking the primary error**: EXACTLY ONE primary
   `error:` stderr line (the demo/auth funnel convention). Primary precedence (deterministic,
   causally justified — the gate-fatal REQUESTED the stop, so any later pipeline/finalize error
   is downstream):
   `GateFinish::Fatal(err)` > `FramePipelineError` > nothing. A superseded (secondary) finalize
   or drain error is dropped from output when a primary exists; when the pipeline error IS the
   primary (gate was `Cancelled`/`Confirmed`), it is reported in full. The pure selector is
   `primary_shutdown_outcome(stream_outcome, gate_finish)` (unit-tested matrix, §8 TA-4).
7. **Exit codes** (see D-T3-8): clean stop 0 (including the Ctrl+C-during-confirmation
   `WaitCancelled` path); any fatal 1.
8. **Broadcast/stream resource fate (D-T1-9 restated; docs live in run module docs + carried to
   Task 4's report):** NEVER delete, NEVER transition. With `enableAutoStop=true` the broadcast
   auto-completes ~1 minute after ingestion ends (official Life-of-a-Broadcast wording). Normal
   Ctrl+C stop: streaming ends cleanly, broadcast completes automatically. Startup failures
   AFTER O1/O2 succeeded (bind/readiness/confirmation/spawn failure): the created `ready`
   broadcast and stream REMAIN (nothing went live — no video flowed, so nothing is left
   `live`); repeat-run accumulation is the documented limitation
   (`userBroadcastsExceedLimit` via `QuotaLike`), operator guidance = delete stale resources in
   YouTube Studio (automated cleanup is out-of-scope). Crash: no own cleanup (brief §12.4;
   Task 4 documents manual recovery). A Ctrl+C during PRE-STREAMING startup (prepare's wait is
   cancelled): `WaitCancelled` maps to the normal-stop path, exit 0, with the distinct
   `RUN_STARTUP_STOP_MESSAGE` line (D-T3-7) — the process never claims the broadcast will
   auto-complete in that case (it never went live).

### D-T3-7 — Success/stop reporting (stdout, NOT tracing)

Mirroring the auth console / demo print conventions (plain `println!`, no tracing event, no
secrets):
- `RUN_STOP_MESSAGE = "streaming stopped; the broadcast auto-completes on YouTube after
  ingestion ends"` — printed on the clean STREAMING stop path (`pipeline.run()` Ok).
- `RUN_STARTUP_STOP_MESSAGE = "run stopped by the stop request before streaming started"` —
  printed when the startup path ended on `LiveApiError::WaitCancelled` (pre-streaming Ctrl+C).
Both constants pinned by tests (§8 TA). New tracing info events (fields whitelisted,
non-secret only): `info!(broadcast_id = prepared.broadcast_id(), "broadcast prepared for
streaming")` (stage 5), `info!("FFmpeg started toward the configured streaming destination")`
(stage 7), `info!("stream confirmed active on YouTube")` (gate thread). NO event field or
message may ever contain the destination, the token, or any scope url beyond the public
`REQUIRED_SCOPE` in the `InsufficientScope` error line.

### D-T3-8 — CLI/usage/exit-code contract

- `auth` and `demo` byte-untouched (modules, usage lines, tests, exit codes).
- `cli.rs` `USAGE` run line: replace the trailing text
  `start the streaming runtime (not implemented yet)` with
  `stream the configured broadcast to YouTube Live` (identical two-space + `run` + padding
  prefix; column alignment preserved).
- Exit codes STABLE: `--help` → 0; usage errors → 2; `StartupError` (LogInit/Configuration) → 1;
  `run` success → 0; every `RunError` fatal → 1 (`AuthError`/`DemoError` analogues unmoved).
  The placeholder code 3 is REMOVED exactly when the complete startup path is wired (this
  task's final wiring commit): the run path no longer has a not-implemented boundary; the four
  prerequisites named by the old placeholder are OAuth (Phase 04) ✓, broadcast lifecycle
  (Task 2) ✓, RTMPS acquisition (Task 2) ✓, and Live Chat — permanently deferred OUT of the
  run path by the user ruling (global plan §1.3 + TODO Task 3 bullet "Do not start or depend
  on a Live Chat source in Phase 05"), so exit 3 has no remaining truthful referent. Encode
  explicitly in the commit message and the adherence report.
- Deleted with `modes.rs`: its message constant, its 4 unit tests (`run_reports_not_implemented
  _exit_code`, `exit_codes_remain_distinct_and_stable` (the non-3 assertions re-home into
  §8 TA-tests), both message tests).
- `cli_tests.rs`: APPEND `usage_text_advertises_the_implemented_run_command` (contains the new
  line) and `usage_text_no_longer_claims_not_implemented` (!contains "not implemented").
- `run` usage-error exit 2 (parser unchanged), config/load failures still 1 (funnel unchanged).

### D-T3-9 — Testability seams (offline-first, zero-judgment)

The run module is seam-injected like `auth`'s `run_with_listener`; `run_command` itself is NEVER
called by tests (the ctrlc handler is process-global; installing it twice fails). Two
test-visible layers:

```rust
// mod.rs — parameter objects (≤2-args rule):
pub(super) struct RunStops { pub(super) stop_flag: StopFlag, pub(super) canceller: WaitCanceller }
pub(super) struct ServiceStops<'a> {           // §2 stage 2's four injected seams
    pub(super) client_config: &'a ClientConfig,
    pub(super) store: &'a dyn CredentialStore,
    pub(super) endpoint: &'a dyn TokenEndpointTransport,
    pub(super) verifier: &'a dyn AuthorizationVerifier,
}
pub(super) type BroadcastTransport = Arc<dyn LiveApiTransport + Send + Sync>;
pub(super) type TransportFactory<'a> =
    Box<dyn FnOnce(AccessToken) -> BroadcastTransport + 'a>;   // tests substitute the fake
pub(super) struct RuntimeStart<'a> {
    pub(super) config: &'a AppConfig,
    pub(super) stops: RunStops,
    pub(super) service: ServiceStops<'a>,
    pub(super) build_transport: TransportFactory<'a>,
}

/// The WHOLE run startup after the Ctrl+C pair install, with injected
/// collaborators + the streaming-stage runner. Production `run_outcome`
/// supplies the concrete seams; tests supply fakes (offline).
pub(super) fn run_runtime(start: RuntimeStart, runner: &dyn StreamingRunner) -> Result<(), RunError>

pub(super) trait StreamingRunner: Send {
    /// Production: starts FFmpeg toward the destination, launches the startup
    /// gate, runs the paced chat-free loop, completes the ordered shutdown.
    fn run(self, app_config: &AppConfig, execution: StreamingExecution) -> Result<(), RunError>;
}
pub(super) struct StreamingExecution {
    pub(super) stop_flag: StopFlag,
    pub(super) canceller: WaitCanceller,
    pub(super) destination: RtmpsDestination,   // redacting Debug; no url accessor
    pub(super) stream_id: String,
    pub(super) gate: SpawnedGate,
}
```

- Production runner: `pub(super) struct PreparedStreaming;` — calls
  `start_stream_pipeline` (runner.rs sibling constructor), spawns/leaves the gate as provided,
  runs `pipeline.run()`, applies the D-T3-6 rules.
- `run_runtime` body order (≤50 lines): `authorize_from_service` → `authorize_scope_evidence`
  → `GoogleLiveApiTransport` factory call → `prepare_broadcast` → `stream_id` copy →
  `broadcast prepared` info → destination reveal → `spawn_startup_gate` →
  `runner.run(config, StreamingExecution { … })`.
- Test doubles (in `runtime_test_fixtures.rs`, `#[cfg(test)]`, the `live_test_fixtures` /
  `config::test_fixtures` precedent — but defined IN `app/run` because the youtube-side fixture
  is `pub(super)` to `youtube`): `ScriptedTransport` (Mutex-queue + op-recording + command
  captures; Mutex NOT RefCell — the gate thread requires `Send + Sync`), scripted stream
  builders via `serde_json::from_value` into the `pub(crate)`-field models (the youtube-side
  JSON builders are not reachable), credential fakes (`FakeStore`/`FakeEndpoint`/
  `FakeVerifier` implementing the Phase-04 traits), and the recording `StreamingRunner` fake.
  Fixture material stays `-fixture`-shaped (`rtmps://ingest.example.test/…`,
  `stream-name-fixture`); NEVER real-shaped.
- The gate/consumer of production constants stays production-only; gate tests use ONLY
  zero-sleep outcomes (§8 TB rules) so no unit test ever sleeps 5 s.
- `Send + Sync` note for the implementer: `GoogleLiveApiTransport` holds a `ureq::Agent` +
  `AccessToken` (String) — both `Send + Sync` by design; if the compiler disagrees on a locked
  ureq 2 version, the ONLY sanctioned adaptation is a one-field wrapper struct
  `SharedTransport(Mutex<GoogleLiveApiTransport>)` implementing `LiveApiTransport` by locking
  per call (harmless: after `prepare`, the gate thread is the sole user of the transport).
  Record whichever shape compiled in the adherence report; do NOT invent a third form.

### D-T3-10 — `prepared_stream_pipeline` (runner sibling, zero demo ripple)

`src/app/pipeline/runner.rs` adds — WITHOUT touching `prepared_pipeline`,
`PipelineConstruction`, or `OutputKind` (the demo construction path stays byte-identical):

```rust
pub(crate) struct StreamParts {
    pub(crate) source: Box<dyn MessageSource>,
    pub(crate) stop_flag: StopFlag,
    pub(crate) destination: RtmpsDestination,
}
pub(crate) fn prepared_stream_pipeline(
    config: &AppConfig,
    parts: StreamParts,
) -> Result<FramePipeline<FfmpegProcess>, FramePipelineError>
```

Body = the existing production assembly (`PipelinePacing::from_config`, `build_chat_store`,
`build_renderer`, `SystemFrameClock`) with the ONLY difference `FfmpegProcess::start_stream
(config, parts.destination)` where `prepared_pipeline` uses `resolve_output_path` +
`start`. `FfmpegProcess::start_stream`'s staging allow retires at this consumption.

### D-T3-11 — Redaction surface enumeration (the audit list; every entry test-pinned or
grep-audited at adherence)

Reachable surfaces of the RTMPS destination / stream key in the WIRED runtime:

1. **Spawn argv**: `std::process::Command` argv exists only in memory; never printed, never
   logged, never in any error (`StreamingError::Spawn` carries executable + io cause only —
   text tested to be destination-free).
2. **FFmpeg stderr drain diagnostics** (`ExitFailure.diagnostics`, 4096-byte tail): the ONLY
   runtime surface FFmpeg can echo the URL through — covered by the mandated redaction pass
   (F-S3) + process-level test with a fake key-shaped URL.
3. **Error Display/Debug**: `RtmpsDestination`/`RtmpsIngestion` Debug = `<redacted>` markers,
   no `Display`; `PreparedBroadcast`/`StreamingExecution` types hold the wrapper only and
   their Debug inherits the redaction (test-pinned); `LiveApiError` can never carry the
   destination (D-T1-4, no secret-typed field); `RunError` payloads are typed inner errors
   already secret-free (Display test);
4. **Panic messages**: production run code contains NO `unwrap`/`expect`/index panics on the
   destination paths; gate-thread panics surface only as the typed `RunError::GateThreadStopped`
   (join error), never as text.
5. **Tracing fields/messages**: whitelist only (`command`, `config_path`, `exit_code`,
   `broadcast_id`) + message-only events with no secret material (D-T3-7); grep-audited at
   adherence (`git grep -n "rtmps\\|stream key\\|ingest" src/app src/streaming` must show only
   test fixtures + doc-comment citations).
6. **`FfmpegProcess` struct**: NO Debug derive introduced; `redaction_target` private.
7. **Debug output of the execution bundle**: `RtmpsDestination`'s Debug marker asserted via the
   recorded `StreamingExecution` in the orchestration test.
8. **Fake-script argv capture** (one unix-gated test): the argv assertion happens INSIDE the
   test process against FIXTURE-shaped material; no capture file persists beyond the TempDir.

### D-T3-12 — Logging of the ready→live transition (recorded, voided behavior)

Post-start, the runtime polls O4 ONLY (D-T2-9); the broadcast's `live` transition (via
autoStart) is never observed, so no lifecycle log exists — recorded in the gate module docs so
no later agent assumes an O5 poll was forgotten. Resource fate/monitoring guidance belongs to
docs (Task 4) + YouTube Studio.

### D-T3-13 — No new dependencies; no Cargo.toml/Cargo.lock diff (D-G11/D-T1-1/D-T2-15).

### D-T3-14 — Code rules on every new/edited file: ≤200 lines (split test files or display
helpers BEFORE review if required), ≤50-line bodies, ≤2 depth, ≤2 params (param objects per
D-T3-9/F-S5), private-by-default members (`pub(super)`/`pub(crate)` only as listed), no
commented-out code, self-documenting names.

### D-T3-15 — E1 coordinated decision (Task-2 carry-forward; recorded ONE way)

**DECISION: formally record the arg-count EXCEPTION; do NOT amend the consumption surface.**
The tension: `ReadinessWait::new` (4 args), `#[cfg(test)] ReadinessWait::with_polling` (6),
`#[cfg(test)] wait_until_stream_active_with_polling` (4), private `confirmation_poll` (4) vs
the ≤2-args guideline. Rationale recorded verbatim in the Task-3 plan + adherence report:
(a) the shapes are the binding "exact API shape" of D-T2-9, already consumed by Task 2's own
orchestrator — amending them post-hoc rewrites the binding Task-2 plan text and churns its
roster for zero production effect; (b) the two multi-arg consumers Task 3 actually touches are
`#[cfg(test)]` re-injection seams (test-only; production argument-weight rules target the
production surface); (c) Task 3's ONE production consumption point of the wait family —
`wait_until_stream_active(&transport, &StreamObservationRequest)` — already satisfies ≤2 args
via its D-T2-9 param object; (d) every NEW Task-3 surface in this plan respects ≤2 args via
parameter objects (`RuntimeStart`, `ServiceStops`, `RunStops`, `StageRuntime`, `StreamParts`,
`StreamingExecution`, `GatePlan`, `RecordingNameSpec` precedent). The ≤2-args rule itself is
NOT relaxed; this is a named, recorded exception for four pre-existing signatures, re-verified
(not expanded) at adherence.

---

## 3. File plan (exact; deltas define the permitted diff surface)

### Streaming (per §9 commit C2)

| # | File | Action | Notes |
|---|---|---|---|
| SP1 | `src/streaming/rtmps_destination.rs` | NEW | D-T3-5 F-S1; `#[cfg(test)] #[path = "rtmps_destination_tests.rs"] mod rtmps_destination_tests;` |
| SP2 | `src/streaming/args.rs` | EDIT | F-S2 additions; `build_arguments` byte-unchanged |
| SP3 | `src/streaming/ffmpeg.rs` | EDIT | F-S3 (`start_stream` item-staged; `finish` target-aware; redaction) |
| SP4 | `src/streaming/sink.rs` | EDIT | F-S4 (`CompletedOutput`, trait signature) |
| SP5 | `src/streaming/mod.rs` | EDIT | re-export `RtmpsDestination` (+ consumer-locked order) |
| SP6 | `src/streaming/args_tests.rs` | EDIT | +2 locked-shape tests |
| SP7 | `src/streaming/rtmps_destination_tests.rs` | NEW | 2 tests |
| SP8 | `src/streaming/ffmpeg_stream_tests.rs` | NEW | 3 process tests (unix-gated) |
| SP9 | consumer adaptations in the SAME commit: `src/app/pipeline/runner.rs` (run() signature), `src/app/demo/mod.rs` (print match), `src/app/demo/mod_tests.rs` (1 line), `src/app/pipeline/test_helpers.rs` (2 finish bodies), `src/app/pipeline/runner_tests.rs` (CompletedOutput unwraps) | EDIT | zero behavior change |

### YouTube (per §9 C3)

| # | File | Action | Notes |
|---|---|---|---|
| Y1 | `src/youtube/live_ingestion.rs` | EDIT | `stream_id` field + accessor; item-staged allow until wiring |
| Y2 | `src/youtube/live_ingestion_tests.rs` | EDIT | +1 test (stream id accessor) |

### Wired runtime (per §9 C4 — the big commit)

| # | File | Action |
|---|---|---|
| A1 | `src/app/run/mod.rs` | NEW (~150): exit funnel, `run_outcome`, `run_runtime`, `RunStops/ServiceStops/RuntimeStart/TransportFactory/BroadcastTransport`, messages |
| A2 | `src/app/run/error.rs` | NEW (~150): `RunError` 8 variants (`Interrupt(InterruptHandlerError)`, `ClientConfig(ClientConfigError)`, `EndpointInit(TokenEndpointError)`, `Credential(CredentialServiceError)`, `InsufficientScope`, `Broadcast(LiveApiError)`, `Pipeline(FramePipelineError)`, `GateThreadStopped`), Display via per-group helpers, `is_stop_request_cancellation`, exit mapping (all 1; cancellation 0) |
| A3 | `src/app/run/credentials.rs` | NEW (~100): `read_client_config` (private duplicate of auth/flow.rs's helper — the duplication-over-cross-module-contract precedent; ~12 lines: `fs::read_to_string` + `parse_client_config`), `establish_credentials(app_config)` (production stop construction + service call), `authorize_from_service(&CredentialService)`, `authorize_scope_evidence(&VerifiedCredentials)` |
| A4 | `src/app/run/broadcast.rs` | NEW (~70): `StageRuntime<'a> { transport, cancellation }`, `prepare_broadcast(app_config, &StageRuntime)` (from_config + prepare), `epoch_seconds_now()` (SystemTime; saturating-or-0 fallback: a fallback 0 yields 1970 scheduledStartTime → server `invalidScheduledStartTime` → typed `InvalidMetadata`, actionable — documented) |
| A5 | `src/app/run/gate.rs` | NEW (~130): `GatePlan { transport, cancellation, stream_id, stop_flag }`, `spawn_startup_gate` (std thread), private `run_confirmation` (Ok → Confirmed + info!; `WaitCancelled` → Cancelled; other → record fatal + `stop_flag.request_stop()`; Mutex access via `if let Ok` — no panics), `SpawnedGate::finish(self) -> GateFinish { Confirmed, Cancelled, Fatal(LiveApiError), Panicked }` (join, then read shared `Arc<Mutex<GateState>>`) |
| A6 | `src/app/run/streaming.rs` | NEW (~120): `StreamingRunner` trait + `StreamingExecution` + `PreparedStreaming` impl + `primary_shutdown_outcome(stream_outcome, gate_finish)` pure selector (D-T3-6 rule 6) |
| A7 | `src/app/run/source.rs` | NEW (~35): `ChatFreeSource` (`MessageSource` → `Ok(None)` always) |
| A8 | `src/app/run/*_tests.rs` + `runtime_test_fixtures.rs` | NEW: rosters §6 TA/TB/TC/TD/TE |
| A9 | `src/app/modes.rs` | DELETE (placeholder + its 4 tests) |
| A10 | `src/app/mod.rs` | EDIT: `mod run;` ↔ `mod modes;`, dispatch arm, module-doc exit paragraph |
| A11 | `src/app/cli.rs` + `src/app/cli_tests.rs` | EDIT: run usage line; +2 usage tests |
| A12 | `src/app/pipeline/interrupt.rs` + `mod.rs` | EDIT: add `install_stop_request_handler` (+ its re-export line, consumer-locked); existing handler/docs byte-unchanged |
| A13 | `src/app/pipeline/runner.rs` | EDIT: add `StreamParts` + `prepared_stream_pipeline` (consumed same commit — no staging needed) |
| A14 | `src/youtube/mod.rs` | EDIT: delete the 10 module-level allows + their staging comments; add production re-exports (`api_transport::LiveApiTransport`, `credential_service::{CredentialService, VerifiedCredentials}`, `google_authorization_verifier::GoogleAuthorizationVerifier`, `google_live_api_transport::GoogleLiveApiTransport`, `live_error::LiveApiError`, `live_ingestion::{LiveBroadcastPreparer, PreparedBroadcast}`, `live_models::{BindCommand, CreateBroadcastCommand, CreateStreamCommand, LiveBroadcast, LiveStream}`, `readiness_wait::{StreamObservationRequest, wait_until_stream_active}`, `rtmps_ingestion::RtmpsIngestion`, `wait_cancellation::WaitCanceller`, `token::AccessToken`); rewrite the staging paragraph (wiring landed; residual re-stages below). `readiness_wait/mod.rs` (`StreamObservationRequest` widen `pub(super)`→`pub(crate)`; add `pub(crate) use confirmation_wait::wait_until_stream_active;`) |
| A15 | item-level allow deletions (consumed): `rtmps_ingestion.into_url`, `wait_cancellation::{new, request_cancel}` (+ module allow via A14), `readiness_wait/confirmation_wait.wait_until_stream_active`, `verify.rs` `granted_scopes` field + accessor, `streaming/ffmpeg.start_stream` | triggered by A3/A4/A5/A7/A10's consumers |
| A16 | `.agent/project-structure.md` | EDIT: app/run map, streaming deltas, allow-inventory recount, v-notes |

Explicitly NOT touched: `src/main.rs`, `src/app/auth/*`, `src/app/demo/{schedule,source}.rs`,
`src/app/pipeline/{clock,pacing,error,source}.rs` (source seam BYTE-unchanged — Phase 06),
`src/chat/*`, `src/renderer/*`, `src/config/*`, `config/config.example.json`, `Cargo.toml`,
`Cargo.lock`, `docs/*`, `README.md` (Task 4), `Dockerfile`, `docker-compose.yml`, `scripts/`.

---

## 4. Exit-funnel and flow details (junior-definitive)

```rust
// run/mod.rs — the exit funnel (mirror of demo/auth patterns)
pub(super) fn run_command(app_config: &AppConfig) -> u8 {
    match run_outcome(app_config) {
        Ok(()) => { println!("{RUN_STOP_MESSAGE}"); RUN_SUCCESS_EXIT }
```
Handling detail pinned IN THE PLAN (no implementer judgment):
- `run_command` matches three arms:
  1. `Ok(())` → `println!("{RUN_STOP_MESSAGE}")` → 0.
  2. `Err(error) if error.is_stop_request_cancellation()` →
     `println!("{RUN_STARTUP_STOP_MESSAGE}")` → 0.
  3. `Err(error)` → `eprintln!("error: {error}")` → `1`.
- `is_stop_request_cancellation(&self) -> bool` = `matches!(self,
  RunError::Broadcast(LiveApiError::WaitCancelled))` (single call — boolean rule).
- `report_run_failure` is the demo-identical two-liner; app/mod.rs's `mode finished` info event
  continues to carry the return code (existing funnel; no duplicate stderr line).
- ALL `RunError` variants map to `STARTUP_FAILURE_EXIT` (1) except the cancellation class above.
- The gate-fatal outranking rule, cancel-before-join, and message selection live ONLY in
  `PreparedStreaming::run` + `primary_shutdown_outcome` (A6) — production/test parity via the
  recorded-sequence fake (§8 TD-5).

---

## 5. Artifact of the startup sequence — checklist for the orchestration test

`run_runtime` must produce, in order, exactly:
`[create_stream, create_broadcast, bind, lookup_stream, lookup_broadcast]` (prepare, one cycle)
`… lookup_stream` (confirmation gate, scripted `active`)
with the credentials probe(s) strictly before `create_stream` (asserted via
`scope_gate_failure_prevents_resource_creation`: scope-gate error → transport ops == `[]`).

---

## 6. Test Roster (offline; no Google, no real broadcast; fakes + fake FFmpeg scripts only)

Fixture-secret confinement: `rtmps://ingest.example.test/ingest/stream-key-fixture`,
`stream-name-fixture`, `access-token-fixture`. Gate tests consume ONLY zero-sleep outcomes.

### TA — `src/app/run/error_tests.rs` + `mod_tests.rs` (4 tests + 1 doc-contract note)
1. `run_origin_failures_map_to_the_startup_failure_code` — one constructed value per
   `RunError` variant (all constructors reachable) → `exit_code() == 1`.
2. `wait_cancelled_maps_to_success_exit_code` —
   `RunError::Broadcast(LiveApiError::WaitCancelled)` → `exit_code() == 0` AND
   `is_stop_request_cancellation() == true`.
3. `run_error_display_lines_are_stable_and_secret_free` — per-variant representative Display
   assertions; NO output contains `rtmps`, `Bearer`, or fixture key material;
   `InsufficientScope` names the public `REQUIRED_SCOPE` (permitted).
4. `primary_shutdown_outcome_matrix_is_deterministic` — the pure selector over
   `(EnvPipelineError-ish stream outcome, GateFinish)` pairs: `(Err, Fatal) → Broadcast(fatal)`,
   `(Err, Cancelled) → Pipeline`, `(Err, Panicked) → Pipeline`, `(Ok, Fatal) → Broadcast(fatal)`,
   `(Ok, Panicked) → GateThreadStopped`, `(Ok, Confirmed|Cancelled) → Ok`.
5. `run_command_is_never_named_by_tests` — doc-contract reminder (not a test): the ctrlc handler
   is process-global; `run_command`/`run_outcome` are exercised ONLY through §8 TD seams.

### TC — `src/app/run/credentials_tests.rs` (3 tests)
1. `credential_service_failure_maps_to_run_error` — `FakeStore` returning
   `CredentialStoreError::ContentNotAvailable` → `RunError::Credential` whose Display contains
   the service's "no stored credentials" text verbatim (delegation contract).
2. `missing_scope_evidence_maps_to_run_insufficient_scope_error` — fake verifier returns
   `VerifiedAuthorization::new(GrantedScopes::from_space_delimited("other-scope-fixture"))`
   around a fake store granting `REQUIRED_SCOPE` (service Ok) → the run-side gate yields
   `RunError::InsufficientScope`.
3. `verified_scope_evidence_passes_the_run_gate` — evidence containing `REQUIRED_SCOPE` → Ok.

### TB — `src/app/run/gate_tests.rs` (3 tests — zero-sleep outcomes only)
1. `confirmed_active_finishes_confirmed` — scripted first lookup `active` → `GateFinish::
   Confirmed`; NO broadcast lookup recorded (O4-only proof).
2. `pre_requested_cancel_finishes_cancelled_with_zero_lookups` → `Cancelled`.
3. `fatal_stream_status_requests_the_stop_and_is_recorded` — scripted lookup → `streamStatus
   "error"` → `GateFinish::Fatal(LiveApiError::BroadcastState{..})` AND the shared
   `StopFlag::is_requested() == true` after `finish()`.

### TD — `src/app/run/runtime_test_fixtures.rs` + `runtime_tests.rs` (5 tests)
Fakes D-T3-9 (Mutex-based, thread-safe; serde_json builders; recording runner).
1. `run_runtime_orchestrates_credentials_then_resources_then_streaming` — happy script
   (§5 ledger asserted exactly); the recorded runner captures
   `format!("{:?}", destination) == "RtmpsDestination(<redacted>)"`, `stream_id` == fixture,
   and reports `Ok`; `broadcast prepared` path consumed `into_ingestion` exactly once (type
   moves).
2. `scope_gate_failure_prevents_resource_creation` — fake verifier evidence WITHOUT the scope →
   `RunError::InsufficientScope`; transport op ledger `[]`; runner never invoked.
3. `broadcast_failure_maps_to_run_error_without_streaming` — scripted O1 failure
   (`LiveApiError::ApiNotFound`) → `RunError::Broadcast` (variant+payload echoed); runner never
   invoked; ledger == `["create_stream"]`.
4. `gate_fatal_stops_streaming_and_primarily_reports_the_wait` — recording runner mirrors the
   production shutdown sequence: spins on `stop_flag` (1 ms sleeps), then
   `canceller`/`gate.finish()`; scripted gate lookup `error` → `RunError::Broadcast` primary;
   proves gate-drive flag flip + join.
5. `shutdown_order_cancels_the_wait_before_joining_the_gate` — the recorded runner simulates a
   frame-pipeline failure, applies the PRODUCTION order (`request_cancel` → `finish`) and
   asserts the gate returns `Cancelled` and the primary is the runner-recorded pipeline error
   (with the doc-note: this script mirrors `PreparedStreaming::run`; update it if production
   ordering changes — the drift risk is documented in the test file).

### TE — `src/app/run/streaming_stage_tests.rs` (1 test, `#[cfg(all(test, unix))]`)
1. `stream_pipeline_passes_the_destination_to_the_ffmpeg_argv` — temp-dir POSIX `#!/bin/sh`
   fake FFmpeg that writes `"$@"` to a capture file and exits 0; config executable pointed at
   the script; stop pre-requested; `prepared_stream_pipeline` (production construction) → pipeline
   runs zero frames → `finish` → `Ok(CompletedOutput::Stream)`; capture's LAST token == the
   `stream-name-fixture` URL token, contains `-f flv` immediately before it, contains NO
   `-movflags`, contains `-g 60` (30 fps × 2). (The ETXTBSY 8-retry mitigation pattern from
   `streaming/ffmpeg_tests.rs` is replicated locally if flakes appear — test-only.)

### TU — `src/youtube/live_ingestion_tests.rs` (+1)
1. `prepared_broadcast_exposes_bound_stream_and_broadcast_ids` — happy script;
   `stream_id() == STREAM_ID_FIXTURE`, `broadcast_id()` accessor (existing), destination
   `into_url` still EXACT (existing D-test semantics preserved).

### TS — streaming appends (7 tests)
1. `stream_arguments_match_the_locked_output_shape` (args): full vector equality; NO
   `-movflags`; `-f flv` immediately before the final URL argument.
2. `stream_gop_is_two_frames_per_second` — fps 30 → `-g 60`; fps 60 → `-g 120`.
3. `destination_debug_is_exactly_the_redacted_marker`.
4. `destination_as_url_exposes_the_assembled_url_to_streaming_siblings`.
5. `clean_stream_exit_returns_the_stream_completion` (ffmpeg_stream, unix): 1-frame script
   exits 0 AFTER `cat > "$1"`-style stdin drain — `finish` → `CompletedOutput::Stream`, NO
   local finalize check.
6. `stream_exit_failure_diagnostics_redact_the_destination` (unix): script echoes
   `Output #0, flv, to 'rtmps://ingest.example.test/ingest/stream-key-fixture'` and exits 1 →
   `ExitFailure.diagnostics` contains `[redacted]`, does NOT contain the fixture URL or
   `stream-key-fixture`.
7. `stream_spawn_failure_never_carries_the_destination` (unix): missing executable path →
   `StreamingError::Spawn` Display contains the executable only — no `rtmps`/key text.

### Ripple adaptations (counted, not new names)
- `demo/mod_tests.rs`: 1 line (`let CompletedOutput::Recording(completed) = … else panic!`);
- `pipeline/runner_tests.rs`: every site consuming a `run()` Ok value unwraps `Recording` the
  same way; `pipeline/test_helpers.rs`: 2 finish bodies wrapped.
- Existing `demo` suite outcomes/counts/assertions unchanged; existing streaming suite outcomes
  unchanged (the local path's `finish` still returns its `CompletedVideo` semantics inside
  `Recording`).

Expected delta: **+26 new tests → ~358 total estimate** (modes.rs's 4 placeholder tests are
DELETED; the OBSERVED gate total is authoritative).

---

## 7. Startup/shutdown rule constants (single source in code; asserted by tests)

| Constant/module | Value |
|---|---|
| `RUN_SUCCESS_EXIT` | 0 |
| `STARTUP_FAILURE_EXIT` | 1 (app/error.rs, unchanged) |
| `USAGE_ERROR_EXIT` | 2 (unchanged) |
| `RUN_STOP_MESSAGE` | "streaming stopped; the broadcast auto-completes on YouTube after ingestion ends" |
| `RUN_STARTUP_STOP_MESSAGE` | "run stopped by the stop request before streaming started" |
| `REDACTED_TEXT` (ffmpeg.rs) | "[redacted]" |
| readiness budget/interval | 36 × 5 s (youtube, unchanged) |
| confirmation budget/interval | 24 × 5 s (youtube, unchanged) |
| `GOP_MULTIPLIER` | 2 |
| `STREAM_MUXER` | "flv" |

---

## 8. Implementation Steps (4.2; sequential; Alpine VM MCP, `vm_status` FIRST; every cargo run
via the Compose pattern §10; NEVER chained commands)

- S0 `git status --porcelain` (empty) + `git branch --show-current`
  (`feat/phase05-production-live-runtime`) + `git log --oneline -5` (HEAD ≥ `a37cb4f`). gitignore
  -compliance check. NO normalization/back-out edits.
- S1 Commit the PLAN (this file) — `git add
  .kilo/plans/20261013-phase05-task3-run-orchestration.md` + commit (§9 ledger).
- S2 Streaming extension (C2): SP1–SP9 in one green commit group — rtmps_destination + args +
  ffmpeg + sink + the 7 streaming tests + the mechanical CompletedOutput ripple (runner/demo/
  test_helpers/runner_tests). `FfmpegProcess::start_stream` carries its item-level staging
  allow. Spot-run: `cargo test --locked streaming` (expect the streaming suite green INCLUDING
  the 7 additions); `cargo test --locked app::pipeline` and `cargo test --locked app::demo`
  (ripple adaptations verified — same counts, same assertions).
- S3 YouTube stream id (C3): Y1+Y2. `PreparedBroadcast::stream_id` carries a one-line item-level
  staging allow ("consume at the run wiring; retire there"). Spot-run `cargo test --locked
  youtube::live_ingestion`.
- S4 The wiring commit (C4): A1–A15 in the designed order WITHIN the commit —
  interrupt installer → run module files + test files → youtube re-exports + staging-allow
  deletions (A14/A15) → `modes.rs` deletion + `app/mod.rs` dispatch + `cli.rs` usage +
  `cli_tests`. NO `src/youtube/*` re-export may be committed unused (consumers same commit).
- S5 Compiler checkpoint: `cargo clippy --locked -- -D warnings`.
  - clean → nothing further; the 10 module allows are fully retired (record).
  - flags on `live_models` / `live_lifecycle` (EXPECTED — documented-but-unread resource fields;
    `is_transition_pending`): re-apply ONE module-level allow on each flagged `mod` declaration
    with the UPDATED V2 contract comment: "wired by the Phase-05 run runtime; the residual
    items are the documented-but-unread resource fields / the pending-state predicate reserved
    for a future phase that reads them — retire when consumption lands".
  - unexpected OTHER flags → do NOT improvise: pause and return the compiler output to the
    caller (planner ruling), per the no-assumption rule.
- S6 Full verification battery (separate invocations, §10):
  `cargo test --locked youtube` (155 + 1 = 156 expected), `cargo test --locked streaming`,
  `cargo test --locked app::run` (+16 expected), fmt check, clippy, then the FULL gate
  `dev-checks.sh` — record `logs/checks/<UTC>.log` + OBSERVED total (~358 estimate).
- S7 CLI smoke inside the container (offline, safe):
  `sh -c "target-path binary --help"`-equivalent via `cargo run --locked -- --help` → assert
  the run line; `cargo run --locked -- run --config config/config.example.json`-like failure
  paths are NOT exercised (no credentials file in the tree; usage errors are parser-covered).
  NO live YouTube/network exercise in this task — the manual end-to-end procedure is Task 4.
- S8 Hygiene: `git status --porcelain` shows ONLY expected files; `git ls-files dist/` empty;
  no config.json/tokens/credentials/MP4s staged; `git diff a37cb4f..HEAD --stat` reviewed against
  §3's delta table.
- S9 A16 (project-structure map) + the redaction/allow grep audit outputs recorded into the
  adherence inputs (`git grep -n -F "allow(dead_code)" src/` → EXPECTED ≈ 8 lines: base 22 − 6
  item − 10 module + 2 re-staged; `git grep -in "liveChat" src/` → zero NEW-code hits; the
  signature surface greps from §11).
- S10 Update `.agent/project-info/context.md` is a COMPLETION-step (4.6) action, not 4.2.

---

## 9. Commit Plan (implementer; feature-branch commits only — NO push, NO merge, NO version
bump; D-G11)

```text
1. docs: track the phase 05 task 3 implementation plan          (S1)
2. feat: extend streaming for the rtmps output target           (S2; includes the
   CompletedOutput ripple + demo behavior preservation)
3. feat: carry the stream id through the prepared broadcast     (S3)
4. feat: wire the production run orchestration                  (S4; delete exit-3 placeholder;
   note in the body: the four prerequisite list is retired — chat deferred by ruling)
5. fix: restage documented-but-unread live model fields flagged by clippy (S5 transcription;
   SKIPPED and recorded as skipped if clippy was clean)
6. docs: map the run runtime files in project structure         (S9)
```

Green-at-each-point is the binding rule: each commit compiles and keeps its touched suites
green (an unused crate re-export or an unretired staged allow both FAIL clippy — hence the
consumer-locked ordering above). Exact wording may adapt to the actual diff shape; commit 4 may
split into two commits if review friction demands — green-at-each-point still binds.

---

## 10. Composition Pattern (for the implementer's Compose invocations; Alpine VM MCP;
`vm_status` first)

```sh
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked streaming"
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked youtube"
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked app::run"
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo fmt --check"
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo clippy --locked -- -D warnings"
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```
Exit codes are authoritative; cargo stderr `Finished` lines may be visually omitted by the MCP
capture. fmt-fix failures: ONLY mechanical `cargo fmt` (never restructure code while
formatting). Clippy checkpoint (S5) and the handling of unexpected outputs follow the
planner-escalation rule — never improvise a fix beyond the two predicted re-stages.

---

## 11. Adherence Expectations (4.3–4.5b reviewers)

- Diff surface vs §3's table; `src/app/pipeline/source.rs`, `src/app/auth/**`, `src/config/**`,
  `Cargo.toml/lock`, `docs/`, `README.md` diffs EMPTY.
- Demo regression: demo module inputs/messages/exit codes byte-equivalent (print fn body now
  guards the Recording variant — same emitted line); demo test COUNTS unchanged; timeline tests
  unchanged.
- Fresh greps recorded: `allow(dead_code)` inventory (≈8 expected lines — computed against the
  fresh grep, never an estimate issued as fact), `allow(unused_imports)` == 0, `liveChat` == 0
  new-code hits, `transition(`/`liveBroadcasts.delete` call sites == 0, `key=` == 0 (doc-only).
- Redaction audit (fresh): `into_url` has EXACTLY ONE production call site (the reveal
  expression) + test sites; `secret()` sites unchanged from Task 2 + none new; NO `Display`
  impl exists on `RtmpsDestination`/`RtmpsIngestion`; NO Debug derive on `FfmpegProcess`; no
  `info!` or error line carries ingestion material.
- Exit-code contract: `--help` 0 / usage 2 / StartupError 1 / run success-clean-stop 0 /
  run fatals 1 / no exit-3 anywhere (`git grep -n "NOT_IMPLEMENTED_EXIT\\|exit 3" src/` == 0).
- E1 transcription present verbatim (the D-T3-15 four signatures + rationale) in the adherence
  report.
- E2/E3 (Task-4 report items): recorded ONLY as carry-forward pointers here — no Task-3 action
  beyond typing.
- Observed-total rule: the gate log's test count is authoritative over the ~358 estimate.
- File/line rules per D-T3-14 on every new/edited file.
- Documentation drift: README/docs make NO production-run claims (Task 4 owns user docs);
  `.agent/project-structure.md` matches the tree after A16.

## 12. TODO Task-3 Bullet — Plan Mapping Self-Check

| TODO Task 3 bullet | Plan coverage |
|---|---|
| Replace run placeholder with production startup loaded+refreshed via Phase 04 credential service; permissions verified before resources | D-T3-2 stages 2–3, D-T3-9, A3; consumption retired the Phase-04 staging |
| Orchestrate startup explicit order; refinements documented | D-T3-2 (9 stages + documented gate-overlap refinement) |
| Extend streaming output minimally; demo keeps MP4; preserve spawn/drain/frame-write/cleanup | D-T3-5 (locked vector diffs exactly (i)–(iv); demo byte-identical behavior) |
| RTMPS URL/key nowhere in command logs/tracing/errors/panics/debug; redaction tests around spawn/exit | D-T3-5 F-S3 pass, D-T3-11 enumeration, §8 TS 5–7, §8 TA-3 |
| Frame loop continuous, chat independent | D-T3-2 stage 9; ChatFreeSource; seam untouched |
| Ctrl+C + fatal shutdown order; stop/cancel waits; stdin; wait; cleanup-failure unmasked | D-T3-6 (8 rules) + A6 primary selector + §8 TB/TD tests |
| Broadcast/stream fate on shutdown + startup failure documented no-auto-delete | D-T3-6 rule 8 (network-level auto-complete; operator per YouTube Studio) |
| Preserve auth/demo + exit contracts; run usage text updated; placeholder removed when complete | D-T3-8 + A9–A11 + the retirement commit note |
| Offline tests (fake transport/chat/FFmpeg) | §6 + §8 (no test contacts Google or runs real streaming) |

---

## 13. Explicitly Out-of-Task (restated for the implementer)

- NO Live Chat artifact (Phase 06); `MessageSource` seam byte-unchanged.
- NO broadcast reuse/dashboard/multi-broadcast; NO retry loops beyond the Task-2 budget waits;
  NO `transition`/`delete` calls; NO duplicate-broadcast-prone re-issuance.
- NO unbounded waits introduced; no timeout field per TODO scope.
- NO `Cargo.toml`/`Cargo.lock` changes; no new dependencies.
- NO docs/README/auth/config-example edits (Task 4); `.agent/project-structure.md` only as A16.
- NO claim of live-API success without a real recorded manual run (Task 4 owns the procedure);
  native validation remains NOT RUN.
- E2/E3 are Task-4 report items — record pointers only.

## 14. Non-Goals / Phase-06 handoff notes

- The chat-source seam (`src/app/pipeline/source.rs`) is THE only Phase-06 entry; run's
  `ChatFreeSource` shows the intended chat-free pre-state and `MessageSource`'s error class is
  the widening point reserved there (existing docs unchanged).
- Broadcast resource automation (`transition`,`delete`) remains unselected; the stuck-broadcast
  operator path stays YouTube Studio (Task 4's streaming-recovery docs).
- A future reader of `live_lifecycle::is_transition_pending` (V2-staged) is the likely consumer
  when a phase starts LOGGING/POLING broadcast states post-start.

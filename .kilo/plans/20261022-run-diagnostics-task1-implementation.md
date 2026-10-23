# Task-1 Implementation Plan — Run-Path Streaming Failure Diagnostics (FFmpeg exit/stderr tail + last observed streamStatus)

Plan date: 2026-10-22 · Owner: Architector (Critical Workflow step 4.1b) · Front-end: NO (no 4.1a)
TODO: `.agent/todos/20261022/20261022-todo-2.md` — **Task 1 ONLY** (Task 2 is a separate later cycle)
Global BINDING plan: `.kilo/plans/20261022-run-streaming-diagnostics-mfk.md` (§ "Encoded Technical Direction (Task 1 — diagnostics)" — NOT re-litigated, encoded verbatim below)
Branch: `feat/run-streaming-diagnostics-mfk` at HEAD `9e9da4a chore: bump version to 0.10.0`, tree clean. Version is ALREADY bumped (workflow step 3 done).

---

## 0. Research and Verification Record (verified against the current tree)

### 0.1 (a) How the redacted stderr tail + exit status travels — carrier design VERDICT

Verified flow facts (read directly from the tree):

- `FfmpegProcess::finish` (`src/streaming/ffmpeg.rs` lines 107–120) joins both drain threads, gets `CapturedPipe`s, waits the child, and calls `report_exit_failure(&status, &captured_stderr)`. Redaction happens ONLY inside that helper via `exit_diagnostics(stderr, self.redaction_target.as_deref())`, and ONLY when the status failed (`ExitFailure` payload). A clean exit returns the data-less `CompletedOutput::Stream` — evidence is dropped.
- `FramePipeline::run` (`src/app/pipeline/runner.rs` line 77–91) returns the sink's `CompletedOutput` verbatim; sink errors wrap as `FramePipelineError::Sink(StreamingError)`.
- `PreparedStreaming::run` (`src/app/run/streaming.rs` line 77) does `pipeline.run().map(|_completed| ())` — the Ok completion is discarded on the run path (this is the incident gap).
- `exit_diagnostics` (`src/streaming/pipe_drain.rs` lines 46–51) = `trailing_bytes` (4096-byte tail cap, `DIAGNOSTIC_CAPTURE_LIMIT_BYTES`) + read-failure note + `redact_destination` (URL → `REDACTED_TEXT = "[redacted]"` from `ffmpeg.rs`).

Carrier design verdict (encoding the global plan's "typed carrier through the stream completion type"):

- New struct **`StreamExit`** in `src/streaming/sink.rs`:
  - `exit_code: Option<i32>` — verbatim from `ExitStatus::code()` (`None` = signal death).
  - `stderr_tail: String` — the tail AFTER `exit_diagnostics` has run (redacted At Capture Time, see 0.1.1).
  - Fields private; `pub(super)` constructor (only `ffmpeg.rs`, the sibling, builds it); `pub(crate)` accessors `exit_code()` and `stderr_tail()`; `#[derive(Debug)]` is safe (contents are redacted text + an integer) and is required by test assertions. NO `Display` — the tail is consumed only by app-layer tracing lines.
- `CompletedOutput::Stream` becomes the one-field tuple variant `Stream(StreamExit)`. Every other variant (`Recording(CompletedVideo)`) is byte-unchanged.
- Re-export added in `src/streaming/mod.rs`: `StreamExit` joins the `pub(crate) use sink::{CompletedOutput, FrameSink};` line.

### 0.1.1 Redaction location — verified and MOVED to capture time

Verified: today redaction runs only in `report_exit_failure` (call-site of `exit_diagnostics`). After this change, `FfmpegProcess::finish` computes `exit_diagnostics(&captured_stderr, self.redaction_target.as_deref())` ONCE, immediately after `child.wait()`, and BOTH consumers receive the already-redacted text:

1. the `ExitFailure` error payload (`report_exit_failure` receives `&str` instead of `&CapturedPipe`, and `.to_string()`s it into the payload), and
2. the new `CompletedOutput::Stream(StreamExit { .. })` carrier.

Consequence enforced: every consumer (error Display or carrier accessor) gets redacted text only — no path can formattably carry the destination URL. `StreamingError::ExitFailure` Display text is byte-unchanged (`"FFmpeg exited with failure status {N}; captured diagnostics follow: …"`). `StreamingError` fields/Display are NOT touched.

### 0.2 (b) The gate.rs extension — verified shapes

- `run_confirmation` (gate.rs lines 97–118) is the ONLY place `GateFinish::Fatal` is recorded; the gate thread already owns the `Arc` transport clone (`plan.transport`). The fatal arm currently does: stop-flag flip + `GateFinish::Fatal(error)`.
- The confirmation-wait timeout surfaces as `Err(LiveApiError::ReadinessTimeout { stage })` produced by `attempt_exhausted(CONFIRMATION_STAGE)` in `src/youtube/readiness_wait/confirmation_wait.rs` (stage `"ingestion-confirmation"`). Only the gate path ever produces this on the post-start side, so matching `matches!(error, LiveApiError::ReadinessTimeout { .. })` is sufficient and stage-agnostic — no new dependency on the private `CONFIRMATION_STAGE` constant.
- One extra `lookup_stream` is performed there; ALL outcomes tolerated:
  - `Ok(Some(stream))` with a raw label → info line naming the label. The label is read from the RAW documented field (`LiveStream.status.stream_status: Option<String>`, `pub(crate)` fields per `src/youtube/live_models/stream_resource.rs`) — NOT via `StreamStatusKind::from_api_label`, so an unknown label is still logged (diagnostic, never gating).
  - `Ok(Some(stream))` without a label, `Ok(None)`, and `Err(_)` → one info line naming the unavailable fact (never fatal, never a RunError variant).
- For offline zero-sleep testing of the timeout path, the gate gets a cfg-split wrapper that calls the EXISTING test-only re-injection `wait_until_stream_active_with_polling` (already `#[cfg(test)] pub(crate)` in `src/youtube/readiness_wait/confirmation_wait.rs`; its re-export in `src/youtube/readiness_wait/mod.rs` line 18–19 is `#[cfg(test)] pub(super)` — it must be widened to `#[cfg(test)] pub(crate)` so the crate root can re-export it). This is a TEST-BUILD-ONLY seam; production `wait_until_stream_active` (24 × 5 s) and all 14 shared wait-machinery tests stay byte-untouched.

### 0.3 (c) What the demo path must NOT change — verified

- `src/app/demo/mod.rs` `print_final_path` matches `CompletedOutput::Recording(recording)` — untouched by the Stream-variant payload.
- The finalize line `"demo recording finalized: …"` is byte-stable.
- Demo never constructs `CompletedOutput::Stream` (verify by the surface inventory in 0.6 — only `ffmpeg.rs` line 117 constructs `Stream`).
- Demo display text of `Recording` and `DemoError` Display lines are pinned in `app/demo/mod_tests.rs` — untouched.

### 0.4 (d) Test strategy — verified feasible offline

All tests loopback/fake only; no Google. Feasibility evidence gathered per test in §6. Tracing-capture is deliberately NOT used (the `console_tests` subprocess-re-exec precedent is too heavy here): the message constants are pinned by direct constant assertions, the pure decision helpers are unit-tested directly, and the gate wiring is pinned by the transport operation ledger (exactly-one-extra-lookup on the timeout path, exactly-none on other fatals) — together these pin the behavior end-to-end without any log capture.

### 0.5 (e) Rule-cap checks — verified

Current line counts (whole-file reads), all within the 200-line cap with the planned additions:

| File | Now | Planned add (≈) | After (≈) | Verdict |
| --- | --- | --- | --- | --- |
| `src/app/run/gate.rs` | 122 | ≈ +48 (2 constants + 4 helpers + cfg wrapper) | ≈ 170 | OK |
| `src/app/run/streaming.rs` | 106 | ≈ +38 (2 constants + 4 helpers + wiring) | ≈ 145 | OK |
| `src/streaming/ffmpeg.rs` | 143 | ≈ +7 / −4 (capture-time tail + carrier) | ≈ 147 | OK |
| `src/streaming/sink.rs` | 57 | ≈ +30 (StreamExit) | ≈ 88 | OK |
| `src/streaming/mod.rs` | 49 | +1 re-export line | 50 | OK |
| `src/app/run/error.rs` | 89 | 0 | 89 | byte-stable |
| `src/app/run/mod.rs` | 185 | 0 | 185 | untouched |
| `src/youtube/mod.rs` | — | +1 `#[cfg(test)]` re-export line | — | label kept |

No sibling split is required. Body/arg/boolean rules are honored by the helpers specified in §5 (each ≤ 50 lines, ≤ 2 params, single-section booleans via named predicate helpers).

### 0.6 Surface inventory — the complete `CompletedOutput::Stream` touch map (verified by `grep`)

Construction sites: `src/streaming/ffmpeg.rs:117` (only production site).
Pattern matches that break and MUST be updated:
- `src/streaming/ffmpeg_stream_tests.rs:77` — `assert!(matches!(completed, CompletedOutput::Stream));`
- `src/app/run/streaming_stage_tests.rs:55` — same shape.
Pattern matches that stay untouched: `ffmpeg_real_tests.rs:76`, `ffmpeg_tests.rs:92`, `app/demo/mod.rs:98` + `mod_tests.rs:66`, `app/pipeline/runner_tests.rs:44`, `app/pipeline/test_helpers.rs:133,164` (all `Recording`).

---

## 1. High-Level Approach

1. **Git handling (implementer)**: stay on `feat/run-streaming-diagnostics-mfk` (already at the version-bump commit `9e9da4a`, tree clean). Commit the per-task plan file first (precedent: HTTP-411 plan-tracking commit `d11501a`), then code, then tests, then the 4.3-review/4.4-docs commits — repo commit style (`type: lowercase summary`). NO push from 4.2 (pushes belong to workflow step 5; branch-only work stays on the feature branch).
2. **Code changes** (3 files in `streaming/`, 2 files in `app/run/`, 1 re-export line in `youtube/`): StreamExit carrier redacted at capture time → through `FramePipeline::run` → into `PreparedStreaming::run`, where two dedicated info/warn evidence lines are emitted BEFORE the gate join (so FFmpeg evidence always precedes the single final `error:` line); on the gate's Fatal(ReadinessTimeout) path only, one tolerated final `lookup_stream` names the last observed streamStatus label (or the lookup-failure fact) via info lines.
3. **Console commands**: the established Alpine-VM Compose pattern; fmt → targeted tests → clippy → full gate (`ALL CHECKS PASSED` + observed test count + log path) → both release-build scripts + `ls -l` → hygiene/secret audit.
4. **Tests**: offline fakes/loopback only, D7-style binding names (§6); every pinned existing contract re-pinned by unchanged tests (§7 checklist).
5. **Docs handoff**: exact notes for the 4.4 docs-specialist (what the new lines mean + operator diagnosis procedure + Studio cleanup reminder), metadata refreshes (`project-structure.md`, `tech.md`, `context.md`) listed per owner.
6. **Stop-and-report conditions** for anything not encoded here (§11).

---

## 2. Exact Code Steps (junior-executable; before/after per edit)

### Step 2.1 — `src/streaming/sink.rs`: add the `StreamExit` carrier + retarget the `Stream` variant

Open `src/streaming/sink.rs`. Two edits.

**Edit A** — add the struct after the `CompletedVideo` impl block (before the `FrameSink` trait), aligned with the existing doc-comment style:

```rust
/// Exit/exit-evidence bundle of a FINISHED streaming FFmpeg process
/// (run diagnostics task): the exit code verbatim (`None` = signal
/// death) and the captured stderr tail AFTER the destination has been
/// redacted to `[redacted]` — the redaction happens at CAPTURE time in
/// `FfmpegProcess::finish`, so no consumer ever sees the URL. No
/// `Display`: app-layer tracing consumes the accessors.
#[derive(Debug)]
pub struct StreamExit {
    exit_code: Option<i32>,
    stderr_tail: String,
}

impl StreamExit {
    /// The streaming-family constructor (`ffmpeg.rs` is the only
    /// production builder; tests never fabricate carriers).
    pub(super) fn new(exit_code: Option<i32>, stderr_tail: String) -> Self {
        Self {
            exit_code,
            stderr_tail,
        }
    }

    /// The child's exit status code; `None` means it died by signal.
    pub(crate) fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// The redacted trailing stderr diagnostics (≤ 4096 bytes).
    pub(crate) fn stderr_tail(&self) -> &str {
        &self.stderr_tail
    }
}
```

**Edit B** — retarget the variant in `CompletedOutput` (update ONLY the `Stream` arm; keep the `Recording` arm and both doc comments verbatim, and refresh the variant's first doc line to name the carrier):

```rust
    /// FFmpeg exited cleanly streaming to the RTMPS destination; the
    /// payload carries its exit status and redacted stderr tail for the
    /// app's shutdown evidence lines.
    Stream(StreamExit),
```

### Step 2.2 — `src/streaming/mod.rs`: re-export the carrier

In the `pub(crate) use` block (line 40), change:

```rust
pub(crate) use sink::{CompletedOutput, FrameSink};
```
→
```rust
pub(crate) use sink::{CompletedOutput, FrameSink, StreamExit};
```

(The comment-labeled `#[cfg(test)]` `CompletedVideo` re-export stays byte-unchanged.)

### Step 2.3 — `src/streaming/ffmpeg.rs`: redact the tail AT CAPTURE TIME and carry it

**Edit A** — change `report_exit_failure` to consume the pre-redacted tail (≤ 2 args; single-section boolean preserved via the existing `status.success()` early return). Before (lines 80–92), after:

```rust
    /// Non-zero exit maps to the typed failure whose diagnostics are the
    /// PRE-REDACTED tail computed by `finish` — the redaction happened at
    /// capture time, so this helper never sees the destination URL.
    fn report_exit_failure(
        &self,
        status: &ExitStatus,
        stderr_tail: &str,
    ) -> Result<(), StreamingError> {
        if status.success() {
            return Ok(());
        }
        Err(StreamingError::ExitFailure {
            exit_code: status.code(),
            diagnostics: stderr_tail.to_string(),
        })
    }
```

**Edit B** — in `FrameSink::finish`, after the `child.wait()` and before `report_exit_failure`, compute the tail once and thread it into the carrier. Before (lines 111–120 relevant span):

```rust
        let status = self
            .child
            .wait()
            .map_err(|source| StreamingError::WaitFailed { source })?;
        self.report_exit_failure(&status, &captured_stderr)?;
        let Some(output_path) = self.local_output.take() else {
            return Ok(CompletedOutput::Stream);
        };
```
→ after:
```rust
        let status = self
            .child
            .wait()
            .map_err(|source| StreamingError::WaitFailed { source })?;
        let stderr_tail = exit_diagnostics(&captured_stderr, self.redaction_target.as_deref());
        self.report_exit_failure(&status, &stderr_tail)?;
        let Some(output_path) = self.local_output.take() else {
            return Ok(CompletedOutput::Stream(StreamExit::new(
                status.code(),
                stderr_tail,
            )));
        };
```

(No other line of `finish` moves; the local-recording path's `verify_local_output(...).map(CompletedOutput::Recording)` stays byte-identical — the unused tail on that path is harmless since `redaction_target` is `None` there and the value is constructed only on the stream arm.)

### Step 2.4 — `src/app/run/streaming.rs`: emit the shutdown evidence lines from the carrier

**Edit A** — imports: add `warn` to the tracing use and `StreamExit` to the streaming use:

```rust
use tracing::{info, warn};
```
and
```rust
use crate::streaming::{CompletedOutput, RtmpsDestination, StreamExit};
```

**Edit B** — add the two single-source-of-truth message constants (directly below the `StreamingExecution`/above `PreparedStreaming`; WATCH_LINK_MESSAGE precedent):

```rust
/// The info event at the END of a streaming run when FFmpeg exited
/// cleanly (exit status 0): the operator log records the exit AND the
/// redacted stderr tail (brief §11.1). Pinned verbatim by a unit test.
pub(super) const FFMPEG_CLEAN_EXIT_MESSAGE: &str =
    "FFmpeg streaming process exited cleanly (exit status 0)";

/// The warn event when FFmpeg exited non-zero during the streaming run
/// (signal death included): NEVER a second `error:` line — the single
/// `error:`-line contract holds (plan-encoded choice). Pinned verbatim.
pub(super) const FFMPEG_EXIT_FAILURE_MESSAGE: &str =
    "FFmpeg streaming process exited with a failure status";

/// The note substituted when the captured stderr tail is EMPTY so the
/// event text stays truthful without an awkward empty suffix.
pub(super) const FFMPEG_NO_DIAGNOSTICS_NOTE: &str = "none captured";
```

**Edit C** — wire the logging into `PreparedStreaming::run`. Before (line 77–85):

```rust
        let stream_outcome = pipeline.run().map(|_completed| ());
        // D-T3-6 rule 2: on a frame-pipeline FATAL the confirmation wait is
        // cancelled BEFORE the gate join, so a mid-interval wait ends on its
        // next attempt check instead of running to its budget.
        if stream_outcome.is_err() {
            canceller.request_cancel();
        }
        let gate_finish = gate.finish();
        primary_shutdown_outcome(stream_outcome, gate_finish)
```
→ after (the pure selector's `Result<(), …>` signature stays byte-unchanged; the carrier is LOGGED before the map, so the evidence always precedes the gate-join and the final `error:` line):
```rust
        let stream_outcome = pipeline.run();
        log_stream_shutdown_evidence(&stream_outcome);
        // D-T3-6 rule 2: on a frame-pipeline FATAL the confirmation wait is
        // cancelled BEFORE the gate join, so a mid-interval wait ends on its
        // next attempt check instead of running to its budget.
        if stream_outcome.is_err() {
            canceller.request_cancel();
        }
        let gate_finish = gate.finish();
        primary_shutdown_outcome(stream_outcome.map(|_completed| ()), gate_finish)
```

**Edit D** — add the four helpers AFTER `primary_shutdown_outcome` (all `fn` private; each body ≤ 50 lines; each ≤ 2 params; single-section booleans via named predicates):

```rust
/// The end-of-run FFmpeg evidence (incident 2026-10-22): a clean exit
/// and its stderr tail are surfaced on the gate-timeout shutdown path
/// where the gate error outranks the (healthy) pipeline outcome, and a
/// non-zero exit gets the encoded warn line. `Ok(Recording)` is
/// unreachable on this production path (demo owns it) and stays silent.
fn log_stream_shutdown_evidence(
    stream_outcome: &Result<CompletedOutput, FramePipelineError>,
) {
    match stream_outcome {
        Ok(CompletedOutput::Stream(stream_exit)) => log_clean_stream_exit(stream_exit),
        Err(FramePipelineError::Sink(StreamingError::ExitFailure {
            exit_code,
            diagnostics,
        })) => log_failed_stream_exit(exit_code, diagnostics),
        _ => {}
    }
}

fn log_clean_stream_exit(stream_exit: &StreamExit) {
    info!(
        exit_code = ?stream_exit.exit_code(),
        "{}; captured diagnostics follow: {}",
        FFMPEG_CLEAN_EXIT_MESSAGE,
        diagnostics_text(stream_exit.stderr_tail())
    );
}

fn log_failed_stream_exit(exit_code: &Option<i32>, stderr_tail: &str) {
    warn!(
        exit_code = ?exit_code,
        "{}; captured diagnostics follow: {}",
        FFMPEG_EXIT_FAILURE_MESSAGE,
        diagnostics_text(stderr_tail)
    );
}

/// Single-section boolean rule: the warn arm's decision lives behind a
/// named predicate (a `Some(0)` status is the exactly-clean shape).
fn stream_exit_failed(exit_code: Option<i32>) -> bool {
    !matches!(exit_code, Some(0))
}

fn diagnostics_text(stderr_tail: &str) -> &str {
    if stderr_tail.is_empty() {
        FFMPEG_NO_DIAGNOSTICS_NOTE
    } else {
        stderr_tail
    }
}
```

NOTE: `log_clean_stream_exit`/`log_failed_stream_exit` return `()` and only emit events — they are not unit-tested for the trace OUTPUT (rationale §0.4); `stream_exit_failed` and `diagnostics_text` are pure and become `pub(super)` ONLY to be reachable from the new test sibling — mark the two predicates `pub(super)` (test-sibling rule), keep the two `log_*` emitters private.

### Step 2.5 — `src/app/run/gate.rs`: the tolerated final lookup at the timeout

**Edit A** — imports: add the wait re-export for test builds, `warn`/`debug`-free (only `info` used), and the models:

```rust
use tracing::info;

use crate::app::pipeline::StopFlag;
use crate::youtube::{
    wait_until_stream_active, LiveApiError, LiveApiTransport, LiveStream,
    StreamObservationRequest, WaitCanceller,
};
```
(+ in the test `use` context the cfg-split below references the crate-root test re-export added in Step 2.6.)

**Edit B** — add the two message constants after `WATCH_LINK_MESSAGE`:

```rust
/// The info event at the confirmation-gate timeout AFTER the tolerated
/// final stream lookup observed a label: the last observed streamStatus
/// is the key discriminator (never left `ready` = YouTube never
/// registered ingest). Non-secret; pinned verbatim by a unit test.
pub(super) const READINESS_TIMEOUT_STREAM_STATUS_MESSAGE: &str =
    "confirmation gate timed out; last observed streamStatus";

/// The info event when the tolerated final lookup itself yields no
/// label (lookup error, not-found, or a label-less resource): the FACT
/// is logged, never fatal. Pinned verbatim by a unit test.
pub(super) const READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE: &str =
    "confirmation gate timed out; the stream status could not be observed";
```

**Edit C** — cfg-split the confirmation wait (test builds exhaust after one zero-sleep attempt; production keeps the D-T2-9 budget byte-exact). Add next to `run_confirmation`:

```rust
/// Test-only confirmation budget: ONE attempt at a ZERO-second interval
/// exhausts without any sleep, allowing a zero-sleep ReadinessTimeout
/// test (the gate_tests zero-sleep rule). Production never sees this.
#[cfg(test)]
const TEST_CONFIRMATION_ATTEMPTS: u32 = 1;
#[cfg(test)]
const TEST_CONFIRMATION_INTERVAL_SECONDS: u64 = 0;

#[cfg(test)]
fn confirmation_budget_wait(
    transport: &dyn LiveApiTransport,
    stream_pair: &StreamObservationRequest<'_>,
) -> Result<(), LiveApiError> {
    crate::youtube::wait_until_stream_active_with_polling(
        transport,
        stream_pair,
        TEST_CONFIRMATION_ATTEMPTS,
        TEST_CONFIRMATION_INTERVAL_SECONDS,
    )
}

#[cfg(not(test))]
fn confirmation_budget_wait(
    transport: &dyn LiveApiTransport,
    stream_pair: &StreamObservationRequest<'_>,
) -> Result<(), LiveApiError> {
    wait_until_stream_active(transport, stream_pair)
}
```

**Edit D** — in `run_confirmation`, replace the direct call and extend the fatal arm. Before (line 102 and lines 110–113):

```rust
    let result = wait_until_stream_active(plan.transport.as_ref(), &observation);
```
→
```rust
    let result = confirmation_budget_wait(plan.transport.as_ref(), &observation);
```
and the fatal arm:
```rust
        Err(error) => {
            plan.stop_flag.request_stop();
            GateFinish::Fatal(error)
        }
```
→
```rust
        Err(error) => {
            if is_confirmation_timeout(&error) {
                log_last_stream_status(plan.transport.as_ref(), &plan.stream_id);
            }
            plan.stop_flag.request_stop();
            GateFinish::Fatal(error)
        }
```
(Encoding: the final lookup happens BEFORE the stop-flag flip/join path returns — the gate thread owns the transport to its end; ordering inside the arm is the plan's "one additional tolerated lookup at failure time".)

**Edit E** — add the three helpers BELOW `run_confirmation` (visibility: `observed_status_label` needs `pub(super)` for direct unit tests; the other two stay private):

```rust
/// Single-section boolean rule: the timeout-only diagnostic decision is
/// a named predicate (the gate path is the only producer of this
/// variant, and the stage argument stays enum-internal).
fn is_confirmation_timeout(error: &LiveApiError) -> bool {
    matches!(error, LiveApiError::ReadinessTimeout { .. })
}

/// ONE additional tolerated stream lookup at the timeout (plan-encoded):
/// any outcome is consumed as a fact — an observed label is named, any
/// other outcome logs the unavailable message; a Network/classified
/// failure is NEVER fatal here and never changes `GateFinish::Fatal`.
fn log_last_stream_status(transport: &dyn LiveApiTransport, stream_id: &str) {
    match transport.lookup_stream(stream_id) {
        Ok(Some(stream)) => match observed_status_label(&stream) {
            Some(label) => {
                info!(stream_status = label, "{}", READINESS_TIMEOUT_STREAM_STATUS_MESSAGE);
            }
            None => {
                info!("{}", READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE);
            }
        },
        Ok(None) => {
            info!("{}", READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE);
        }
        Err(_) => {
            info!("{}", READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE);
        }
    }
}

pub(super) fn observed_status_label(stream: &LiveStream) -> Option<&str> {
    stream
        .status
        .as_ref()
        .and_then(|status| status.stream_status.as_deref())
}
```
NOTE on `observed_status_label`: if `pub(super)` makes it unreferenced in non-test builds, rustc/clippy will flag dead code. Avoid that: gate the fn for tests OR consume it from the private call site (it IS consumed by `log_last_stream_status` in production builds — so plain `pub(super)` is exercised and no staging allow is permitted). If `cargo clippy -D warnings` still flags anything here, the FIX is `#[cfg(test)] pub(super)` — do NOT add any `#[allow]` (D12); pick the cfg variant that compiles clean.

### Step 2.6 — `src/youtube/mod.rs`: the ONE test-only crate re-export

Immediately after line 135 (`pub(crate) use readiness_wait::{wait_until_stream_active, StreamObservationRequest};`) add:

```rust
// Test-only crate export for the confirmation-gate zero-sleep timeout
// tests (production uses the D-T2-9 budget; no signature changes).
#[cfg(test)]
pub(crate) use readiness_wait::wait_until_stream_active_with_polling;
```

(No production path gains a new export; the readiness-wait machinery's signatures and its 14 tests stay byte-untouched. The `pub(super)` at `readiness_wait/mod.rs` line 19 → `pub(crate)`: change ONLY that cfg-gated re-export line.)

---

## 3. Test Plan (D7-style binding names; all offline)

### 3.1 New streaming-layer test — `src/streaming/ffmpeg_stream_tests.rs`

**New test** `stream_completion_carries_exit_code_and_redacted_stderr_tail`:
- Fake body (new `const`, Zoom-free): `STDIN_DRAIN_ECHO_EXIT_ZERO_BODY = "cat > /dev/null; echo \"Output #0, flv, to 'rtmps://ingest.example.test/ingest/stream-key-fixture'\" 1>&2; exit 0"` (echo AFTER stdin drain → the stderr lines are captured; exit 0).
- Write fake script in the `TempDir` with the existing helpers (`write_fake_executable`, `stream_config`, `start_stream_fake`), write one frame, `finish()`.
- Assertable contract:
  - `matches!(completed, CompletedOutput::Stream(_))` via `let Ok(CompletedOutput::Stream(stream_exit)) = ... else { panic!(...) };`
  - `stream_exit.exit_code() == Some(0)`
  - `stream_exit.stderr_tail().contains("[redacted]")`
  - `!stream_exit.stderr_tail().contains("rtmps://") && !stream_exit.stderr_tail().contains("stream-key-fixture")` (single-section rule: split into two `assert!` lines).

**Existing-test updates** (assertion shapes only, contracts otherwise):
- `clean_stream_exit_returns_the_stream_completion`: `matches!(completed, CompletedOutput::Stream)` → `CompletedOutput::Stream(_)`.

### 3.2 Updated streaming-stage assertion — `src/app/run/streaming_stage_tests.rs`

- Line 55: `assert!(matches!(completed, CompletedOutput::Stream));` → `CompletedOutput::Stream(_)`. Nothing else changes (the argv pins stay).

### 3.3 New test file — `src/app/run/streaming_diagnostics_tests.rs`

Declare it in `src/app/run/streaming.rs` after the existing test module:

```rust
#[cfg(test)]
#[path = "streaming_diagnostics_tests.rs"]
mod streaming_diagnostics_tests;
```

File content (header doc note reporting the constants are the single source of truth for the two shutdown evidence lines):

- `ffmpeg_exit_messages_are_pinned`:
  - `assert_eq!(super::streaming::FFMPEG_CLEAN_EXIT_MESSAGE, "FFmpeg streaming process exited cleanly (exit status 0)");` — wait, the test file IS a child of `streaming`, so `super::*` reaches the constants directly (the `Access path note` precedent in streaming/mod.rs). Use `use super::*;` and assert the three constants verbatim (`FFMPEG_CLEAN_EXIT_MESSAGE`, `FFMPEG_EXIT_FAILURE_MESSAGE`, `FFMPEG_NO_DIAGNOSTICS_NOTE = "none captured"`).
- `stream_exit_failed_classifies_zero_status_and_signal_death`:
  - `assert!(!stream_exit_failed(Some(0)));`
  - `assert!(stream_exit_failed(Some(1)));`
  - `assert!(stream_exit_failed(None));`
- `an_empty_tail_reads_the_none_captured_note`:
  - `assert_eq!(diagnostics_text(""), FFMPEG_NO_DIAGNOSTICS_NOTE);`
  - `assert_eq!(diagnostics_text("ERROR"), "ERROR");`

### 3.4 New gate tests — `src/app/run/gate_tests.rs`

Need a `ReadinessTimeout`-scripted gate: `script_gate_lookup` supports repeated `push` calls on `lookup_stream_results` (the `ScriptedTransport` records every op). With the Step 2.5 test budget (1 attempt / 0 s), the timeout path needs: attempt 1 lookup → non-active → exhausted → final lookup.

- **New test** `confirmation_timeout_performs_exactly_one_final_status_lookup`:
  - `transport.script_gate_lookup(Ok(Some(stream_with_status("ready"))));` then push ONE more: `transport.script_gate_lookup(Ok(Some(stream_with_status("ready"))));` — Mark: `script_gate_lookup` pushes exactly one outcome; call it twice.
  - `spawn_startup_gate(gate_plan(...))` → `finish()` → assert `matches!(finish, GateFinish::Fatal(LiveApiError::ReadinessTimeout { .. }))` and `transport.operations() == ["lookup_stream", "lookup_stream"]` (exactly one EXTRA tolerated lookup) and the plan's stop flag flipped (`stop_flag.is_requested()` — build the plan with a fresh `StopFlag` clone like the existing fatal test does).
  - (The LOGGED label itself is pinned via the constant in 3.5; the ledger + `observed_status_label` test pin the wiring without a tracing capture.)
- **New test** `confirmation_timeout_tolerates_a_failing_final_status_lookup`:
  - First lookup `Ok(Some(stream_with_status("ready")))` (consumes the attempt), second lookup `Err(LiveApiError::Network { reason: "final-lookup-fixture".to_string() })` — hmm check LiveApiError::Network shape is `{ reason: String }` ✓ (live_error/mod.rs line 58).
  - Assert: still `GateFinish::Fatal(LiveApiError::ReadinessTimeout { .. })` (the tolerated failure never becomes the reported variant), and `transport.operations() == ["lookup_stream", "lookup_stream"]`.
- **New test** `non_timeout_gate_fatal_performs_no_extra_status_lookup`:
  - `transport.script_gate_lookup(Ok(Some(stream_with_status("error"))));` (immediate `BroadcastState` fatal).
  - Assert `matches!(finish, GateFinish::Fatal(LiveApiError::BroadcastState { .. }))` and `transport.operations() == ["lookup_stream"]` — ONE lookup total: no extra tolerated lookup outside the timeout shape.
- **New test** `observed_status_label_reads_the_raw_stream_status_label`:
  - `assert_eq!(observed_status_label(&stream_with_status("ready")), Some("ready"));`
  - `assert_eq!(observed_status_label(&inserted_stream()), None);` — `inserted_stream()` (the status-less fixture) is currently PRIVATE in `runtime_test_fixtures.rs` (line 145, `fn inserted_stream()`): change it to `pub(super)` (fixture sibling requires it; it is cfg(test)-only, production untouched).
- The pre-existing 3 gate tests keep their bodies (the cfg-split budget changes nothing for them: first-attempt resolution still holds; `pre_requested_cancel...` still records ZERO lookups).

### 3.5 Constant pins extended — `src/app/run/mod_tests.rs`

New test `gate_timeout_stream_status_messages_are_pinned`:
```rust
assert_eq!(
    READINESS_TIMEOUT_STREAM_STATUS_MESSAGE,
    "confirmation gate timed out; last observed streamStatus"
);
assert_eq!(
    READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE,
    "confirmation gate timed out; the stream status could not be observed"
);
```
(import `super::gate::{WATCH_LINK_MESSAGE, READINESS_TIMEOUT_STREAM_STATUS_MESSAGE, READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE}` — extend the existing use line.)

### 3.6 Explicitly NOT added (scope discipline)

- No tracing-output capture tests (rationale §0.4); no `frame-write-mid-loop stderr rescue` (that path cannot join the drain threads — documented residual limitation in §9); no changes inside `readiness_wait` test rosters; no config/example changes; no `sign`/`version` changes beyond what step 3 already bumped.

---

## 4. Byte-Stability Checklist (verify line-for-line before commit)

1. `src/app/run/error.rs` — zero edits; all 8 `RunError` Display lines unchanged (`run_error_display_lines_are_stable_and_secret_free` must pass untouched).
2. `src/youtube/live_error/**` — zero edits (ReadinessTimeout keeps its exact Display/fields; no field additions anywhere).
3. Single `error:` line: `report_run_failure` (`app/run/mod.rs`) untouched; the new lines are `info!`/`warn!` only.
4. `RUN_STOP_MESSAGE` / `RUN_STARTUP_STOP_MESSAGE` / `RUN_SUCCESS_EXIT` / exit mapping — untouched (`stop_messages_and_success_exit_are_pinned` passes).
5. `WATCH_LINK_MESSAGE` + watch-line emission in the gate Ok arm — untouched (`watch_link_message_and_url_are_pinned` passes).
6. `StreamingError` Display — zero edits; `ExitFailure`'s `"FFmpeg exited with failure status …; captured diagnostics follow: …"` unchanged.
7. Demo: `Recording` variant semantics + `print_final_path` demo line byte-stable (`app/demo/mod_tests.rs:66` untouched).
8. Shutdown ORDER unchanged: stop flag flip (gate thread) → (streaming) pipeline run → cancel-if-err → gate join → primary selector. The ONLY insertion is the logging call between `pipeline.run()` and the cancel check (pure emission, no state change) — the `shutdown_order_cancels_the_wait_before_joining_the_gate` runtime test must pass untouched.
9. Redaction guarantees: destination URL → `[redacted]` at CAPTURE; the streamStatus label and the status-label field names are non-secret; no line ever prints the destination URL, stream key, `rtmps://`, tokens, or `Bearer`. Deliberate `SpawnPlan`-style non-derivation: keep NO `Display` on `StreamExit`.
10. Zero `Cargo.toml` dependency lines change (Cargo.lock untouched); zero new `#[allow(...)]` attributes; zero commented-out code.

---

## 5. Command Inventory (§5-style, Alpine-VM Compose pattern; exit code is authoritative)

Run from the Compose working directory `/rust-youtube-streamer` inside the container:

1. Targeted dev-loop spot checks (after each code step):
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked -- streaming::ffmpeg_stream_tests streaming_stage_tests`
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked -- gate_tests streaming_diagnostics_tests`
2. Full gate (the standard dev-checks sieve):
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
   - Record: exit 0, per-check PASS lines, summary `ALL CHECKS PASSED`, the observed test count, and the gitignored log path (`logs/checks/<UTC>.log`). Baseline before this task: 360 passed (v0.9.1 tree; tree has no test-count changes since step 3). Expected after: ≥ 368 passed / 0 failed (8 new-or-updated tests re-verified; some updated assertions replace old ones, so record the OBSERVED count, never an estimate).
3. Release artifacts (both, then cross-check sizes with `ls -l`):
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh`
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh`
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows`
   - Record both byte sizes (`ls -l`-cross-checked); artifacts are gitignored (`dist/` → `.gitignore:34`), never committed.
4. Hygiene/secret audit (host, Windows git):
   - `git status --porcelain` — exactly the touched task paths (and no `target/`, no `dist/`, no credential material).
   - `git check-ignore -v dist/rust-youtube-streamer-service dist/windows/rust-youtube-streamer-service.exe` → `.gitignore:34:dist/`.
   - `git ls-files dist/ videos/ credentials/` → empty except the pre-existing `credentials/.gitkeep`.
   - `git grep -n -F "rtmps://" -- src/` → 0 hits in new code (pre-existing loopback-test URL text in `ffmpeg_stream_tests.rs`/`streaming_stage_tests.rs` fixtures is the known exempt set: `rtmps://ingest.example.test/...` fixture strings; verify nothing NEW was added).
   - `git grep -n -F "ya29" -- src/ ; git grep -n -F "AIza" -- src/` → 0.
   - `git grep -n -F "allow(dead_code)" -- src/` → the pre-existing 8 attribute lines (+1 comment mention); `git grep -n -F "allow(unused_imports)" -- src/` → 0.
5. Formatting/clippy: covered INSIDE the gate (`fmt-check` with `--check`, `clippy --locked -- -D warnings`); never `cargo fmt` on auto-apply outside the gate, never editing format by hand.

## 6. Git Handling (implementer; repo commit style — lowercase `type: summary`)

1. Pre-step (if not already tracked): commit THIS plan file on the feature branch — `chore: track run-diagnostics Task 1 implementation plan`.
2. Code+tests commit (after the full gate is green): `feat: surface FFmpeg exit and stderr diagnostics on the run shutdown path` — includes Steps 2.1–2.6 and all §3 test edits.
3. Any 4.3 review/simplify fix commits follow the reviewer's guidance (`refactor: …`/`test: …`).
4. The 4.4 docs-specialist commits docs/metadata separately (`docs: ...`).
5. NO push, NO merge, NO TODO/checkbox/DONE edits in 4.2 — those belong to 4.6/workflow step 5.
6. `.gitignore` compliance is the standing rule: run `git status` before commit; stage ONLY intended files; never force-add ignored paths; never commit `target/`, `dist/`, `videos/`, real `config.json`, credential files, or log files.

## 7. Metadata Refreshes (after code; owners per workflow cycle)

- `.agent/project-structure.md` (docs-specialist at 4.4): update the `app/run` maps — `gate.rs` gains the timeout final-lookup helpers + the two status constants; `streaming.rs` gains the two diagnostics constants + evidence helpers; streaming map: `sink.rs` gains `StreamExit` (the `Stream(StreamExit)` carrier), `ffmpeg.rs` gains the capture-time redacted tail; `youtube/mod.rs` gains the `#[cfg(test)]` polling re-export line; bump the module-file counts and the test rosters (the new `streaming_diagnostics_tests.rs` file must appear).
- `.agent/project-info/tech.md` (docs-specialist): one truthful note — the run shutdown now surfaces FFmpeg's exit status + destination-redacted stderr tail and the last observed streamStatus at gate timeout via info/warn tracing lines (streaming boundary keeps std+config-only deps; no dependency change).
- `.agent/project-info/context.md` (docs-specialist at 4.4; planner/architector maintain milestoned records): record this task's implementation facts with the observed gate count and artifact sizes; NO real-Google verification claims.

### 7.1 Docs handoff for 4.4 — `docs/production-run.md` content spec (docs-specialist owns the file edit; implementer does NOT touch docs)

Update "What the Logs Look Like (Without Secrets)" table — add AFTER row 3 (`FFmpeg started …`):

| Order | Line / event | Notes |
| --- | --- | --- |
| 3a | `stream confirmed active on YouTube` / `public watch link …` | unchanged rows 4–5 above; keep their numbering honest after renumbering |
| 4 | (end of run) `FFmpeg streaming process exited cleanly (exit status 0)` | info; `exit_code` field; carries `captured diagnostics follow: <redacted tail or "none captured">` |
| 4 | (exit failure) `FFmpeg streaming process exited with a failure status` | **warn**; `exit_code` field (or omitted for signal death); the redacted tail follows in the same event |

And in the confirmation-timeout context:

| Order | Line / event | Notes |
| --- | --- | --- |
| 5 | `confirmation gate timed out; last observed streamStatus` | info; the label is the raw documented `status.streamStatus` value |
| 5 | `confirmation gate timed out; the stream status could not be observed` | info; the final lookup failed/was empty (network problem or a label-less resource) |

Operator diagnosis procedure (add to the "Readiness timeouts" bullet in "Common ingestion/API failures", or as a short sub-block right there):

1. On `confirmation gate timed out; last observed streamStatus stream_status="ready"` — YouTube never registered the ingest: the encode side is healthy (see 2), investigate the YouTube side — check Studio (made-for-kids channel setting, stream routing, channel live access, `docs/google-cloud-prerequisites.md` eligibility), delete the orphaned `ready` broadcast + liveStream pair in YouTube Studio (the "Broadcast/Stream Fate" section above) before re-running.
2. On `…streamStatus` naming another label (`inactive`, `error`, `created`) — treat per the documented liveStreams status semantics; `error` label points at the stream's own diagnostics.
3. On `…the stream status could not be observed` — the final lookup itself failed (network/API): resolve connectivity, delete stale Studio resources, re-run.
4. The FFmpeg exit/exit-tail lines close the encode-side question: a clean exit (exit status 0) with progress-line stderr tail means FFmpeg encoded and streamed frames for the full budget; a `warn` failure-status line names a broken piped/encode process — its tail's last lines name the underlying cause, with the RTMPS URL pre-redacted.
5. Studio cleanup reminder: each failed run leaves an orphaned `ready` pair per the fate policy — delete before re-running, or the account eventually hits `userBroadcastsExceedLimit`.
6. Honest limits note for the docs: the mid-loop broken-pipe shape (`writing a frame to the FFmpeg process failed: broken pipe …`) still cannot print the stderr tail (the drain threads are never joined in that path) — the single `error:` line remains the evidence surface there; never claim otherwise.

## 8. Encoded Constraints (binding; carries the global plan + repo rules)

- **NO new dependencies** — `Cargo.toml`/`Cargo.lock` untouched (only version, already bumped).
- **NO `#[allow(...)]` additions** (D12 style; staging is cfg-variant only, per Step 2.5 NOTE).
- **NO commented-out code**; self-documenting names; minimal comments only where the contract is non-obvious.
- **Private-by-default**: `StreamExit` fields private; accessors/methods/helpers at the narrowest visibility that compiles (`pub(crate)` accessors, `pub(super)` constructor/tests-needed fns).
- **Single-section boolean conditions** in every `if`/`match`/guard decision (helpers provided).
- **Max 2 args/function**; bodies ≤ 50 lines; files ≤ 200 lines.
- **Non-zero FFmpeg exit ⇒ warn** (signal death included); clean exit ⇒ info; the gate-timeout lookup facts ⇒ info.
- **NEVER log the destination URL or stream key**; no RTMPS/token/Bearer text ever in the new lines.
- **Diagnostics are NEVER RunError variants** — `error.rs` stays shape-stable; no variant/field additions.
- **Do NOT change** the readiness-wait machinery's signatures, its 14 tests, `LiveApiError` variants/Display, the shared `Sleep`/poll constants in production, the shutdown order, exit codes, the demo finalize line, or `RUN_*`/`WATCH_LINK_MESSAGE`/pinned Display lines.
- **Offline tests only** — fakes or `127.0.0.1` loopback; never Google.
- **`demo`/`auth` byte-stable**; no front-end surface exists.
- **Fix line-width by rustfmt** (the gate's `fmt-check`), not by hand.

## 9. Residual Limitations to Record Honestly (docs + completion-report inputs)

1. A mid-loop broken-pipe frame write still drops the (never-joined) stderr evidence — the single `error:` line names the failure; surfacing the tail there is future work (bigger rerouting of the write/finish path).
2. `Ok(Stream)` exit_code is observed `Some(0)` in practice, but the code carries `status.code()` verbatim — nothing assumes the value.
3. The last observed streamStatus logged is the FINAL tolerated lookup's label at failure time, not a history — the docs say exactly that (never claim "the last poll of the 24 attempts' loop observed X" — the loop itself still discards per-attempt history by design; the failure-shape deduction is made from the FINAL lookup).
4. Whole-task framing: evidence-gathering for the NEXT run's diagnosis; NOT a root-cause fix (global-plan risk #1 wording).

## 10. Stop-and-Report Conditions (return the question to the caller)

- The `git grep`ed pre-existing line-cap arithmetic (§0.5) fails at implementation time (a file exceeds 200 lines after rustfmt) → STOP, report actual counts, propose the named sibling split, do not improvise one.
- `stream_exit_failed`/`observed_status_label` visibility produces a `-D warnings` dead-code/clippy error not covered by this plan's cfg-note → STOP and report (no ad-hoc allows).
- Any existing pinned test fails after the edits → STOP; do not modify pinned text; re-run the checklist section 4 to locate the contract break.
- Any developer-observed need for a `Cargo.toml`, `.gitignore`, or CLI change → STOP (out of this step's scope entirely).

## 11. Acceptance Cross-Check vs TODO Task 1 (this plan covers ALL THREE bullets)

1. "During a run's shutdown, surface FFmpeg's exit status and the captured, destination-redacted stderr tail as info/warn log lines through the run-mode reporting path — including on the gate-timeout shutdown path" → Steps 2.3 (redaction at capture), 2.4 (the carrier consumed by app/run/streaming.rs; warn non-zero, info clean; emission precedes the gate join so the evidence survives the gate-outranking path). §3.1/§3.2/§3.3 pin the carrier and constants.
2. "When the confirmation gate times out, perform one tolerated final stream lookup and log the last observed streamStatus label (or the lookup-failure fact)" → Step 2.5 (exact gate.rs shape); §3.4 pins the ledger (exactly-one-extra lookup on timeout; none on other fatals; tolerance of failure).
3. "Pin the new behaviors with offline tests; keep every pinned existing line/exit contract intact; run the standard dev-checks gate; rebuild both release artifacts with `ls -l` sizes; hygiene/secret audit" → §3 roster + §4 checklist + §5 inventory + §6 git handling.

The made-for-kids content (TODO Task 2) has NO step in this plan — deliberately omitted per the sub-agent boundary.

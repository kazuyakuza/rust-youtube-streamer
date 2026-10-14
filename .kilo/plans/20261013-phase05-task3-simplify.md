# Phase 05 — Task 3 Code-Simplification Plan (sub-step 4.3, simplifier side)

- Date: 2026-10-14
- Input diff reviewed: `git diff a37cb4f..20905a3` (Task 3 landed surface: `src/app/run/`,
  `src/streaming/`, `src/app/pipeline/`, `src/app/{mod,cli}`, `src/app/demo/`, `src/youtube/`)
- Authorities (binding, re-read in full): `.agent/todos/20261013/20261013-todo-1.md` (Task 3 +
  global Scope/Boundaries + Explicitly-Out-of-Scope), global plan
  `.kilo/plans/20261013-phase05-production-youtube-live-runtime.md` (D-G1..D-G11), Task-3 plan
  `.kilo/plans/20261013-phase05-task3-run-orchestration.md` (D-T3-1..D-T3-15)
- Repo state at review: branch `feat/phase05-production-live-runtime`, HEAD `20905a3`, gate green
  `logs/checks/20261014T194332Z.log` — OBSERVED test total **355**, `#[allow(dead_code)]`
  attribute inventory = 8 lines (+1 doc-comment mention in `src/youtube/mod.rs` line 68)
- Scope judgment: **NOT too extensive.** Four contained items, all single-file-local, all
  behavior-identical, zero architectural change, zero test churn. Executable by the implementer
  in one green commit.

## 0. Why these items are rule violations, not taste changes

The D-T3-14 binding decision applies the project code rules to every new/edited Task-3 file,
and D-T3-15 explicitly records: "The ≤2-args rule itself is NOT relaxed; this is a named,
recorded exception for four pre-existing signatures" — the four are
`ReadinessWait::new`, `ReadinessWait::with_polling`, `wait_until_stream_active_with_polling`,
`confirmation_poll` (all `src/youtube/readiness_wait/`). Three NEW production functions landed
outside that list with more than two parameters. All three are private, single-file-local, and
have no test references (verified by grep: `join_gate`, `spawn_with`, `report_exit_failure`
appear only inside their defining files), so the fixes are fully mechanical.

Adjudication note (do NOT "fix" these): test-only multi-arg helpers —
`run_with_scripted` (3 args, `runtime_tests.rs`), `service` (4 args, `credentials_tests.rs`) —
stand under the recorded D-T3-15(b) rationale "production argument-weight rules target the
production surface". Changing them adds boilerplate and churns test fakes; prohibited.

---

## 1. Item S1 — inline the 3-arg `join_gate` helper (`src/app/run/streaming.rs`)

ANCHOR RULE (applies to every edit below): all quoted line numbers refer to the files as they
stand at HEAD `20905a3`, BEFORE any of these edits. The verbatim text blocks are the ONLY
authoritative anchors — locate each edit by exact text match, never by raw line number after a
previous edit has shifted the file. Apply S1 → S2 → S3 → S4; every anchor stays unique and
findable in that order (S2c inserts at the text "before the `impl FfmpegProcess` block's
closing `}`", which S3's literal edits do not invalidate).

`join_gate(canceller, gate, stream_outcome)` has 3 production params (rule: ≤2). Inline its
3-line body at the single call site. Ordering is preserved EXACTLY (D-T3-6 rule 2: cancel
BEFORE join on a fatal; join otherwise).

**Edit 1a — replace lines 74–77** (inside `impl StreamingRunner for PreparedStreaming::run`,
after `prepared_stream_pipeline` construction):

```rust
        info!("FFmpeg started toward the configured streaming destination");
        let stream_outcome = pipeline.run().map(|_completed| ());
        let gate_finish = join_gate(&canceller, gate, &stream_outcome);
        primary_shutdown_outcome(stream_outcome, gate_finish)
```

verbatim with:

```rust
        info!("FFmpeg started toward the configured streaming destination");
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

**Edit 1b — delete the whole `join_gate` function, lines 81–93, plus its single blank
separator line 80** (doc comment, the 5-line signature, the body, closing brace):

```rust
/// D-T3-6 rule 2: on a frame-pipeline FATAL the confirmation wait is
/// cancelled BEFORE the gate join, so a mid-interval wait ends on its next
/// attempt check instead of running to its budget.
fn join_gate(
    canceller: &WaitCanceller,
    gate: SpawnedGate,
    stream_outcome: &Result<(), FramePipelineError>,
) -> GateFinish {
    if stream_outcome.is_err() {
        canceller.request_cancel();
    }
    gate.finish()
}
```

**Do NOT touch the imports.** `WaitCanceller` stays used by `StreamingExecution` (field type)
and `FramePipelineError` stays used by `primary_shutdown_outcome`.

Expected: file 112 → 104 total lines. `run` body stays ≤50 lines, depth ≤2, condition is a
single section (`stream_outcome.is_err()`). `runtime_runner_fixture.rs` (which mirrors the
sequence inline) needs NO change — after this edit the mirror is literal.

## 2. Item S2 — `report_exit_failure` becomes a method (`src/streaming/ffmpeg.rs`)

Task 3 widened the base 2-arg function to 3 production params by threading the redaction
target. The target is already an `FfmpegProcess` field; read it through `&self` instead of a
third parameter. Behavior-identical (same inputs, same laziness: nothing is built on a
successful exit).

**Edit 2a — in `impl FrameSink for FfmpegProcess::finish` (line 143), replace:**

```rust
        report_exit_failure(&status, &captured_stderr, self.redaction_target.as_deref())?;
```

verbatim with:

```rust
        self.report_exit_failure(&status, &captured_stderr)?;
```

**Edit 2b — delete the whole free function (lines 151–163) plus its single blank separator
line (line 150):**

```rust
fn report_exit_failure(
    status: &ExitStatus,
    stderr: &CapturedPipe,
    redaction_target: Option<&str>,
) -> Result<(), StreamingError> {
    if status.success() {
        return Ok(());
    }
    Err(StreamingError::ExitFailure {
        exit_code: status.code(),
        diagnostics: exit_diagnostics(stderr, redaction_target),
    })
}
```

**Edit 2c — add this private method at the END of the `impl FfmpegProcess { ... }` block**
(right after the `start_stream` method's closing `}`, before the block-closing `}`), verbatim:

```rust

    /// Non-zero exit maps to the typed failure whose diagnostics carry the
    /// captured stderr tail with the destination redacted FIRST (D-T1-11).
    fn report_exit_failure(
        &self,
        status: &ExitStatus,
        stderr: &CapturedPipe,
    ) -> Result<(), StreamingError> {
        if status.success() {
            return Ok(());
        }
        Err(StreamingError::ExitFailure {
            exit_code: status.code(),
            diagnostics: exit_diagnostics(stderr, self.redaction_target.as_deref()),
        })
    }
```

PROHIBITED: changing the `exit_diagnostics(stderr, target)` argument order or the redaction
pass inside `pipe_drain.rs` (secret safety, never weaken). `pipe_drain.rs` and
`local_finalize.rs` get NO edits at all.

## 3. Item S3 — `spawn_with` gets a `SpawnPlan` parameter object (`src/streaming/ffmpeg.rs`)

`spawn_with(config, arguments, local_output, redaction_target)` has 4 production params.
Bundle the three spawn inputs into a private struct (the project's recorded param-object
pattern: `GatePlan`, `StageRuntime`, `StreamParts`). Behavior-identical: same argv, same
fields, same order.

**Edit 3a — inside `start`, replace:**

```rust
        let arguments = args::build_arguments(config, &output_path)?;
        spawn_with(config, arguments, Some(output_path), None)
```

verbatim with:

```rust
        let arguments = args::build_arguments(config, &output_path)?;
        let plan = SpawnPlan {
            arguments,
            local_output: Some(output_path),
            redaction_target: None,
        };
        spawn_with(config, plan)
```

**Edit 3b — inside `start_stream`, replace:**

```rust
        let arguments = args::build_stream_arguments(config, &destination)?;
        spawn_with(
            config,
            arguments,
            None,
            Some(destination.as_url().to_string()),
        )
```

verbatim with:

```rust
        let arguments = args::build_stream_arguments(config, &destination)?;
        let plan = SpawnPlan {
            arguments,
            local_output: None,
            redaction_target: Some(destination.as_url().to_string()),
        };
        spawn_with(config, plan)
```

**Edit 3c — replace the region of lines 73–80 (the old 2-line `spawn_with` doc comment PLUS
the 6-line signature — from `/// The single spawn path shared by the local and` through
`) -> Result<FfmpegProcess, StreamingError> {`) verbatim with:**

```rust
/// The shared spawn inputs (≤2-args rule via parameter object): the locked
/// vector plus exactly one target shape — a local file to verify, or a
/// destination URL to redact. No `Debug`/`Display` may ever be derived:
/// `redaction_target` IS the secret.
struct SpawnPlan {
    arguments: Vec<String>,
    local_output: Option<PathBuf>,
    redaction_target: Option<String>,
}

/// The single spawn path shared by the local and streaming starts: piped
/// stdin/stdout/stderr, no shell, drain threads attached before returning.
fn spawn_with(config: &AppConfig, plan: SpawnPlan) -> Result<FfmpegProcess, StreamingError> {
    let SpawnPlan { arguments, local_output, redaction_target } = plan;
```

(The old `spawn_with` doc comment is REPLACED — it must NOT end up sitting above the
`SpawnPlan` struct; the block above already carries it in its new position.)
The rest of `spawn_with`'s body (from `let mut command = Command::new(&config.ffmpeg.executable);`
down through the closing `}`) stays byte-unchanged.

Line math: net **+12** for S2+S3 combined (itemized: `start` literals +4, `start_stream`
literals 0, method added into the impl +16, deleted free function −14, `spawn_with` region
8 lines → 14 lines +6). File 185 → **≈197 total lines; MUST stay ≤200** (hard cap). Do NOT
"trim" by shortening unrelated docs; if the observed count exceeds 200, STOP and return the
count to the caller (planner decision).

## 4. Item S4 — mis-owned test re-export comment (`src/youtube/mod.rs`)

The 3-line "auth flow's offline test fakes hand-build `TokenResponse`..." comment (lines
140–142) now sits directly ABOVE the `live_models` re-export block whose consumers are the
run-runtime fixtures — the comment mis-describes what it heads. Purely a doc-placement fix;
`#[cfg(test)]` import ordering carries zero behavior.

**Edit 4 — replace lines 140–150**

```rust
// Test-only re-exports: the auth flow's offline test fakes hand-build
// `TokenResponse` values and implement `TokenEndpointTransport`, which
// requires naming these types; production code never uses them.
#[cfg(test)]
pub(crate) use live_models::{
    BindCommand, CreateBroadcastCommand, CreateStreamCommand, LiveBroadcast, LiveStream,
};
#[cfg(test)]
pub(crate) use token::{GrantedScopes, RefreshToken};
#[cfg(test)]
pub(crate) use transport::RefreshRequest;
```

verbatim with:

```rust
// Test-only re-exports: the auth-flow and run-runtime offline fakes
// hand-build token/scope values and token-endpoint requests, which
// requires naming these types; production code never uses them.
#[cfg(test)]
pub(crate) use token::{GrantedScopes, RefreshToken};
#[cfg(test)]
pub(crate) use transport::RefreshRequest;
// Test-only re-exports: the run runtime's offline transport fake scripts
// live resources through these models; production code never uses them.
#[cfg(test)]
pub(crate) use live_models::{
    BindCommand, CreateBroadcastCommand, CreateStreamCommand, LiveBroadcast, LiveStream,
};
```

The second block (current lines 151–159, the "run runtime's offline fakes implement the
transport/verifier seams" comment + `VerificationError` / `TokenExchangeResult` /
`VerifiedAuthorization`) stays byte-unchanged.

---

## 5. Binding prohibitions (whole plan)

- Behavior-identical ONLY: startup ordering (D-T3-2), shutdown policy (D-T3-6), the ONE
  destination reveal point (D-T2-12/D-T3-2 stage 6), redaction pass and its order
  (note-append THEN redact, `pipe_drain.rs` untouched), exit-code contract (D-T3-8), the
  stdout messages and constants (§7 table), tracing field whitelist (D-T3-7).
- NEVER weaken secret safety: no `Debug`/`Display` derive on `SpawnPlan` or `FfmpegProcess`;
  no new printing/formatting of `redaction_target`, `as_url()`, or `into_url()`; the only
  `into_url()` production call site stays the single reveal expression in `run/mod.rs`.
- NO Live Chat artifact; `src/app/pipeline/source.rs` and `ChatFreeSource` stay byte-unchanged.
- NO `auth`/`demo` behavior or exit-code change; `args.rs` locked vectors (local AND stream)
  byte-unchanged; `prepared_pipeline`/`PipelineConstruction`/`OutputKind` byte-unchanged;
  `live_models`/`live_lifecycle` V2 re-stage allows and their comments byte-unchanged.
- NO test added, removed, renamed, or re-asserted: total MUST stay EXACTLY 355.
- NO new `#[allow(dead_code)]` anywhere; the inventory stays at 8 attribute lines.
- NO Cargo.toml/Cargo.lock change, no new dependency, no docs/README/project-structure edits
  (no file is added/deleted here, so the structure map stays accurate).
- NO commented-out code; ≤200 lines/file, ≤50-line bodies, depth ≤2, ≤2 args after the edits.
- Feature-branch commit only — NO push, NO merge, NO version bump (D-G11).

## 6. Execution order + verification (implementer; Alpine VM MCP; `vm_status` FIRST; every
cargo step via the Compose pattern; NEVER chained commands)

1. `git status --porcelain` (must be empty) and `git branch --show-current`
   (`feat/phase05-production-live-runtime`, HEAD `20905a3`).
2. Apply S1 → S2 → S3 → S4 exactly as drafted.
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo fmt"`
   then `... sh -c "cargo fmt --check"` (fmt-fix ONLY mechanical per plan §10; never
   restructure while formatting).
4. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo clippy --locked -- -D warnings"`
   — must be clean; unexpected lints → STOP, return compiler output to the caller (no
   improvisation).
5. Spot suites: `... sh -c "cargo test --locked streaming"` and
   `... sh -c "cargo test --locked app::run"` — green, same counts as today.
6. FULL gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
   — green; record `logs/checks/<UTC>.log`; OBSERVED total MUST be exactly **355** (the gate
   log is authoritative).
7. Line/shape audit: `wc -l src/streaming/ffmpeg.rs` → ≤200 (expected ≈197);
   `wc -l src/app/run/streaming.rs` → expected ≈104;
   `git grep -n "join_gate" src/` → 0 hits;
   `git grep -n "#\[allow(dead_code)\]" src/` → same 8 attribute lines + the youtube/mod.rs
   comment mention, no new lines;
   `git grep -n "redaction_target" src/streaming/` → hits only in `ffmpeg.rs`/`pipe_drain.rs`
   (unchanged surfaces confirmed).
8. Hygiene per gitignore-compliance: `git status --porcelain` shows ONLY the 3 edited files
   (`src/app/run/streaming.rs`, `src/streaming/ffmpeg.rs`, `src/youtube/mod.rs`) + the gate
   log; no `.gitignore`-matching file staged.
9. Commit (single commit, feature branch):
   `refactor: bring the task-3 spawn and gate helpers within the two-args rule`
   Body must note: behavior-identical rule-compliance simplification of the landed Task-3
   diff (`a37cb4f..20905a3`); test total unchanged at 355.

## 7. Findings reviewed and DELIBERATELY left unchanged (record for the reviewers)

- `args.rs` input-segment duplication between the two builders — mandated byte-identical
  locked vectors (D-T3-5 F-S2); dedup is PROHIBITED by the demo-never-regresses contract.
- `prepared_stream_pipeline` assembly duplication vs `prepared_pipeline` — mandated
  "WITHOUT touching `prepared_pipeline`" (D-T3-10).
- `read_client_config` duplicate of `auth/flow.rs`'s helper — sanctioned
  duplication-over-cross-module-contract precedent (plan A3).
- Test-only multi-arg helpers (`run_with_scripted`, `service`) — recorded D-T3-15(b)
  adjudication (see §0).
- `StreamingExecution.stream_id` (production-unused, `_stream_id`) — test-pinned reveal
  contract (TD-1); removing it churns tests.
- `pipe_drain.rs`/`local_finalize.rs` module split itself — already the line-cap simplification
  (commit `06cd933`); shapes are clean, no further extraction.
- `gate.rs` `finish()` match, `runtime_runner_fixture.rs` `gate_label` strings, the per-file
  fake-FFmpeg script helpers (ETXTBSY replication) — readability-neutral or sanctioned; churn
  exceeds benefit.
- The 8-line `allow(dead_code)` inventory including the two V2 re-staged module allows —
  "retire when read" contracts, byte-unchanged (task-mandated).

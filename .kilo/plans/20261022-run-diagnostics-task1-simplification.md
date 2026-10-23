# Task-1 Simplification Plan — Run-Path Streaming Failure Diagnostics (4.3 SIMPLIFICATION half)

Plan date: 2026-10-22 · Owner: Code Simplifier (Critical Workflow step 4.3) · Executor: junior implementer (ZERO judgment calls allowed — copy the after-code verbatim)
Branch: `feat/run-streaming-diagnostics-mfk` at HEAD `59fd299`, tree clean (plus the untracked reviewer file below).
BINDING front-matter: `.kilo/plans/20261022-run-diagnostics-task1-implementation.md` (§2 shapes, §4 byte-stability checklist, §8 constraints) — every step here preserves it.
TODO: `.agent/todos/20261022/20261022-todo-2.md` Task 1 ONLY.

## Verified baseline (observed on the Alpine VM at plan time, `--locked`)

- `cargo fmt --check`: exit 0.
- `cargo test --locked -- gate_tests`: 7 passed / 0 failed.
- `cargo test --locked -- ffmpeg_stream_tests`: 4 passed / 0 failed (note: the correct filter is the plain substring `ffmpeg_stream_tests`; the module path is `streaming::ffmpeg::ffmpeg_stream_tests`, so the `streaming::ffmpeg_stream_tests` string in the implementation plan §5 matches nothing).
- Total suite: 369 tests; exactly 6 test-profile warnings, all in the adjudicated-acceptable set (`with_token_url`, `token_type` pre-Phase-05 pair + the 4 accepted run-gate items). Nothing in this plan changes those.

## Coordination with the reviewer plan (READ FIRST)

The code-reviewer half of 4.3 filed `.kilo/plans/20261022-run-diagnostics-task1-review-fixes.md` (untracked). Its **Fix A** (`gate.rs` helper extraction) is orthogonal and remains required — it is NOT repeated here. Its **Fix B** prescribes a two-function restructure of `log_last_stream_status`; step **S1 below supersedes Fix B** for that function: S1 satisfies the same max-depth constraint (no 3rd-level nest, ≤2 nesting levels) with LESS weight (one function instead of two, one emission site per message instead of two). Apply S1 INSTEAD of Fix B, never both, and never re-create Fix B's `log_observed_stream_status` helper.

## Step S1 — `src/app/run/gate_diagnostics.rs`: collapse the triple-duplicated unavailable arm

Genuine duplication: the identical `info!("{}", READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE)` statement appears in THREE arms and the inner `match` nests at depth 3 (the rule + reviewer finding). The tolerated-outcome semantics ("any outcome is consumed as a fact") are exactly "no label ⇒ unavailable", which collapses to a two-arm match.

Replace ONLY the body of `log_last_stream_status`. Before (current committed shape):

```rust
pub(super) fn log_last_stream_status(transport: &dyn LiveApiTransport, stream_id: &str) {
    match transport.lookup_stream(stream_id) {
        Ok(Some(stream)) => match observed_status_label(&stream) {
            Some(label) => {
                info!(
                    stream_status = label,
                    "{}", READINESS_TIMEOUT_STREAM_STATUS_MESSAGE
                );
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
```

After (copy verbatim):

```rust
pub(super) fn log_last_stream_status(transport: &dyn LiveApiTransport, stream_id: &str) {
    let observed_stream = transport.lookup_stream(stream_id).ok().flatten();
    match observed_stream.as_ref().and_then(observed_status_label) {
        Some(label) => {
            info!(
                stream_status = label,
                "{}", READINESS_TIMEOUT_STREAM_STATUS_MESSAGE
            );
        }
        None => {
            info!("{}", READINESS_TIMEOUT_STATUS_UNAVAILABLE_MESSAGE);
        }
    }
}
```

Rules (verify against §4 of the implementation plan; every one holds):

- Behavior-identical: EXACTLY ONE `transport.lookup_stream` call (the operations ledger the gate tests pin is unchanged — `pop` panics identically on an empty queue either way); the three tolerated outcomes (lookup error, `Ok(None)`, label-less resource) all still emit the SAME unavailable event; a label still emits the SAME status event with the same `stream_status` field. Message constants untouched.
- Pass-by-reference is required, not incidental: the owned `Option<LiveStream>` stays in the local `observed_stream` so the borrowed `label` outlives the match. Do NOT inline the `let` into the `match` scrutinee (borrow error) and do NOT switch to a closure in `and_then` (would trip `clippy::redundant_closure` under `-D warnings`) — pass the `observed_status_label` fn item directly as shown.
- No import changes: `LiveApiError` stays used by `is_confirmation_timeout`; `LiveStream` stays used by `observed_status_label`. `Result::ok`/`Option::flatten`/`Option::and_then` are stable on the pinned 1.82.
- Rule caps: 2 params, no new `if`, nesting depth ≤2, body 13 lines; file drops to ~54 lines (cap 200 holds).
- Doc comment above the function stays byte-unchanged (it already describes "any outcome is consumed as a fact").

## Step S2 — `src/streaming/ffmpeg_stream_tests.rs`: raw-string the echo fake body

Genuine readability smell: the new const uses four `\"` escapes on one long line while its sibling `URL_ECHOING_EXITER_BODY` in the same file already uses the `r#"..."#` form. The raw form carries the IDENTICAL script bytes (the body contains no `"#` sequence, so the raw terminator is unambiguous) — the fake script written to disk, and therefore every assertion, is byte-unchanged.

Before (current committed line):

```rust
const STDIN_DRAIN_ECHO_EXIT_ZERO_BODY: &str = "cat > /dev/null; echo \"Output #0, flv, to 'rtmps://ingest.example.test/ingest/stream-key-fixture'\" 1>&2; exit 0";
```

After (copy verbatim):

```rust
const STDIN_DRAIN_ECHO_EXIT_ZERO_BODY: &str = r#"cat > /dev/null; echo "Output #0, flv, to 'rtmps://ingest.example.test/ingest/stream-key-fixture'" 1>&2; exit 0"#;
```

Test names, fixture URL text and the three contains/not-contains assertions stay binding-unchanged.

## Verification (after EACH step; exit code is authoritative; MCP output may omit cargo stderr)

1. After S1 (and after the reviewer's Fix A, if applied first): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked -- gate_tests` → expect 7 passed / 0 failed.
2. After S2: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked -- ffmpeg_stream_tests` → expect 4 passed / 0 failed.
3. Final tree: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → expect exit 0, `ALL CHECKS PASSED`, OBSERVED test count recorded (expected 369 — no tests added or removed; record the observed number, never the estimate).
4. Warning audit: the `cargo test` compile output must show exactly the same 6 pre-existing test-profile warnings; any NEW warning is a stop-and-report condition.

## Git handling (implementer; repo style `type: lowercase summary`)

- Stage ONLY `src/app/run/gate_diagnostics.rs` and `src/streaming/ffmpeg_stream_tests.rs` in this simplification commit (Fix A's `gate.rs` edit belongs to the reviewer plan's own commit flow).
- Commit message: `refactor: collapse the gate timeout status arms and raw-string the stream fake body`
- `.gitignore` compliance: `git status` before commit; no `target/`, `dist/`, logs, or credential paths; NO push (workflow step 5 owns the push).

## NOT-TOUCH GUARDRAILS (binding)

Adjudicated-acceptable, leave exactly as-is: the `gate.rs`/`gate_diagnostics.rs` split (no merge-back); the whole `pub(in crate::app::run)` + `#[cfg(test)] pub(super)` visibility set; the `LiveStream` production re-export and the `#[cfg(test)] wait_until_stream_active_with_polling` re-export placement; `#[cfg(test)] pub(super) stream_exit_failed`; the 4 test-profile warn-only dead-code items; no new `allow` attributes anywhere. Pinned texts (message constants, WATCH_LINK/RUN_* lines), transmission/verification semantics, the readiness-wait machinery, shutdown order, exit mapping, demo path, `Cargo.toml/lock`, `docs/*` (4.4 owns those), Task 2 — all byte-untouched by this plan.

## Evaluated and NOT taken (no action; inputs for the caller/4.5b adherence)

- Unifying `log_clean_stream_exit`/`log_failed_stream_exit` behind `tracing::event!(Level::INFO/WARN, ...)`: fewer lines but strictly less readable and deviates from the plan §2.4 shapes — rejected.
- Collapsing `StreamExit::new`'s struct literal in `sink.rs`: rustfmt-accepted as committed; churn with zero value — rejected.
- Narrowing `observed_status_label`/timeout-constant visibility below the adjudicated set — forbidden by the caller ruling.
- `write_fake_executable` + the ETXTBSY-retry trio duplicated between `ffmpeg_tests.rs` and `ffmpeg_stream_tests.rs` (with a third shape in `streaming_stage_tests.rs`): a shared unix-only test-fixture module would dedup it, but it is a pre-Task-1 (Phase-03) cross-file restructuring — DEFERRED candidate for a future TODO (caller decision required per the TODO-file rule; nothing created here).

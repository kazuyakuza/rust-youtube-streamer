# Phase 07 Task 3 — 4.3 Simplification Plan (test-only fixture-script fold)

> Scope authority: binding plan `.kilo/plans/20261029-phase07-task3-integration-switch.md` (§B pinned bodies, §C.7 roster rows, §E scenario text), TODO Task 3, repo code rules. Target implementer: JUNIOR under 50% restriction — every edit below is closed-form; NO judgment calls, NO structural/architectural/scope decisions.
> Baseline: branch `feat/phase07-streamlist-retrieval`, HEAD `45fe2b9`, clean tree, gate **493 passed / 0 failed** (`logs/checks/20261029T192248Z.log`), clippy 0.
> This plan edits TEST code only (six `src/app/run/` files; three are `#[cfg(test)]` fixture/roster modules). ZERO production-surface change, ZERO test-name change, ZERO test-count change, ZERO assertion-semantics change; scripted values stay byte-identical. `src/youtube/` is NOT touched. NOT plan-structure-undoing: §B.4–B.7 pinned bodies, the `manual_clamp` relocation, trait shapes and the one-runtime shape are NO-EDIT; §E prescribes scenarios and semantics, not literal lines, so the roster folds are within the binding plan.

## Verdict

SIMPLIFICATION REQUIRED — exactly one genuine cross-file duplication class (review scope 1/4): the repeated fixture-build boilerplate across the rewritten rosters. Folded into three ≤2-arg (0-arg + receiver) scenario builders on the existing `ChatFakes` param object (`runtime_chat_fakes.rs`, the roster-side fixtures point §C.7 already registers). All other candidate classes adjudicated LEAVE (matrix at the end).

## The duplication (observed, exact)

- **Block X — the §E termination script** `script_session(vec![Some(Ok(yielded(&response_offline(vec![]))))])` — 7 sites: `chat_worker_tests.rs` L37–39, `chat_worker_flow_tests.rs` L119–121 and L149–151, `chat_worker_budget_tests.rs` L59–61 and L159–161, `gate_chat_tests.rs` L39–41, `runtime_tests.rs` L70–71.
- **Block Y — the empty token-bearing batch** `script_session(vec![Some(Ok(yielded(&response_with_token(vec![], RESUME_TOKEN_FIXTURE))))])` — 4 sites: `chat_worker_flow_tests.rs` L75–80, L113–118, L173–178; `chat_worker_budget_tests.rs` L106–111.
- **Block Z — the token-bearing one-message history batch** `script_session(vec![Some(Ok(yielded(&response_with_token(vec![text_item("h-1")], RESUME_TOKEN_FIXTURE))))])` — 3 sites: `chat_worker_flow_tests.rs` L21–26, L49–54, L143–148.

Inline (non-`script_session`) uses of these builders inside multi-delivery scripts (`chat_worker_tests.rs` L54–58, `chat_worker_budget_tests.rs` L177–184) are NOT folds (different composition). The two `response_offline(vec![text_item("m-1")])` sites in the flow roster (2 occurrences, parameterizable) are LEAVE — folding them would push that file's edit share over 30%.

## Step 1 — `src/app/run/runtime_chat_fakes.rs`: add the three builders

1a. Widen the `use crate::youtube::{...}` brace (currently lines 13–15) to also import `response_offline`, `response_with_token`, `text_item`, `yielded`, `RESUME_TOKEN_FIXTURE` (rustfmt will order the list — run `cargo fmt` in Step 7 and accept its order):

```rust
use crate::youtube::{
    response_offline, response_with_token, text_item, yielded, LiveApiError, LiveChatTransport,
    RESUME_TOKEN_FIXTURE, RunChatStreamLedger, ScriptedRunStreamTransport,
};
```

1b. Inside `impl ChatFakes` (after the existing `operations()` method, currently ending line 74), append EXACTLY:

```rust
    /// Queues the §E terminator session: one offline-ended empty batch, then the clean close.
    pub(crate) fn script_graceful_end_session(&self) {
        self.stream_ledger
            .script_session(vec![Some(Ok(yielded(&response_offline(vec![]))))]);
    }

    /// Queues a session delivering one empty batch carrying the resume token;
    /// the token clears the pending history and opens the live window (§A.3).
    pub(crate) fn script_token_batch_session(&self) {
        self.stream_ledger.script_session(vec![Some(Ok(yielded(
            &response_with_token(vec![], RESUME_TOKEN_FIXTURE),
        )))]);
    }

    /// Queues a session delivering the token-bearing one-message history batch —
    /// the batch the §A.3 baseline rule discards whole while history is pending.
    pub(crate) fn script_history_batch_session(&self) {
        self.stream_ledger.script_session(vec![Some(Ok(yielded(
            &response_with_token(vec![text_item("h-1")], RESUME_TOKEN_FIXTURE),
        )))]);
    }
```

Expected result: file 99 → ~120 lines (≤200). No existing line of the file changes besides the `use` brace.

## Step 2 — `src/app/run/chat_worker_tests.rs`

2a. Replace lines 37–39 (the Block X script inside `worker_offline_ended_batch_disconnects_without_terminal_error`) with one line:

```rust
    fakes.script_graceful_end_session();
```

2b. Line 11: drop `response_offline` (no longer used in this file; `response_with_token`, `yielded`, `LiveApiError` stay):

```rust
use crate::youtube::{response_with_token, yielded, LiveApiError};
```

No other line changes. The `lookup_found`/`drain`/`terminal_of` helpers, all 6 test names and all assertions stay byte-unchanged.

## Step 3 — `src/app/run/chat_worker_flow_tests.rs` (8 replacements, imports UNCHANGED)

Apply by ORIGINAL line numbers (file is 200 lines):

| Original lines | Block | Replacement |
|---|---|---|
| 21–26 | Z | `    fakes.script_history_batch_session();` |
| 49–54 | Z | `    fakes.script_history_batch_session();` |
| 75–80 | Y | `    fakes.script_token_batch_session();` |
| 113–118 | Y | `    fakes.script_token_batch_session();` |
| 119–121 | X | `    fakes.script_graceful_end_session();` |
| 143–148 | Z | `    fakes.script_history_batch_session();` |
| 149–151 | X | `    fakes.script_graceful_end_session();` |
| 173–178 | Y | `    fakes.script_token_batch_session();` |

NOT replaced (verbatim): L27–33, L55–59, L81–85, L86–91, L184–188, every assertion, every test name, the `use` block (all five fixture names + `LiveApiError` + `RESUME_TOKEN_FIXTURE` stay imported and used). Expected result: 200 → ~172 lines.

## Step 4 — `src/app/run/chat_worker_budget_tests.rs` (3 replacements, imports UNCHANGED)

| Original lines | Block | Replacement |
|---|---|---|
| 59–61 | X | `    fakes.script_graceful_end_session();` |
| 106–111 | Y | `    fakes.script_token_batch_session();` |
| 159–161 | X | `    fakes.script_graceful_end_session();` |

NOT replaced: L137 (`script_session(vec![])` — the empty stalled session), the multi-delivery script L177–184, all assertions, all 8 test names, the `use` block (still consumed by the inline sites). Expected result: 200 → ~191 lines.

## Step 5 — `src/app/run/gate_chat_tests.rs`

5a. Replace lines 39–41 (Block X) with one line:

```rust
    fakes.script_graceful_end_session();
```

5b. Line 15: `response_offline` and `yielded` become unused — replace the whole line with:

```rust
use crate::youtube::WaitCanceller;
```

## Step 6 — `src/app/run/runtime_tests.rs` (test 1 only — §C.7 edit-scope row respected)

6a. Replace lines 70–71 (Block X on the `chat` fakes object) with one line:

```rust
    chat.script_graceful_end_session();
```

6b. Lines 11–13: drop `response_offline` and `yielded` (the remaining three are still used) — replace the brace with:

```rust
use crate::youtube::{LiveApiError, ScriptedRunStreamTransport, WaitCanceller};
```

No other test (2–5) is touched.

## Step 7 — Verification (container-only, ordered; every cargo command on the Alpine VM as `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust …`)

1. `cargo fmt` then `cargo fmt --check` → 0 (rustfmt may re-wrap the new call sites; accept, do not hand-wrap).
2. `cargo test --locked chat_worker` → **22 passed / 0 failed** (matches §L.3).
3. `cargo test --locked gate_chat` → **2 passed**; `cargo test --locked runtime` → **5 passed**.
4. `cargo clippy --locked -- -D warnings` → 0 (unused-import risk lives entirely in Steps 2b/5b/6b — a leftover import is a hard error here).
5. FULL gate `sh scripts/dev-checks.sh` → exit 0, four PASS lines, `ALL CHECKS PASSED`, **493 passed / 0 failed** (behavior-identical fold: the count must NOT change).
6. Diff audit: `git status --porcelain` shows ONLY the six files of Steps 1–6.
7. STOP conditions: any targeted/full-gate count deviation, any compile error after the import trims that requires a non-listed edit ⇒ STOP and report to the caller (do not improvise).

## Rules check (per edit)

- Every file ≤200 after the fold (final: flow ~172, budget ~191, runtime_tests ~199, others smaller).
- New methods: 0 params + receiver (≤2 rule), body ≤4 lines, depth 0.
- No test renamed/added/deleted; no assertion message changed; scripted values byte-identical by construction (helper bodies are the verbatim moved lines).
- Zero new `#[allow(`; no commented-out code; names self-documenting.

## Explicitly NOT done (adjudications for the reviewer)

| Candidate | Verdict | Reason |
|---|---|---|
| §B.4–B.7 pinned bodies (`run_chat_worker`, `run_stream_loop`, `consume`, `send_stop_aware`, `into_chat_message`, `sleep_in_stop_slices`, `backoff_delay_ms` + its `manual_clamp` allow, `record_terminal`, `process_batch`/`retain_token`) | NO-EDIT | plan-pinned staged contracts (as amended by §L.2); untouched-allowed per the task's own scope |
| The repeated `stop_flag.is_requested()` checks | LEAVE | each instance sits inside a pinned body; folding would change pinned shape |
| `sleep_in_stop_slices` pub(super) with its single caller; `CHAT_STOP_TICK_SLICE_MS` pub(super) const used only in-file | LEAVE | §B.7/§B.5 LITERAL wording pins both visibilities (§L.2.1 rationale); narrowing them edits pinned text |
| `into_chat_message` optional-path branches | LEAVE | §B.7 byte-verbatim body; the id/timestamp paths are exercised by the flow-delivery pins |
| `worker_plan`+`chat_channel` 2-line pair (3× in budget, 2× in gate) | LEAVE | below fold threshold — the sites diverge (stop-flag clone follows) and each is already 1–2 lines |
| Flow-roster `response_offline(vec![text_item("m-1")])` pair | LEAVE | only 2 occurrences; folding pushes the file past the 30% edit cap |
| Worker-roster assertion epilogues (`drain`+len+contains+terminal) | LEAVE | not byte-identical (per-test failure messages differ); flattening would lose the §E message pins |
| `ClientTlsConfig`/`MetadataValue` plumbing | OUT OF SCOPE | lives in Task-2 KEEP files (`stream_client.rs`/`stream_concrete.rs`), not Task-3 landed files |
| `lookup_found` → method symmetry | LEAVE | renaming/moving the existing roster helper is churn beyond the genuine duplication |

## Phase-4-note-worthy (for the structure map / dead inventory — NOT edits here)

- Dead-code inventory is UNCHANGED by this fold (zero production surface, zero `allow(` delta); the §L.6/§L.8 observed set is what Task 4 re-cuts.
- Note for Task 4: two plan-pinned surfaces have only single-file visibility use — `sleep_in_stop_slices` (`pub(super)`, one in-module caller) and `CHAT_STOP_TICK_SLICE_MS` (`pub(super)`, file-local). Recorded in §B.7/§B.5 verbatim, so they stay as landed; if a future pass narrows them, it is its own decision, not this task's.
- Line-count claims for the structure map come from the post-fold tree at the next gate (flow ~172 / budget ~191 / runtime_tests ~199 / runtime_chat_fakes ~120 — OBSERVED values after Step 7.1, never these estimates).

## File-change budget (<30% per file, satisfied)

| File | Touched lines | Share |
|---|---|---|
| `runtime_chat_fakes.rs` | ~21 added (99 → ~120) | ~17% |
| `chat_worker_flow_tests.rs` | 48 (200 → ~172) | 24% |
| `chat_worker_budget_tests.rs` | 13 | 6.5% |
| `chat_worker_tests.rs` | 5 | 3% |
| `gate_chat_tests.rs` | 6 | 7.5% |
| `runtime_tests.rs` | 5 | 2.5% |

Commit (by the implementer, after the gate): `refactor(test): fold the repeated chat stream scripts into ChatFakes builders` — six files only, no push.

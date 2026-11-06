# Task-2 4.3 Code-Simplification Plan — reconnect cause labels

Caller step: 4.3 (Code Simplification), TASK 2 only. Scope commits: `b3eec6e` (summary split), `e78093b` (reason labels), `51c742b` (tests) on `fix/diagnostics-followup` (HEAD `51c742b`). Binding plan: `.kilo/plans/20261106-followup-task2-reconnect-reason-plan.md`.

Verdict: **REQUIRED — 2 adopted edits (S1, S2)**, both minor and local, fully specified below for the ≤50% implementer. Zero structural/scope decisions; no architect needed.

Lens honored: row bytes/keys/values/field order/skip attr, label words, lock spans, gating, test assertion values, and test names are all FROZEN. Neither edit touches any of them.

---

## 1. Audit results per lens (evidence-backed)

1. **Post-split hygiene** — CLEAN. `events.rs` (153): no leftover imports (`HistogramSummary`/`CLOCK_DOMAIN_NOTE`/`SummaryEvent`/`SummarySnapshot` all moved, zero stale refs; `git grep "diagnostics::events::" -- src` lands only on the three correct consumers). `mod.rs` (75) decl added alphabetically. `state.rs`/`diagnostics_test_fixtures.rs`/`collector.rs` (176/147/196) each carry exactly one updated import path, no duplicates. `epoch_seconds_now` widened to `pub(super)` — minimal for the `events_summary` sibling. Visibility audit: nothing wider than needed was ADDED (`SummaryEvent`/`SummarySnapshot` `pub(crate)` is the pre-split status quo the binding plan §3 explicitly froze).
2. **Test-roster duplication** — the new roster reuses the shared builders correctly (`script_token_batch_session`, `script_graceful_end_session`, `worker_plan`, `collector_with_output_file`, `RESUME_TOKEN_FIXTURE`). ONE genuine fold found: the 6-line inline token-batch session in test 1 is a verbatim copy of `runtime_chat_fakes::script_token_batch_session`'s body (identical batch content: `vec![]` + `RESUME_TOKEN_FIXTURE`) → adopted as S2. The local `script_lookup`/`network_failure` helpers vs `chat_delivery_diagnostics_tests.rs` are REJECTED (see §3).
3. **Over-complex constructs** — one needless binding found: `let reason = CHAT_CLEAN_CLOSE_REASON;` in `handle_clean_close` (pure step-3 indirection, no width or reuse need) → adopted as S1. `report_reconnect`, `with_reason`, `await_reestablishment(last_detail, reason)`: match the binding plan §2 snippets exactly; no dead params; no Option/`.map` shuffles.
4. **Label consts** — single-source VERIFIED: `git grep -n "clean_close\|transient_failure"` shows the ONLY string-literal definitions are `chat_worker.rs:57–58`; delivery imports via the existing `use super::chat_worker::{...}` block (no new import line); the test module pins by constant through `super::`, never a duplicated literal. Module-doc mentions in the test header are prose, not values.
5. **File relief** — current true line counts: events.rs 153 / events_summary.rs 56 / mod.rs 75 / collector.rs 196 / collector_stages.rs 143 / state.rs 176 / diagnostics_test_fixtures.rs 147 / chat_worker.rs 196 / **chat_stream_delivery.rs 199** / chat_reconnect_reason_tests.rs 114. Only `chat_stream_delivery.rs` (slack 1) is cap-fire exposed; S1 frees it to 198. Post-edit projections: all ten files ≤200 with the two edits netting −1 and −7; every other file unchanged (see table §4).

---

## 2. Adopted edits (atomic, executable without judgment)

### S1 — inline the needless reason binding (`src/app/run/chat_stream_delivery.rs`)

Old (lines 154–158, exact):

```rust
    /// Clean close = the transient re-establishment arm (§A.4/A.5).
    pub(super) fn handle_clean_close(&mut self) -> bool {
        let reason = CHAT_CLEAN_CLOSE_REASON;
        self.await_reestablishment(STREAM_CLEAN_CLOSE_DETAIL.to_string(), reason)
    }
```

New (exact):

```rust
    /// Clean close = the transient re-establishment arm (§A.4/A.5).
    pub(super) fn handle_clean_close(&mut self) -> bool {
        self.await_reestablishment(STREAM_CLEAN_CLOSE_DETAIL.to_string(), CHAT_CLEAN_CLOSE_REASON)
    }
```

- Behavior: identical — the constant is a `&'static str`, passed unchanged to the same param; no row, label, lock span, or gating change.
- Width check (fmt safety): the new call line = 8 (indent) + 90 (content) = **98 ≤ 100** → rustfmt keeps it single-line, no re-wrap, no churn.
- File projection: 199 → **198** post-fmt (slack 2).

### S2 — reuse the existing token-batch builder in test 1 (`src/app/run/chat_reconnect_reason_tests.rs`)

Old (the inline session inside `clean_close_reconnect_rows_carry_the_clean_close_label`, exact 6 lines after the existing `fakes.script_token_batch_session();`):

```rust
    fakes
        .stream_ledger
        .script_session(vec![Some(Ok(yielded(&response_with_token(
            vec![],
            RESUME_TOKEN_FIXTURE,
        ))))]);
```

New (exact 1 line, immediately after the first `script_token_batch_session` call):

```rust
    fakes.script_token_batch_session();
```

Then the still-unused imports (the two names leave the roster once the inline block is gone):

Old (exact 3 lines):

```rust
use crate::youtube::{
    response_plain, response_with_token, text_item, yielded, LiveApiError, RESUME_TOKEN_FIXTURE,
};
```

New (exact 1 line):

```rust
use crate::youtube::{response_plain, text_item, yielded, LiveApiError};
```

- Behavior: identical — `script_token_batch_session` (runtime_chat_fakes.rs L85–91) queues `vec![Some(Ok(yielded(&response_with_token(vec![], RESUME_TOKEN_FIXTURE))))]`, byte-for-byte the replaced inline spec. Two calls queue the same TWO sessions in the same order (history-discarded, then live empty batch): same rows, same counters, same session count before the graceful-end session.
- Assertion values and the test name are UNTOUCHED (2 labelled rows, 2 batch rows, no `"reason"` on batch rows).
- Import-line shape precedent: `chat_delivery_diagnostics_tests.rs:17` uses the identical single-line mixed order (fn items then `LiveApiError`) and is fmt-stable → new line = 71 columns ≤ 100, no churn.
- File projection: 114 − 5 − 2 = **107** post-fmt.

Do NOT apply S2's pattern to test 3 — its inline `response_plain` session is a one-message live batch, a shape with NO existing builder; it stays verbatim.

---

## 3. Candidates considered and REJECTED (named)

| # | Candidate | Rejection |
|---|---|---|
| R1 | Fold local `script_lookup(fakes)` / `network_failure()` into `runtime_chat_fakes.rs` (shared with `chat_delivery_diagnostics_tests.rs`) | Binding plan §5 deliberately encoded the duplication ("test-module isolation; local fixture fns"); folding crosses the roster boundary and edits a file OUTSIDE the Task-2 commits — medium refactor → beyond the ≤50% implementer and against the plan; not "within one roster boundary". |
| R2 | Fold `network_failure()` vs `chat_worker_budget_tests.rs::network_failure(attempt)` | Different signatures/semantics (attempt-keyed reason) — not identical. |
| R3 | Narrow `SummaryEvent::new`/`SummaryEvent`/`SummarySnapshot` `pub(crate)` to `pub(super)` post-split | Plan §3 explicitly froze visibility "unchanged" at the split; narrowing cascades into the `diagnostics_test_fixtures.rs` `pub(crate) use` re-export (E0365-class) — structural scope decision. Status quo kept. |
| R4 | Merge collector.rs's two import lines (`events` + `events_summary`) into one nested path (196 → 195) | The two-line shape is the plan's §3 exact encoded form; 1-line gain is marginal and invites rustfmt re-wrap churn in the 196-line file. Hygiene check (no DUPLICATED imports) already passes. |
| R5 | Shared `reconnect_row_tail(reason)` helper across tests 1–2 in the new roster | Net line cost is POSITIVE (+2: helper 3 lines vs 1 saved); abstraction without payoff. |
| R6 | Simplify `content.matches(labelled_row.as_str()).count()` forms | `as_str()` is minimal-necessary (String is not a Pattern); pinned assertion values. |
| R7 | Fold test 3's inline `response_plain` session | No matching builder exists (`script_history_batch_session` carries a token + different content) — would change row bytes. |
| R8 | Shorten `chat_worker.rs` (196) via const-block relocation | Consts' location is the plan's encoded §2 table decision; no other Task-2-owned lines to free. Slack 4 acceptable. |
| R9 | Reorder/`pub(in ...)`-ize the label consts' visibility | `pub(super)` from `chat_worker` == `pub(in crate::app::run)` — already the minimum covering delivery + the `#[path]` test child; no change possible. |

---

## 4. Line-budget table (post-fmt projections, true file lines)

| File | Now | After S1+S2 | Slack vs 200 |
|---|---|---|---|
| `src/app/run/chat_stream_delivery.rs` | 199 | **198** (S1) | 2 |
| `src/app/run/chat_reconnect_reason_tests.rs` | 114 | **107** (S2) | 93 |
| `src/app/diagnostics/events.rs` | 153 | 153 | 47 |
| `src/app/diagnostics/events_summary.rs` | 56 | 56 | 144 |
| `src/app/diagnostics/mod.rs` | 75 | 75 | 125 |
| `src/app/diagnostics/collector.rs` | 196 | 196 | 4 |
| `src/app/diagnostics/collector_stages.rs` | 143 | 143 | 57 |
| `src/app/diagnostics/state.rs` | 176 | 176 | 24 |
| `src/app/diagnostics/diagnostics_test_fixtures.rs` | 147 | 147 | 53 |
| `src/app/run/chat_worker.rs` | 196 | 196 | 4 |

fmt-churn risk: NONE — both edits' replacement lines width-checked ≤100 cols (§2); no other file touched, so no import/block reflow anywhere. No fn_call_width outcome changes.

---

## 5. Implementer guide (execute exactly, in order; single unchained commands)

1. Apply S1 verbatim (Edit tool; the two code blocks above are the complete old/new).
2. Apply S2 verbatim (two replacements in `chat_reconnect_reason_tests.rs`: the 6-line inline block → 1 builder call; the 3-line import block → the 1-line import).
3. No other edit. Do not touch assertion values, test names, labels, the skip attribute, field order, `report_reconnect`, or `await_reestablishment`'s signature.
4. Verify (Alpine VM docker, container-only; commands single, unchained):
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` → exit 0 (if it fails, STOP and report the exact diff — do NOT run `cargo fmt` to auto-apply).
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --all-targets -- -D warnings` → exit 0.
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test` → **558 passed / 0 failed** (test COUNT unchanged; any deviation = STOP, report).
   - Line cross-check: touched files report 198 and 107 per §4.
5. Commit (single commit, repo message style): `refactor: inline the clean-close reason binding and reuse the token-batch builder per 4.3` — files: ONLY `src/app/run/chat_stream_delivery.rs` + `src/app/run/chat_reconnect_reason_tests.rs`. Then report this plan's path so the caller tracks it.

STOP protocol: any gate failure, any fmt reflow, or any line count above 200 after the edits → STOP and report to caller; do not improvise.

---

## 6. Byte-stability + constraint ledger

- Row bytes/keys/values/field order/skip attr: UNCHANGED (S1 passes the same const to the same param; S2 queues byte-identical sessions).
- Labels `clean_close`/`transient_failure`, lock spans (`state_depth`), gating (`queue_metrics`), `add_reconnect` 1-arg: UNTOUCHED.
- Test names + every assertion value: UNCHANGED (558 total tests, same roster shape).
- ≤2 args everywhere: unchanged. Post-fmt ≤200 everywhere: table §4. No sleeps added. No new `#[allow(`: edits introduce neither. Formats/serialization unchanged; demo/redaction/exit contracts not touched (both files are chat-run + test scope only).

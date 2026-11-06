# Task-1 Implementation Plan (RE-CUT, caller-redirected) — Accept the provider-real fractional `publishedAt` shape AND thread the stamp into `rendered`/`written` stage rows via the bounded in-flight entry

Source: `.agent/todos/20261106/20261106-todo-1.md` → Task 1. Global plan: `.kilo/plans/20261106-diagnostics-followup.md`.
Executed in workflow step 4.2 by the implementer on branch `fix/diagnostics-followup`, HEAD `97d775e` (v0.14.1). Planning-only step 4.1b wrote this file (re-cut in place after the caller verified the root cause); the ONLY repository write of 4.1b.
Non-front-end: NO 4.1a work exists. Task-2 (reconnect `reason`) is OUT of this plan's scope.

## 0. Verified current-state trace (read at HEAD `97d775e`; clean tree, `git diff 7437c62 97d775e` touches only docs/version — production code is v0.14.0)

### CONFIRMED ROOT CAUSE (caller-verified fixed input; re-verified in code)

**G1 — the real PRIMARY loss point is the RFC-3339 parser breadth, not the wiring of the dequeued row.**

- `src/youtube/live_chat_rfc3339.rs:15` — `const MAX_FRACTION_DIGITS: usize = 3;`
- `src/youtube/live_chat_rfc3339.rs:40–48` — `split_validated_fraction` accepts only 1–3 fractional digits; anything longer (`4..` and upward) fails → the WHOLE parse returns `None` (module doc line 6 states the 3-digit bound).
- The provider's actual `publishedAt` carries SIX fractional digits (microseconds — RFC-3339 micro precision the API emits; operator-run shape e.g. `2026-11-06T02:13:48.236707Z`, `logs/run/20261106-021348.log` line 2).
- Therefore EVERY live timestamp parse → `None` → `carrier.published_epoch = None` → `into_chat_message` attaches no timestamp → ALL per-message rows null in `logs/diagnostics.log` — INCLUDING the already-wired `dequeued` row. The operator's nulls had this SINGLE cause; message-id correlation still worked (rows carried correlations, per operator evidence).

Consequence of G1 alone: the operator's nulls WOULD have been fixed for the `dequeued` row (it is already wired and, with a parseable stamp, emits the epoch). G2 (below) is a separate, additive gap: the `rendered`/`written` rows NEVER carried the stamp regardless of parse success.

### Where the provider timestamp EXISTS upstream (and where the wiring is already correct)

1. `src/youtube/live_chat_stream_mapping.rs` → `map_item` (lines 28–44): builds `ChatTextMessage` with `published_epoch: snippet.published_at.as_deref().and_then(parse_rfc3339_seconds)` (lines 39–43). Pinned upstream by `live_chat_stream_mapping_tests.rs:134–139` (`published_epoch` equals `parse_rfc3339_seconds(PUBLISHED_AT_FIXTURE)`; fixture constant `src/youtube/stream_test_fixtures.rs:25` = `"2026-10-25T12:30:45Z"` → epoch `1_792_931_445`; a no-fraction shape, so the fixture stays valid under G1).
2. `src/youtube/live_chat_rfc3339.rs` → `parse_rfc3339_seconds` (lines 25–32): documented UTC forms → `Option<u64>` epoch seconds. **This is where G1 lands.**
3. Carrier: `src/youtube/live_chat_mapping.rs:21–26` — `ChatTextMessage { author, text, message_id: Option<String>, published_epoch: Option<u64> }`.
4. C-1 conversion `src/app/run/chat_stream_delivery.rs` → `into_chat_message` (lines 175–184): attaches `with_message_id` and `with_timestamp(Some(seconds))` into the app `ChatMessage` (`src/chat/model.rs:12–47`; readers `message_id()` line 57, `timestamp()` line 61).
5. The FULL `ChatMessage` rides the bounded channel: `send_stop_aware` (`chat_stream_delivery.rs:88–111`) `try_send`s it; `src/app/run/source.rs` passes it through unchanged; `src/app/pipeline/runner.rs:110` `self.note_message_dequeued(&message)` passes the FULL `&ChatMessage` (`runner_diagnostics.rs:13–17` forwards verbatim).
6. `src/chat/store.rs` → `accept` (104–108) / `write_log_record` (132–146): reads `message.timestamp()` verbatim; never strips it.

### Wiring gaps that remain after G1 (the earlier plan's D1 design — still real)

- **`dequeued` stage row: ALREADY WIRED.** `src/app/diagnostics/collector.rs:56–70` `note_message_dequeued` reads `message.timestamp()` via `TimingDetails::dequeued(correlation, message.timestamp()).with_queue_depth(depth)` (line 67–68). With G1 fixed this row emits truth with ZERO diff. (The previous cut's reading stands; the "every row null" finding was the parser's doing.)
- **`rendered` stage row: NULL by wiring.** `collector.rs:130–139` `record_message_render` → `TimingDetails::stage(entry.correlation.as_deref(), message_ns)` (line 137); `TimingDetails::stage` (`src/app/diagnostics/events.rs:50–57`) hardcodes `provider_epoch_seconds: None` (events.rs:53).
- **`written` stage row: NULL by wiring.** `collector.rs:93–105` `note_write_completed` → `TimingDetails::stage(...)` (line 101); the shared local bookkeeping `DequeuedMessage { correlation, stamp_nanos }` (`collector_stages.rs:56–60`, in `CollectorLocal.last_dequeued` line 53) carries NO provider field.
- **Enqueue path: NO capture.** `collector_stages.rs:82–100` `report_enqueue_success(correlation: Option<&str>)` — 1 arg — builds `InFlightEntry` (`state.rs:14–19`: `correlation, received_nanos, enqueued_nanos, backpressured`) with NO provider field; worker caller `report_enqueue_success_if_enabled` (`chat_stream_delivery.rs:113–117`) passes only the correlation.

### Tests that pin the area today

- `src/youtube/live_chat_rfc3339_tests.rs` — ONE test: `rfc3339_parsing_covers_documented_utc_forms` (24-line roster). It pins the OLD truth: line 14–16 accepts a 3-digit fraction; **line 23 REJECTS a 4-digit fraction (`".1234Z"`) — the exact pin that must be RE-PINNED to the new truth.** No empty-fraction pin exists yet.
- `src/app/diagnostics/collector_tests.rs` — 9 tests; `secret_message()` (26–30) already carries `.with_timestamp(Some(1_728_381_296))`; three `report_enqueue_success(message.message_id())` call sites (lines 39, 102, 157–158) need the new argument. No `provider_epoch_seconds` row pins anywhere (grep: zero hits in `*_tests.rs`).
- `src/app/diagnostics/state_tests.rs` — 6 tests; `entry()` literal (7–14) constructs `InFlightEntry` (needs the new field); `inflight_evicts_oldest_at_cap` (cap-128 eviction) must keep passing untouched.
- `src/app/pipeline/runner_diagnostics_tests.rs` — 2 tests, snapshot-count pins only; no change.
- `src/app/run/chat_delivery_diagnostics_tests.rs` — 4 tests; drives the REAL worker (`process_batch` → `send_stop_aware` → `report_enqueue_success`); `text_item` carriers carry `PUBLISHED_AT_FIXTURE` (no fraction); no provider pins yet.
- `src/app/run/latency_load_fixtures.rs` / `latency_load_source.rs` — harness producers use `text_item` through the REAL `process_batch`; the new arguments flow automatically; latency rosters pin snapshots/pacing → **harness fixtures are NOT touched (decision D6 stands).**
- `diagnostics_test_fixtures.rs` and `config/test_fixtures.rs` — untouched. **Docs contain NO fraction-bound claim** (grep over `docs/`: zero hits), so G1 needs NO doc-field change; the 4.4 duty stays the two per-message-row descriptions listed in §4.

## 1. Final wiring decision (encoded, one choice each — no alternatives in 4.2)

**Fix G1 — parser breadth at `split_validated_fraction`; the ONLY change is the digit bound.**

1. `src/youtube/live_chat_rfc3339.rs:15` — `MAX_FRACTION_DIGITS: usize = 3` → `9` (nano-precision bound; RFC-3339's documented nano ceiling). **Exact decision: accept 1–9 fractional digits; reject 0 digits after the dot (the current `!fraction.is_empty()` guard stays) and reject >9 as malformed.** The mechanism (`split_validated_fraction` body) is otherwise unchanged — only the constant and the module doc.
2. Honest semantics stay: `epoch_seconds(days, hour, minute, second)` receives NO fractional part — the fraction is DISCARDED; the stored epoch remains second-granularity. Encode the module-doc update (line 6 area) so it says BOTH things truthfully: parses up to 9 fractional digits, and the stored epoch value deliberately keeps second granularity (fractional precision does not carry into the epoch). No new function, no behavior elsewhere.
3. Why 9 and not 6: the system accepts every RFC-3339-legal provider shape (locator-nano upper bound) rather than fitting exactly today's micro shape; >9 stays `None` (malformed), so a bogus provider value keeps failing closed.

**Fix G2 — the in-flight entry IS the carrier for the two post-dequeue stages (D1, unchanged from the earlier cut).** The dequeued row keeps its existing DIRECT `&ChatMessage` source (already truthful; with G1 it emits the stamp from ONE call). The worker captures the provider stamp at `send_stop_aware` next to the correlation capture, `report_enqueue_success` stamps it into `InFlightEntry`, `DiagnosticsState::take_dequeued` returns the popped entry's stamp, and `note_message_dequeued` stores it into `DequeuedMessage` for `record_message_render` / `note_write_completed`. Rationale: satisfies the TODO's explicit "preserve across the bounded in-flight entry (one extra field expected)" AND keeps the new field consumed (no dead-code warning). Uncorrelated/evicted entries yield `null` on `rendered`/`written` — honest bookkeeping loss (bound still holds).

**Exact shape changes:**

1. `src/app/diagnostics/state.rs` — `InFlightEntry` gains ONE field:
   ```rust
   pub(super) provider_epoch_seconds: Option<u64>,
   ```
   `IN_FLIGHT_CAP = 128` and eviction unchanged; memory bound grows by one `Option<u64>` per entry (bounded by construction).
   `take_dequeued` changes return `() → Option<u64>` (the popped entry's stamp; `None` when uncorrelated), updating its doc line; the latency recording and uncorrelated counting stay byte-equivalent inside.
2. `src/app/diagnostics/collector_stages.rs`
   - `DequeuedMessage` gains `pub(super) provider_epoch_seconds: Option<u64>,`.
   - `report_enqueue_success` extension (≤2-args rule: exactly 2 args, NO new parameter object):
     ```rust
     pub(crate) fn report_enqueue_success(&self, correlation: Option<&str>, provider_epoch_seconds: Option<u64>)
     ```
     and the `InFlightEntry` literal gains the field. `Option<&str>` is `Copy` — the existing `correlation` reuse in `QueueEnqueueFields` stays compilable. `QueueEnqueueFields`/`QueueEvent` shapes UNTOUCHED (queue rows keep byte-identical JSON).
   - `record_message_render` MOVES from `collector.rs` into the existing `impl DiagnosticsCollector` block here (visibility `pub(super)`; uses the already `pub(super)` `local`/`with_state`/`emit`), its row emission becoming `TimingDetails::stage(entry.correlation.as_deref(), message_ns).with_provider(entry.provider_epoch_seconds)`; the events import line widens with `TimingDetails, TimingEvent`.
3. `src/app/diagnostics/collector.rs`
   - New private helper (replaces the `state_depth` call ONLY in `note_message_dequeued`):
     ```rust
     fn state_dequeue(&self, correlation: Option<&str>, now: u64) -> (u64, Option<u64>) {
         self.with_state(|state| {
             let provider_epoch_seconds = state.take_dequeued(correlation, now);
             (state.queue_depth(), provider_epoch_seconds)
         })
     }
     ```
   - `note_message_dequeued`: `let (depth, enqueued_provider) = self.state_dequeue(correlation, now);` and the `DequeuedMessage` literal gains `provider_epoch_seconds: enqueued_provider,`. The dequeued-row emission (`TimingDetails::dequeued(correlation, message.timestamp())`) stays exactly as-is (direct source).
   - `note_write_completed`: the written-row emission becomes `TimingDetails::stage(entry.correlation.as_deref(), lag_ns).with_provider(entry.provider_epoch_seconds)`.
   - `note_render_completed` keeps calling `self.record_message_render(now)` (resolved from the sibling impl after the move).
4. `src/app/diagnostics/events.rs` — ONE builder added to `TimingDetails` (≤2-args rule; mirrors the existing `with_queue_depth` pattern):
   ```rust
   pub(super) fn with_provider(mut self, provider_epoch_seconds: Option<u64>) -> Self {
       self.provider_epoch_seconds = provider_epoch_seconds;
       self
   }
   ```
   `TimingDetails::stage` keeps its 2-arg signature and `None` default; `TimingEvent` already serializes `provider_epoch_seconds` (events.rs:18, 66). NO new keys; `SummaryEvent`/`QueueEvent` untouched.
5. `src/app/run/chat_stream_delivery.rs` — inside `send_stop_aware` directly AFTER the existing `reported_correlation` capture (lines 93–97), mirror-pattern capture:
   ```rust
   let reported_provider = self.plan.diagnostics.as_ref().and_then(|_| message.timestamp());
   ```
   The `Ok(())` arm becomes `self.report_enqueue_success_if_enabled(reported_correlation.as_deref(), reported_provider);`. `report_enqueue_success_if_enabled` gains the second parameter (2 args ≤ rule). The `Full` arm / `report_enqueue_full_if_enabled` / `note_worker_receipt` / `report_batch_received` are UNTOUCHED (no provider on queue rows).
6. `src/app/diagnostics/mod.rs` — register the new roster: `#[cfg(test)] #[path = "collector_provider_tests.rs"] mod collector_provider_tests;` (roster registration convention lives here, see `collector_tests` precedent).

**Not changed (enumerated for 4.2/4.3 verification):** `src/youtube/live_chat_stream_mapping.rs` (its `and_then(parse)` call is the G1 beneficiary, code unchanged), `src/youtube/stream_test_fixtures.rs` (PUBLISHED_AT_FIXTURE stays the no-fraction shape — changing it would re-pin many rosters), `src/chat/model.rs`, `src/chat/store.rs`, `src/app/run/latency_load_*`, `diagnostics_test_fixtures.rs`, `config/test_fixtures.rs`, `runner.rs`/`runner_diagnostics.rs`, all queue-row JSON shapes, `SummarySnapshot`, `CLOCK_DOMAIN_NOTE`.

## 2. Test plan (existing rosters first; ONE new diagnostics roster because `collector_tests.rs` is at 183/200)

All offline, fake clocks (`TestCollector.advance`), zero sleeps, no new deps/`allow(`:

### G1 pins — `src/youtube/live_chat_rfc3339_tests.rs` (roster stays 1 test file; count +1)

| # | Change | File | Pins |
|---|--------|------|------|
| R1 (RE-PIN, truth change) | `rfc3339_parsing_covers_documented_utc_forms` line 23 | `live_chat_rfc3339_tests.rs` | The 4-digit-fraction assert FLIPS to the new truth: `parse_rfc3339_seconds("2026-10-25T12:30:45.1234Z")` now returns `Some(1_792_931_445)`. Additionally inside the SAME test: a 10-digit fraction `".1234567890Z"` → `None` (stays rejected), and the EMPTY fraction shape `"2026-10-25T12:30:45.Z"` → `None` (new assert for a previously unpinned rejection that stays closed). |
| R2 (NEW test) | `micro_and_nano_fraction_shapes_parse_to_the_same_second` | `live_chat_rfc3339_tests.rs` | The provider-real micro shape: `parse_rfc3339_seconds("2026-10-25T12:30:45.236707Z")` → `Some(1_792_931_445)` — SIX digits accepted AND the second-granularity truth (same base-second value as the no-fraction form, proving the fraction is discarded); plus `parse_rfc3339_seconds("2026-10-25T12:30:45.000000001Z")` → `Some(1_792_931_445)` — the 9-digit ACCEPT bound. |

### G2 pins

| # | Test name (exact) | File | Pins |
|---|-------------------|------|------|
| T1 | `timestamp_bearing_message_populates_all_three_stage_rows` | NEW `src/app/diagnostics/collector_provider_tests.rs` | Drive the full stage cycle (`report_batch_received` → `note_worker_receipt` → `advance(10)` → `report_enqueue_success(Some("m-9"), Some(1_792_931_445))` → `advance(20)` → `note_message_dequeued(&message)` → `note_render_started` → `advance(5)` → `note_render_completed` → `note_write_started` → `advance(7)` → `note_write_completed`) on `collector_with_output_file(&path)` with a `ChatMessage` carrying `.with_message_id("m-9").with_timestamp(Some(1_792_931_445))`; assert `read_to_string(path).matches("\"provider_epoch_seconds\":1792931445").count() == 3` (dequeued + rendered + written). Model the cycle on `drive_full_stage_cycle`. |
| T2 | `timestamp_absent_rows_stay_null_on_all_three_stages` | NEW same roster | Same cycle with a message WITHOUT `.with_timestamp` (id `"m-absent"`); assert `matches("\"provider_epoch_seconds\":null").count() == 3`. |
| T3 | `take_dequeued_returns_the_popped_entry_provider_stamp` | `src/app/diagnostics/state_tests.rs` | Entry with `provider_epoch_seconds: Some(42)` → `take_dequeued(Some("m-1"), 10)` returns `Some(42)`; an entry with `None` returns `None`; an unmatched correlation returns `None` (and bumps `uncorrelated_total`). The local `entry()` helper gains `provider_epoch_seconds: None,`. |
| T4 | `worker_stamp_survives_the_bounded_in_flight_entry` | `src/app/run/chat_delivery_diagnostics_tests.rs` | End-to-end worker truth: script `text_item("m-1")` + offline batch (pattern of `worker_reports_batch_received_and_enqueue_success`), harness `collector_with_output_file(&path)` (import from `diagnostics_test_fixtures`), `plan.diagnostics = Some(...)`, `spawn_chat_worker(plan).join()`, then `channel.receiver.recv()` → `note_message_dequeued(&message)` → `note_render_started` → `note_render_completed`; read the file and assert dequeued and rendered rows each carry `"provider_epoch_seconds":1792931445` (`count() == 2`). Pins the worker-side capture chain: carrier publishedAt → enqueue arg → `InFlightEntry` → popped stamp → rendered row. The fixture's no-fraction shape keeps the literal valid; G1's micro shape is pinned by R2 at the parser boundary. |
| T5 (assertion, not a test) | — | `collector_tests.rs` `emits_summary_after_interval_and_not_earlier` | Add `assert!(!content.contains("\"provider_epoch_seconds\""), "summary rows never carry the per-message field");` — pins summary rows UNCHANGED (careful: `clock_note` DOES contain the word "provider"; assert the full quoted KEY only). |

**Compile-preserving re-pinning / truth re-pinning (NEVER truth-weakening):**
- R1 is a TRUTH re-pin: the 4-digit rejection pin was written against the old 3-digit bound; the new bound makes that assert false, so it changes WITH the parser change (same commit), stating the new acceptance explicitly.
- `collector_tests.rs` lines 39, 102, 157–158: add `message.timestamp()` as the second argument to the three `report_enqueue_success(...)` calls (`secret_message()` carries a timestamp; `uncorrelated_message_is_counted_not_dropped` uses one without → `None` naturally).
- `state_tests.rs` `entry()` literal gains the field.
- NO other existing assertion changes. Rosters: 9 collector + 6 state + 2 runner + 4 delivery + 1 rfc3339 + 6+4 latency all remain; +1 state, +1 delivery, +1 rfc3339 test, +1 new roster (2 tests).

## 3. Line-budget table (post-rustfmt; every touched source file ≤ 200; count with real newlines)

| File | Current | Delta projection | Projected | Risk |
|---|---|---|---|---|
| `src/youtube/live_chat_rfc3339.rs` | 153 | const 3→9 (same lines), module doc rewrite (+1–2 honest lines) | ~155 | low |
| `src/youtube/live_chat_rfc3339_tests.rs` | 24 | R1 flip/extends (+2 asserts), R2 new test (+~9) | ~34 | low |
| `src/app/diagnostics/state.rs` | 165 | +1 field, +2 return/arms, +1 doc | ~169 | low |
| `src/app/diagnostics/state_tests.rs` | 88 | +1 helper field, +~12 T3 | ~101 | low |
| `src/app/diagnostics/collector_stages.rs` | 116 | +1 field, +1 arg+entry field, +~13 moved `record_message_render` + import widening | ~131 | low |
| `src/app/diagnostics/collector.rs` | 199 | −11 (move out) +7 (`state_dequeue`) +1 literal +1 builder wrap | ~197 | **HIGH — STOP if rustfmt pushes > 200** |
| `src/app/diagnostics/events.rs` | 189 | +5 builder | ~194 | low |
| `src/app/diagnostics/mod.rs` | 70 | +3 registration | ~73 | low |
| `src/app/run/chat_stream_delivery.rs` | 192 | +1 capture line (arm/signature re-wrap in-line) | ~194 | low |
| NEW `src/app/diagnostics/collector_provider_tests.rs` | 0 | T1+T2 | ≤ 100 | low |
| `src/app/diagnostics/collector_tests.rs` | 183 | +2 wraps, +1 T5 assertion | ~187 | medium |
| `src/app/run/chat_delivery_diagnostics_tests.rs` | 138 | +~32 T4 + import line | ~171 | low |

**STOP protocol:** if any file exceeds 200 lines post-fmt, or `state_dequeue` forces a third argument, or a new `#[allow(` or dependency is needed, or a G1/G2 test cannot be made offline/deterministic — HALT, report the exact file/function and the overflowing projection to the caller; do NOT improvise further splits (roster/sibling split adjudication belongs to the caller, precedent R3).

**Guardrails for 4.2:** no commented-out code, no new `#[allow(`, no new dependency, ≤2 arguments per fn (D1 chose exactly 2 everywhere), ≤2 nesting depth per touched fn body, ≤50 lines per fn, prefer `pub(super)`/private, self-documenting names, single-section boolean conditions (existing guards and the three-arm `try_send` match keep their shapes; no new compound conditions), real newlines only, G1 touches ONLY the constant + the module doc in `live_chat_rfc3339.rs` (no parse-logic restructure).

## 4. Git boundaries (branch `fix/diagnostics-followup` stays; no push in 4.2; `origin` only)

1. **Plan-tracking commit (plan file only):** `docs: cut provider timestamp task1 plan re-cut` — stage ONLY this plan file.
2. **Feature commit A (production wiring + compile/truth-preserving test updates):** `feat: accept provider micro-fraction timestamps and thread the publication stamp into stage rows` — files: `live_chat_rfc3339.rs`, `state.rs`, `collector_stages.rs`, `collector.rs`, `events.rs`, `mod.rs` (registration), `chat_stream_delivery.rs`, `collector_tests.rs` (arg updates + T5 assertion), `state_tests.rs` (helper literal), AND `live_chat_rfc3339_tests.rs` R1 (the truth re-pin must ride WITH the parser change — the old 4-digit reject would otherwise fail the gate). Gate after A: **550/0 expected** (zero new tests; R1 flips an existing assert without changing the count).
3. **Test commit B (new pins):** `test: pin fraction-acceptance bounds and provider-stamped stage rows` — files: R2 in `live_chat_rfc3339_tests.rs`, NEW `collector_provider_tests.rs` (T1, T2), `chat_delivery_diagnostics_tests.rs` (T4). Gate after B: **555/0 expected** (+5: R2, T1, T2, T3, T4).
4. **DO-COMMIT list:** nothing else. `.gitignore` check before each commit (`git status` shows only intended paths; no `target/`, `dist/`, `logs/`, `videos/`).
5. **DOCS DUTY IS OUT OF 4.2** (4.4 docs-specialist). Truthful lines to change there (informational list):
   - `docs/production-run.md:335` — pipeline-timing bullet: one sentence that the three stage rows carry `provider_epoch_seconds` with the provider-supplied `publishedAt` epoch WHEN the provider supplied one and the stamp survived the bounded in-flight staging (null otherwise: demo messages, evicted/uncorrelated staging, unparseable stamps).
   - `docs/production-run.md:369` — clock-domain paragraph: may confirm the per-message wall-clock stamp is carried "when available" (honest wording; no real-Google claims).
   - `docs/latency-test-map.md:97` — correlation step: still truthful; the specialist may tighten wording to "when present".
   - NO doc needs to change for G1 (no doc states a fraction-digit bound; grep verified). The ONLY honest-precision note lives in the `live_chat_rfc3339.rs` module docs (in-source).

## 5. Byte-stability ledger

- **Unchanged:** FFmpeg argv (recording + streaming vectors), `demo` schedule/Display/exit path, `ChatMessage::Display` (`model.rs:66–70`), exit codes, the single-`error:` contract, redaction surfaces (no new strings; the stamp is a bare `u64`), enabled/disabled/invalid diagnostics config behaviors (`from_config` gating untouched; zero cost when diagnostics off — one extra `Option` capture at two already-instrumented sites), queue-row JSON (batch/enqueue/full/reconnect byte-identical), summary rows byte-identical (T5), `clock_note` verbatim, `IN_FLIGHT_CAP` 128 + eviction + high-water/oldest-age accounting, `CollectorLocal`/`report_*` queue semantics, chat-store log-record behavior, and every RFC-3339 rejection shape EXCEPT 4–9 fractional digits (date/calendar/hour/minute/second/missing-Z/non-digit-component/empty-fraction rejections unchanged — R1 keeps asserting them).
- **The ONLY serialization change:** on the three per-message stage rows, the EXISTING key `provider_epoch_seconds` goes from constant `null` to the numeric provider epoch when the provider supplied a parseable `publishedAt` (now including 4–9 fractional-digit shapes) and the stamp survived the bounded staging; absent stays `null`. No new keys, no field reordering.
- **Gate arithmetic:** baseline OBSERVED 550/0 → commit A: 550/0 (R1 flips an existing assert; no new tests) → after commit B: 550 + 5 (R2, T1, T2, T3, T4) = **555/0 expected**. OBSERVED governs: if the pre-change tree's actual count differs at 4.2 start, re-derive and record the arithmetic in the close-out.

## 6. Verification commands (Alpine VM; ONE command per call; VM status check FIRST)

1. `alpine-vm_vm_status` — before any command.
2. Targeted (after commit A):
   - `docker compose run --rm rust sh -c "cargo test -q rfc3339"` (R1 green)
   - `docker compose run --rm rust sh -c "cargo fmt --check"`
   (after commit B, additionally:)
   - `docker compose run --rm rust sh -c "cargo test -q provider"` (T1/T2/T4)
   - `docker compose run --rm rust sh -c "cargo test -q diagnostics::state_tests"`
   - `docker compose run --rm rust sh -c "cargo test -q collector_tests"`
3. Full gate at the 4.2 finale: `sh scripts/dev-checks.sh` (format + check + clippy + full test; UTC-timestamped log under gitignored `logs/checks/`). Acceptance: gate green, 555/0 (or corrected arithmetic), ≤200 lines on every touched file, zero new `#[allow(` (`git grep -n "allow(" -- src/ | wc -l` stays at the documented baseline).
4. Step 4.3 cross-checks: `with_provider` used at BOTH stage sites; `state_dequeue` single consumer; `provider_epoch_seconds` on all three row constructions; no `TimingDetails` third-arg; no queue-row shape diff; `MAX_FRACTION_DIGITS` is the ONLY logic-bearing delta in `live_chat_rfc3339.rs`.

## 7. Traceability — TODO Task-1 requirement → plan section

| TODO Task-1 requirement | Plan section |
|---|---|
| Trace where the provider timestamp is lost (mapping/parser/carrier/C-1) | §0 (G1 = parser breadth — the primary loss; dequeued wiring already present but parsed to `None`) |
| Verify `note_message_dequeued` + enqueue/receipt report discards | §0 (dequeued already wired; rendered/written/in-flight gaps = G2) |
| TODO §40(1): provider wall-clock carried per message WHEN AVAILABLE — both corrected paths: (a) provider `publishedAt` → parser (G1: 1–9 fractional digits accepted, fraction discarded, second-granularity epoch) → carrier epoch → `dequeued` row direct from `&ChatMessage`; (b) same stamp → worker capture at `report_enqueue_success` → `InFlightEntry` → popped at dequeue → `rendered`/`written` rows (G2) | §1 (G1 + G2), §2 (R1/R2 + T1–T4) |
| Populate `provider_epoch_seconds` on dequeued/rendered/written; absent stays null | §1, §2 T1/T2 |
| Staging correlation preserves the value across the bounded in-flight entry (one extra field; cap 128; memory bounded) | §1 items 1–2 (state/collector_stages), §2 T3/T4 |
| No cross-domain duration math; `clock_note` semantics stay valid | §5 (no subtraction added; note byte-stable; fraction simply discarded — never subtracted) |
| No visible-text change | §0.4/§5 (`Display` untouched) |
| Offline pins: populated/absent, bounds hold, summary unchanged | §2 R1/R2 + T1–T5 (T5 = summary unchanged; the existing cap-128 eviction pin untouched) |
| No sleeps; injected clocks | §2 harness usage (`TestCollector.advance`) |
| Re-pin historical expectations ONLY truthfully | §2 R1 (the 4-digit reject pin — changed truth) + arg-count updates (silent pins) |
| No new deps/`allow(`, ≤200 lines, ≤2 args, ≤2 depth, junior-executable | §3 + STOP protocol |
| Docs update is 4.4 work | §4.5 (listed lines; G1 needs no doc change — verified) |
| Acceptance: rows carry the numeric epoch on all three stages; gates pass; enabled/disabled/invalid config behaviors unchanged | §2, §5, §6 |

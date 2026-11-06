# Task-2 4.5b Overall Plan Adherence — reconnect rows gain a non-secret `reason` label

Step: 4.5b (Overall Plan Adherence), TASK 2 only, TODO `.agent/todos/20261106/20261106-todo-1.md`.
Binding plan: `.kilo/plans/20261106-followup-task2-reconnect-reason-plan.md` (tracked `a25af2f`).
4.3 simplification plan: `.kilo/plans/20261106-followup-task2-reconnect-reason-simplification-plan.md` (untracked — 4.6 carries it).
Branch: `fix/diagnostics-followup`. Audit date: 2026-11-06. Audit method: read-only git + file reads; NO gate re-run (the caller-observed gates govern; detailed below).

## Verdict

**ADHERENT WITH ACCEPTABLE DEVIATIONS — zero non-adherent items, zero REQUIRED fixes.**
All four caller-approved deviations verified landed exactly as described; three new minor note-items (N1–N3) are recorded and classified ACCEPTABLE — they need no code change, and one (N3, README test-count staleness) is flagged for the 4.6 close-out decision only.

## Audit scope (verified)

Commits (HEAD `6f825a7`): `a25af2f` (plan) → `b3eec6e` (split) → `e78093b` (labels) → `51c742b` (tests) → `6f825a7` (S2 dedup fold) + UNCOMMITTED docs pass (`docs/production-run.md`, `.agent/project-structure.md`) + untracked simplification plan. `git status` confirms src/ is exactly the committed state (only the 2 docs paths modified; 1 untracked plan file) — the 4.2 abort/retry cycle left no code-state ambiguity: the successful pass committed everything; the post-commit docs deltas are `.md`-only (non-cargo-inputs), so the final gate log stays authoritative for the code tree.

## 1. Row semantics (checklist 1) — SATISFIED, evidence-cited

- `src/app/diagnostics/events.rs:77–88` — `QueueEvent` declares `reason: Option<&'a str>` as the **LAST** field (L87) after `backoff_ms` (L85), under `#[serde(skip_serializing_if = "Option::is_none")]` (L86). Serializer = struct order (`collector.rs` `emit` → `serde_json::to_string`), so `reason` appends at row end exactly after `backoff_ms`.
- `with_depth` skeleton initializes `reason: None` (events.rs:109) ⇒ batch/enqueue/full queue rows **never emit the key** (byte-absence, not `null`); `backpressure`/`wait_ns`/`batch_size`/`correlation` keep their pinned `null` emission (no skip attr anywhere else; `with_reason` is the only attach point, events.rs:140–143).
- `TimingEvent` rows carry NO `reason` field (events.rs:10–18) — message-stage rows cannot carry the key.
- Labels are EXACTLY `clean_close` / `transient_failure`: `src/app/run/chat_worker.rs:57–58` — `pub(super) const CHAT_CLEAN_CLOSE_REASON: &str = "clean_close";` / `CHAT_TRANSIENT_FAILURE_REASON: &str = "transient_failure";` (single source; delivery imports via its existing `use super::chat_worker::{…}` block, delivery L20–23; tests pin by constant via `super::`, roster L15).
- Every production call site labels: clean-close arm → `handle_clean_close` (delivery L156–157, via the caller-approved binding indirection); transient arm → `handle_failure` (delivery L151). The failure arm NEVER feeds `error.to_string()` into rows — `error.to_string()` goes only into `await_reestablishment`'s `last_detail`, consumed solely by the pre-existing terminal-cell path (`record_terminal(… transport_terminal(last_detail))`, delivery L167); diagnostics rows receive only the `&'static str`. Signature: `await_reestablishment(&mut self, last_detail: String, reason: &'static str)` (delivery L164), `report_reconnect(&self, backoff_ms: u64, reason: &'static str)` (collector_stages.rs:116–121) — both ≤2 args, byte-for-byte the plan §2 snippets (verified via `git diff b3eec6e..e78093b`).
- Unknown/other-cause honesty holds as planned: `reason` stays `Option` + skip-attr, so a hypothetical unlabelled cause still writes its row (the row shape is preserved; today both production sites always label).
- Per-row filter pins: roster test 1 filters the batch lines (`"event":"queue"` AND `!"batch_size":null`, count == 2, all without `"reason"`) and test 3 pins per-row absence on `backoff_ms:null` queue rows.

## 2. Split (checklist 2) — SATISFIED, evidence-cited

`git diff eeffc4e..b3eec6e` verified line-by-line: the summary half left `events.rs` **verbatim** (removed block byte-identical to `events_summary.rs:11–56` incl. the `CLOCK_DOMAIN_NOTE` string and both `SummaryEvent`/`SummarySnapshot` `pub(crate)` visibilities unchanged); the ONLY widening is `fn epoch_seconds_now` → `pub(super) fn` (minimal, per plan §3); `collector.rs` import split into the two encoded lines, `state.rs`/`diagnostics_test_fixtures.rs` imports swapped in place (net 0 lines each), `mod.rs` `mod events_summary;` inserted after `mod events;`. No behavior-bearing line changed — the split's neutrality is proven structurally (verbatim diff) and by the final gates' green summary/queue-row pins; no intermediate post-split gate log exists (the 4.2 abort/retry cycle is the caller-approved cover).

## 3. Tests (checklist 3) — SATISFIED WITH TWO LEDGER NOTES

- Count: gate arithmetic 555 + 3 = **558 / 0** OBSERVED in BOTH post-code logs (`20261106T140218Z.log`, `20261106T142629Z.log`, both `ALL CHECKS PASSED`).
- Names exactly per plan §5: `clean_close_reconnect_rows_carry_the_clean_close_label`, `transient_failure_reconnect_rows_carry_the_transient_failure_label`, `other_queue_rows_carry_no_reason_key` (`src/app/run/chat_reconnect_reason_tests.rs`, final 107 lines).
- Independence/zero-sleeps: the roster duplicates `script_lookup`/`network_failure` locally (plan-encoded isolation; simplification R1 correctly rejected a cross-roster fold), uses `join()` + tempdir file reads, no sleeps anywhere.
- Existing-test no-repin claim VERIFIED TRUTHFUL: `chat_delivery_diagnostics_tests.rs::worker_reports_reconnect_with_backoff` (L143–160) is untouched by all Task-2 commits (commit footprints exclude the file) and asserts ONLY state counters (`reconnect_count == 1`, `reconnect_backoff_total_ms == 1_000`) — semantics unchanged by the 1-arg `add_reconnect` (state.rs:86–89 confirmed).
- **N1 (newly recorded, ACCEPTABLE):** the plan §5 also specified snapshot-counter asserts in tests 1–2 (`reconnect_count == 2/1`, backoff totals). The landed tests 1–2 (checked at 51c742b and after the fold) assert row counts + labelled-row counts only; no snapshot assert exists. Equivalent semantics hold: each labelled row ↔ one counter increment+backoff-add happen in the ONE code path (`report_reconnect`), and the state side stays pinned by the untouched `worker_reports_reconnect_with_backoff`. Deviation was not in the caller's approved list — recorded here for the ledger; classified ACCEPTABLE (no fix required; state coverage exists at roster level).
- Dedup fold (`6f825a7`) verified behavior-preserving: it swapped the inline `response_with_token(vec![], RESUME_TOKEN_FIXTURE)` session for a second `fakes.script_token_batch_session()` whose body (runtime_chat_fakes.rs:85–91) is that byte-for-byte same builder call; import trims matched; only that 1 file changed (11-line diff, net −7 → 107).

## 4. Caps / hygiene (checklist 4) — SATISFIED (final post-fmt table, own recount)

| File | Lines (measured now) | Plan post | ≤200 |
|---|---|---|---|
| `src/app/run/chat_stream_delivery.rs` | **199** | 199 (post-S1-rejection) | ✓ |
| `src/app/run/chat_worker.rs` | **196** | 194 | ✓ |
| `src/app/diagnostics/events.rs` | **153** | ~155 | ✓ |
| `src/app/diagnostics/events_summary.rs` | **56** | ~55 | ✓ |
| `src/app/run/chat_reconnect_reason_tests.rs` | **107** | ~95 | ✓ |
| `src/app/diagnostics/collector.rs` | **196** | 196 | ✓ |
| `src/app/diagnostics/collector_stages.rs` | **143** | 143 | ✓ |
| `src/app/diagnostics/mod.rs` | **75** | 75 | ✓ |
| `src/app/diagnostics/state.rs` | **176** | ~159 (stale estimate) | ✓ |
| `src/app/diagnostics/diagnostics_test_fixtures.rs` | **147** | 147 | ✓ |

- Args ≤2 everywhere touched (see §1); **no param >2** introduced. Depth ≤2 at all changed sites (the new code adds no third-level nesting).
- `#[allow(` count across `src/`: **9** (8 `dead_code` + 1 relocated `#[allow(clippy::manual_clamp)]` in `chat_worker_helpers.rs:30`) — IDENTICAL to the pre-task baseline, zero new.
- fmt: the final trees pass `cargo fmt --check` in both post-code gate logs (both `PASS fmt-check`).

## 5. Byte-stability (checklist 5) — SATISFIED

Task-2 commit footprints (verified via range diffs) touch ONLY the plan §3's table files + 2 new files; `Cargo.toml` (v0.14.1 confirmed read) / `Cargo.lock` / `src/streaming/` / `src/app/demo/` / `src/config/` / `src/youtube/` all untouched. The byte-stability witnesses — the exact locked streaming argv vectors (stream_args_tests.rs audio-OFF and with-audio literal `vec![]` pins, 33/42 tokens), demo schedule (28 messages / 496 frames, schedule.rs `DEMO_MESSAGE_TOTAL` + tests), Display/exit-code/`[redacted]`/shutdown-order/64-capacity pins — are untouched and green in both 558/0 gates. Pre-existing queue-row shapes: verified struct-order + absence/presence reasoning (§1) plus the untouched `collector_tests.rs` queue-row/summary witnesses green in both logs. Summary-row bytes: `events_summary.rs` moved struct verbatim (§2) and `emits_summary_after_interval_and_not_earlier` runs green in both logs.

## 6. Docs pass (checklist 6) — SATISFIED (uncommitted, 2 paths)

- `docs/production-run.md`: exactly ONE line changed (the `Queue/backpressure diagnostics` bullet, §Pipeline Observability — the canonical section) appending ONE sentence byte-identical to plan §6's encoded sentence (verified `git diff`); no duplicated procedures added anywhere else in the file.
- `.agent/project-structure.md`: recount numbers INDEPENDENTLY RE-VERIFIED by my own counting (≥3 spot-checks): run/ = **41 files** counted in the directory listing (16 sources + 6 fixtures + 19 rosters all reconcile); run rosters = **19 rosters / 77 tests** (the map's own arithmetic sums to exactly 77 with `chat_reconnect_reason_tests 3` included); diagnostics/ = **14 files** (8 sources + 5 rosters + 1 fixture reconcile with the map's file entries); the `events.rs`/`events_summary.rs`/`collector_stages.rs`/`chat_stream_delivery.rs`/`chat_worker.rs`/Cargo.toml v0.14.1 entries are all truthful against the final tree I read.
- Secret safety: the uncommitted diffs contain no tokens, no RTMPS URLs, no keys, no real-Google claims — the labels are app-local fixed words; the provider-internals exclusion is stated in the doc sentence (`never the raw error text` clause lives in the structure map; the doc sentence carries only the label words).
- **N3 (RESIDUAL, for 4.6/caller):** `README.md:47` still reads "550 tests, re-recorded by the 2026-11-01 latency test-map pass" while the final truth is 558. The drift PRE-EXISTS Task 2 (introduced when Task-1's chains raised 550→555; Task-1's own docs commit `15acc8b` touched 3 paths, README not among them) and the plan §6 scoped Task-2 docs to production-run.md only. NOT a Task-2 non-adherence; the docs commit at 4.6 may ride a one-line README integer fix (550 → 558) if the caller wants the convention precedents honored.

## 7. TODO Task-2 acceptance criteria (checklist 7) — offline evidence governs; live proof operator-run by design

| Acceptance criterion | Verdict | Evidence |
|---|---|---|
| Reconnect rows distinguish the two cause labels offline; all other rows' shapes unchanged | SATISFIED-WITH-EVIDENCE | Roster tests 1–3 (§3) + TestCollector/`{...,"backoff_ms":1000,"reason":"<label>"}` line pins waited green within both 558/0 gates; skip-attr key-absence verified in events.rs (§1) |
| No secrets and no provider-internal materials appear in labels or rows | SATISFIED | Fixed `pub(super)` consts only (§1); `error.to_string()` provably never reaches rows (delivery diff §1); labels grep-uniquely single-sourced (simplification plan §1.4 audit + my read) |
| Gates pass (fmt/check/clippy/test); previous diagnostics tests stay green or re-pinned truthfully | SATISFIED | Both post-code gate logs: four PASS lines each, `ALL CHECKS PASSED`, 558/0; ZERO re-pins needed (§3 truthful-audit); zero new `allow(` (§4) |

Requirement bullets: reason-on-reconnect-rows-only ✓ (§1/§3-test-3); lowercase-underscore stable labels ✓; unknown/other causes still write the row ✓ (Option+skip preserved, §1); offline pin coverage for clean-close, transient, other-rows shapes ✓ (§3); doc update one-truth-sentence on the canonical section ✓ (§6; commit pending at 4.6). No live/real-Google claim exists anywhere in the diff ✓.

## 8. Deviations ledger (checklist 8) — complete

| # | Deviation | Classification | Verification |
|---|---|---|---|
| V1 | `events_summary.rs` split pre-decided (events.rs 194 → 153 + 56) | ACCEPTABLE (caller-approved) | §2 — verbatim move; plan §3 slack math holds (153 within ~155 slack band) |
| V2 | `let reason = CHAT_CLEAN_CLOSE_REASON;` binding indirection (199) AND 4.3's S1 fold **REJECTED-BY-FMT** | ACCEPTABLE (caller-approved; interplay recorded) | Current tree carries the binding at delivery L156 and passes fmt-check on both post-code gates; the S1 as-written projection ("98 ≤ 100 stays single-line; 199 → 198") was falsified by the real fmt run — tentatively re-applied then reverted the moment the identical call reflow re-fired, proving the original fmt ruling necessary. **The plan-simplification's §S1 is henceforth REJECTED**; the binding indirection is the sanctioned final shape. S2 (the roster fold = 6f825a7) stands |
| V3 | Test-3 per-row key-absence adaptation | ACCEPTABLE (caller-approved) | §3 — intent preserved: ≥2 non-reconnect queue rows found and asserted reason-free per row |
| V4 | 4.2 abort/retry cycle | ACCEPTABLE (caller-approved) | §audit-scope — working tree = the successful pass's commits; docs-only deltas on top |
| N1 | Tests 1–2 omit the plan-specified snapshot-counter asserts | ACCEPTABLE (NEW, recorded) | §3 — equivalence + untouched state-counter pin |
| N2 | Plan §3 "Before" numbers stale (delivery listed 198/198; task baseline was 192, final 199; state.rs listed ~159; actual 176 pre-task from Task-1's InFlightEntry work) | PLAN-DEFECT NOTE (NEW, no consequence) | §4 — caps hold; STOP protocol never fired on these; recorded per instruction to note the §3 slack-assumption defect |
| N3 | README "550 tests" staleness (vs 558) | RESIDUAL flag for 4.6 (NEW) | §6 — drift pre-dates Task 2; outside Task-2 docs scope |
| N4 | Delivery import block = 4 wrapped lines vs plan's "stays 3" | ACCEPTABLE (NEW, fmt-governed local shape) | delivery L20–23; within cap; a rustfmt wrap-fact, not a scope decision |

## 9. Gate-log state map (resolving the caller's timestamp ambiguity)

| Log (UTC stamp) | Test count | State it governs |
|---|---|---|
| `logs/checks/20261106T030703Z.log` | 555/0 | mid-Task-1-era tree |
| `logs/checks/20261106T033632Z.log` | 555/0 | Task-1 final tree (Task-2 planning baseline; matches plan's "OBSERVED 555/0") |
| `logs/checks/20261106T140218Z.log` | **558/0** | post-tests tree (51c742b, BEFORE the 4.3 S2 fold — ran 14:02:31Z; test commit at 14:02:56Z) |
| `logs/checks/20261106T142629Z.log` | **558/0** | **FINAL tree including the S2 fold** (gate 14:26:29Z; fold commit 6f825a7 at 14:29:00Z; src identical to the committed final state) |

⇒ The post-4.3-fix re-verify log IS `20261106T142629Z.log`; the pre-fold 558/0 belongs to `20261106T140218Z.log`; both cite identical counts, so the caller's citations are doubly-covered — the FINAL-tree evidence the adherence verdict stands on is `20261106T142629Z.log` (`ALL CHECKS PASSED`; fmt/check/test/clippy all PASS; version 0.14.1 inside the log).

## 10. Traceability (plan §9 → evidence)

| Plan §9 row | Evidence |
|---|---|
| Reason on reconnect rows ONLY, from today's classification, fixed labels | §1 (events.rs struct order + skip attr; delivery diffs; ≤2-arg signatures) |
| Exact label words chosen in this task's plan slot | §1 (chat_worker.rs:57–58 = `"clean_close"` / `"transient_failure"`) |
| Unknown/other causes still write the row; classifier enforces labels | §1 (Option+skip preserved) |
| Offline pins: clean-close, transient, other-rows shape | §3 (roster tests 1–3, green in both 558/0 logs) |
| Doc one-truth-sentence, link-not-duplicate | §6 (production-run.md single hunk on the canonical section) |
| Gates pass / prior tests green or re-pinned truthfully | §3/§9 (558/0 both logs; zero re-pins, audit honest) |
| No secrets/provider internals in labels or rows | §1/§6 |
| ≤2 args, ≤200 lines, no deps, no allows, no sleeps | §3/§4 (table; 9-allow audit unchanged; Cargo.toml untouched) |

## Close-out items owned elsewhere (NOT required fixes of this audit)

1. 4.6: commit the 2 uncommitted docs paths + track the untracked simplification plan file (workflow step hygiene; caller-ruled docs-pass shape).
2. 4.6/caller decision: optional one-line README count fix 550 → 558 (N3).
3. Release-artifact rebuild + `ls -l` cross-check: the global-plan Step-5/4.6 close-out duty (production code changed — artifacts in `dist/` predate the branch); explicitly OUT of Task-2 scope per plan §6.
4. Operator-run live diagnostic-file inspection for the real reconnect-label evidence: by design never agent-verified.

**FINAL: Path of this report is the deliverable; verdict ADHERENT WITH ACCEPTABLE DEVIATIONS; REQUIRED fixes: none.**

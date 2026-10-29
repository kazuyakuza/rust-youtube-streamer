# Phase 07 Task 4 — 4.3 Simplification Plan (code-simplifier pass)

> Binding authority: `.kilo/plans/20261029-phase07-task4-verification-docs.md` (Task-4 plan, §A.7/§C/§F-4.3); TODO `.agent/todos/20261029/20261029-todo-1.md` Task 4.
> Reviewed tree: HEAD `c160ad5`, tree clean, gate 494/0 (`logs/checks/20261029T203729Z.log`).
> Scope reviewed: `src/app/run/chat_worker_burst_tests.rs` (76 lines), the `chat_worker.rs` registration block (file 187 lines), the Task-4 docs set (`production-run.md`, `google-cloud-prerequisites.md`, `configuration.md`/`authentication.md` drift-checks, `README.md`, `project-structure.md`, `tech.md`, `context.md`), and `.agent/reports/20261029-phase-07-streamlist-migration.md`.
> Verdict: NO duplication fold (see §3). TWO edits required: S1 (one code rule-compliance extraction) and D1 (one doc-accuracy repair). Everything else is NO-EDIT.

---

## 1. Evidence matrix (what was verified, all PASS unless stated)

| Area | Check | Observed | Verdict |
|---|---|---|---|
| burst roster | helper reuse vs `runtime_chat_fakes.rs` | uses the existing `ChatFakes::new`, `script_token_batch_session`, `worker_plan`; `chat_channel`/`run`-side helpers `lookup_found`/`drain`/`terminal_of` from `chat_worker_tests`; crate-root fixture builders `response_plain`/`response_offline`/`text_item`/`yielded` — nothing already-helperized is reinvented | PASS |
| burst roster | full-cap prefill helper existence | `git grep -rn "prefill\|filler" src` and `git grep -n CHAT_CHANNEL_CAPACITY src`: the ONLY test-side hits are the burst roster itself (+ the const def in `chat_worker.rs`). No pre-existing prefill/full-capacity helper exists — nothing was overlooked | PASS (no fold) |
| burst roster | 7-line session script similarity to `chat_worker_flow_tests.rs:21–27` | inline in both rosters; NOT a shared helper; the two scripts differ in item names (`m1`/`m2` vs `burst-1`/`burst-2`) and run shape (`run_worker` inline vs `spawn_chat_worker` + drain loop). A fold into a new shared helper would fail the name/count-invariance requirement and would rewrite a Task-3-landed byte-stable roster | LEAVE-AS-IS |
| burst roster | file/line caps | 76 lines ≤ 120 target ≤ 200 hard cap | PASS |
| burst roster | method body length | test fn body = 58 lines (18–75) — EXCEEDS the 50-line body rule (`.kilo/rules/max-lines-per-method.md`) | **S1** |
| `chat_worker.rs` | registration block pattern | `#[cfg(test)]` + `#[path = "chat_worker_burst_tests.rs"]` + `mod chat_worker_burst_tests;` immediately after the budget block — byte-shaped identical to the five sibling blocks; file 187 ≤ 200 | PASS |
| `chat_worker.rs` | no other change | only the 4-line block was added (commit `7ab6eee`, 2 files) | PASS |
| `production-run.md` | quota text vs live-chat section drift | the quota numbers (5 units/call, 1-unit resolution, 10,000/day, ≤ 11 calls ≤ 55 units) appear ONCE in the doc (Quota subsection, lines 186–194) and mirror — without contradicting — `google-cloud-prerequisites.md:140`, `tech.md:29/30`, report §5, context.md Task-4 bullet | PASS (see §4 watch-note) |
| `production-run.md` | bound-facts triple ("10 consecutive transient failures / 60 s cap / 1 s stop-slices") | three occurrences (Diagnostics ¶163, Recovery-table ¶174–178, step 6b ¶333) — all consistent with the code constants (`CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES = 10`, `CHAT_BACKOFF_MAX_MS = 60_000`, `CHAT_SLEEP_SLICE_MS = 1_000`) and each carries its own operator framing; plan-encoded (§C.a.10/§C.a.11/§D) | LEAVE-AS-IS (§4 watch-note) |
| `production-run.md` | polling-era residue | `poll` hits are ONLY the readiness/confirmation-gate waits (lines 51/53) and the honest "no poll interval exists at all"/supersession statements (144/148/150/194); no doc claims chat polling is selected | PASS |
| README vs context.md | test count + versions + sizes | 494 (README:47, context.md:41/326, report:101/107/149/178) consistent; v0.13.0 consistent; `tokio 1.53.1`/`tonic 0.13.1`/`prost 0.13.5` verified against `Cargo.lock` (867–868, 1315–1316, 1254–1255); Production Run sentence (README:121) carries no polling claim; H2 anchor `#live-chat-behavior-diagnostics-recovery-privacy-quota` still resolves (H2 unchanged) | PASS |
| `project-structure.md` | re-cut vs the real tree | `git ls-files src/app/run` = 31 (13 sources + 4 fixtures + 14 rosters — matches line 11); run rosters `git grep -c "#\[test\]" src/app/run` sums 58 with per-roster counts identical to line 26 (burst 1, flow 6, budget 8, worker 6, gate_chat 2, chat_failure 2, chat_source 5); `git ls-files src/youtube` = 92 (47 + 4 + 41 — matches line 54); youtube `#[test]` sum = 231; `live_error` = 19 variants (verified by grep of `src/youtube/live_error/mod.rs`); `chat_worker_delivery` row absent; the seven stream-family sentinel rows present (lines 70, 87–93); dead-code inventory (line 151) matches the L.6 grep (8 + 1, no comment mention) | PASS |
| completion report | §C.i 15-section skeleton | all 15 headings present in the exact encoded order (`## Overview` … `## Evidence Index`) | PASS |
| completion report | number freshness | gate 494 everywhere (the 493/507 mentions are correctly framed as the historical baselines/logs); 12336636/7665712 consistent in §9 table and §10; "493 filtered out" consistent with the 493 baseline; sizes have no comma/no-comma contradiction (both forms render the same integers) | PASS |
| context.md | Phase-07 dated block | Task-3 bullet (line 40) carries the landed-plan-template fragment **"29 → OBSERVED run-roster recut"** — the OBSERVED slot was never filled; a reader cannot recover the value from the record | **D1** |

---

## 2. Edit S1 — CODE — bring the burst-test body under the 50-line rule (single local-helper extraction)

File: `src/app/run/chat_worker_burst_tests.rs` ONLY. Invariants: test name, the 1-test roster count, every assertion, every message string, gate total 494 — all unchanged. NO new `allow(`. NO import changes needed (the new signature uses the fully-qualified `std::sync::mpsc::Receiver`, the sibling `drain` precedent at `chat_worker_tests.rs:21`).

**S1 step 1 — insert this helper** directly between the `use super::chat_worker_tests::{drain, lookup_found, terminal_of};` line and the `#[test]` attribute:

```rust
/// Blocking-drains the receiver until `total` messages are taken — the
/// capacity-release mechanism the retain-retry spins against (zero sleeps).
fn drain_to_total(
    receiver: &std::sync::mpsc::Receiver<ChatMessage>,
    total: usize,
) -> Vec<ChatMessage> {
    let mut drained = Vec::new();
    while drained.len() < total {
        drained.push(
            receiver
                .recv()
                .expect("the pinned burst drains message by message"),
        );
    }
    drained
}
```

**S1 step 2 — replace this block** (the current lines 39–48, the `let total` statement through the closing `}` of the `while` loop):

```rust
    let total = CHAT_CHANNEL_CAPACITY + 2;
    let mut drained: Vec<ChatMessage> = Vec::new();
    while drained.len() < total {
        drained.push(
            channel
                .receiver
                .recv()
                .expect("the pinned burst drains message by message"),
        );
    }
```

**with this single line:**

```rust
    let drained = drain_to_total(&channel.receiver, CHAT_CHANNEL_CAPACITY + 2);
```

Nothing else in the file changes: `handle.join()` line, both assert loops, `drained[CHAT_CHANNEL_CAPACITY]` / `drained[CHAT_CHANNEL_CAPACITY + 1]` indexing, the `terminal_of`/`drain` asserts — all byte-stable.

**S1 step 3 — verification (exact commands, container):**
1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` → exit 0 (if it fails, run `cargo fmt` in the container and re-check; keep the diff inside the two S1 zones).
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked chat_worker_burst` → exit 0, `1 passed; 0 failed`.
3. FULL gate `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0, OBSERVED `494 passed; 0 failed` (record the fresh log path; never hardcode).

Post-edit arithmetic: test fn body = 49 lines ≤ 50; file total ≤ 120 target.

---

## 3. Explicit NO-EDIT decisions

1. **No prefill-helper fold into `runtime_chat_fakes.rs`**: the prefill loop exists ONLY in the burst roster (grep-proven); folding would push test-shape choices (`prefill-{index}`/`capacity` strings) into the shared fakes module and change nothing's behavior — zero dedup benefit, shared-surface growth. Against the "genuine duplication" bar.
2. **No fold of the two-sessions script** (`script_token_batch_session` + the 2-item live session): already uses the shared helper for the part that is shared (`script_token_batch_session`); the remainder is one `stream_ledger.script_session(vec![…])` call — the ledger IS the abstraction, and the sibling flow roster carries the same inline shape with different item names.
3. **No change to the registration block**: sibling-identical pattern, position per plan, cap holds.
4. **No doc rewording** beyond D1 — citation/research wording is docs-specialist-owned.

## 4. Report-only watch-notes (NOT for this pass to act on)

- W1: the 10-failures/60 s/1 s bound triple in `production-run.md` is a 3-site drift surface if a constant ever changes; currently accurate against the code. A future constants change should re-touch all three sites.
- W2: the gate-count integer 494 lives in README:47, context.md:41/326 and the report — any later test addition at step 5 drifts four texts; the docs pass (§F 4.4) already owns this sweep.

## 5. Edit D1 — DOC — fill the dangling OBSERVED placeholder in `context.md`

File: `.agent/project-info/context.md`, line 40 (Task-3 bullet inside the `## Phase 07 Tasks 2–4 + closure` block). `context.md` is planner/docs-owned: apply this via the pass the caller designates (4.4 docs-specialist or a caller-authorized metadata commit) — the junior implementer must not touch it without that direction.

Anchor (unique in the file): `the Task-2 staging ledger closed; 29 → OBSERVED run-roster recut. Gate 493/0`

Replace with: `the Task-2 staging ledger closed; the run rosters recut to the observed 13 rosters carrying 57 tests (verified at commit `ca9c55f`). Gate 493/0`

Basis (observed, this pass): `git grep -c "#\[test\]" ca9c55f -- src/app/run` → 13 roster files, sum 57 (the 14th roster + the 58th test is Task 4's burst pin, recorded in the Task-4 bullet). Nothing else on the line changes; all other context.md zones byte-stable.

## 6. Execution order + commit guidance for the junior implementer

1. S1 steps 1–3 (code + verification), commit: `refactor(test): extract the burst-pin drain loop to a file-local helper (max-lines-per-method)` — `src/app/run/chat_worker_burst_tests.rs` ONLY.
2. D1 (docs), commit per the designated docs pass: `docs: fill the Task-3 run-roster observed recut value in context.md` — `.agent/project-info/context.md` ONLY.
3. NO push. NO TODO edits. NO staging of `dist/` or `logs/`. If any gate/build step fails: STOP and surface to the caller.

# Simplification Plan — Task 2 (made_for_kids): 4.3 code-simplifier output

Plan date: 2026-10-23 · Author: Code-Simplifier Agent (Critical Workflow step 4.3, SIMPLIFICATION half)
Scope reviewed (Task-2-changed files ONLY, per commits `7fb1cd4` + `40e506f`):
`src/config/model.rs`, `config/config.example.json`, `src/config/test_fixtures.rs`, `src/config/loader_tests.rs`, `src/config/broadcast_made_for_kids_parse_tests.rs`, `src/config/loader.rs` (wiring), `src/youtube/live_models/commands.rs`, `src/youtube/live_models/request_bodies.rs`, `src/youtube/live_models/mod.rs` (wiring), `src/youtube/live_models_tests.rs`, `src/youtube/google_live_api_transport_tests.rs`, `src/youtube/live_ingestion.rs`, `src/youtube/live_ingestion_tests.rs`, `src/youtube/live_models_made_for_kids_tests.rs`, `src/youtube/live_ingestion_made_for_kids_tests.rs`; plus the committed docs of `40e506f` (report-only review).
Binding inputs: `.kilo/plans/20261022-mfk-task2-implementation.md` (D1–D9, S0–S18, §8 ledger), global plan `.kilo/plans/20261022-run-streaming-diagnostics-mfk.md`, HEAD `40e506f`, baseline gate 378 passed / 0 failed (`logs/checks/20261023T030824Z.log`).
Caller rulings respected (do NOT undo): the `assert_eq!(broadcast.made_for_kids, Some(false));` in `live_ingestion_tests.rs`, the fmt re-wrap, D2's example value `false` + fixture lock.

---

## 1. Verdict summary

- **1 edit to APPLY** (in `src/youtube/live_ingestion_made_for_kids_tests.rs`): the two plan-named tests are near-clones — 26 of 43 lines are byte-identical setup differing only in the declared bool. Fold the shared flow into one local 1-arg helper; both tests keep their S11 names, counts, fixtures and assertion semantics. Exact full-file content in §2.
- Everything else in the changed set is **ADJUDICATED LEAVE-AS-IS** (§3). Notably: the duplicated full-body `json!` expectation blocks in `live_models_made_for_kids_tests.rs` are the binding S10/§8 wire-pin form — folding them would DERIVE the expected value instead of pinning it literally, weakening the byte-stability evidence. They stay.
- **0 deferred big items requiring caller action now; 2 future-work candidates recorded** in §4 (report-only; no TODO file created per the overwrite-todo-file-prevention rule).
- Caps re-verified clean at HEAD: `loader_tests.rs` 199 (guardrail: untouched by this plan), `live_models_tests.rs` 197, `google_live_api_transport_tests.rs` 195, `live_ingestion_tests.rs` 188, `model.rs` 147, `request_bodies.rs` 125, `live_ingestion.rs` 162, new rosters 79/86/43. Post-fold file projects to 36 lines.
- No new `#[allow(` exists or is introduced (git grep confirmed). No allocations to trim: the new field is `Option<bool>` (Copy) — the S7/S9 plumbing lines are moves, not clones; no narrowing-visibility opportunity exists (each new field matches its sibling's visibility exactly).

## 2. Step to APPLY (junior-executable; NO judgment calls required)

The 4.3 pass could not edit `src/**` (permission-blocked); this file IS the handoff. One step, one file.

### Step 1 — fold the duplicated command-pin setup in `src/youtube/live_ingestion_made_for_kids_tests.rs`

Replace the ENTIRE current content of `src/youtube/live_ingestion_made_for_kids_tests.rs` (43 lines) with exactly:

```rust
//! Made-for-kids plumbing pins: the optional per-stream declaration flows
//! verbatim from the validated config into the captured O2 command, both
//! for true and false. All against the shared scripted transport fake; no
//! network, no Google.

use crate::config::valid_config;
use crate::youtube::live_test_fixtures::*;

use super::live_ingestion_tests::{queue_preparation, FIXED_NOW_EPOCH_SECS};

use super::*;

#[test]
fn declared_true_flows_from_config_to_the_broadcast_command() {
    assert_eq!(captured_declaration_for(true), Some(true));
}

#[test]
fn declared_false_flows_from_config_to_the_broadcast_command() {
    assert_eq!(captured_declaration_for(false), Some(false));
}

fn captured_declaration_for(declared: bool) -> Option<bool> {
    let mut config = valid_config();
    config.youtube.broadcast.made_for_kids = Some(declared);
    let transport = ScriptedLiveApiTransport::default();
    queue_preparation(&transport);
    LiveBroadcastPreparer::from_config(&config, FIXED_NOW_EPOCH_SECS)
        .prepare(&transport, &WaitCanceller::new())
        .expect("the happy script prepares");
    let captured_broadcast = transport.captured_create_broadcast.borrow();
    captured_broadcast
        .as_ref()
        .expect("the O2 command was captured")
        .made_for_kids
}
```

Do NOT touch any other file. Do NOT rename the two `#[test]` functions (they are the plan-S11 names). The content above is already rustfmt-canonical for the project's default settings (no `rustfmt.toml` in repo root; default `max_width=100`, `chain_width=60`: the 79-char return chain splits per element, the 74-char `let` stays one line).

Why this is behavior-identical: same config source (`valid_config()` + `made_for_kids = Some(declared)`), same scripted fake and script (`queue_preparation`), same clock (`FIXED_NOW_EPOCH_SECS`), same prepare call, same captured-O2-command assertion (`Some(true)`/`Some(false)`); the `bool` parameter is only re-wrapped as `Some(...)` at the config-set line — the pin semantics (declared value flows verbatim to the command) are unchanged. The helper returns the Copy `Option<bool>` while the `Ref` guard is alive — no borrow issue (the guard drops at end of the helper; the returned value is owned Copy).

### Verification after applying (exact commands)

Container commands go through the Alpine-VM MCP: call `vm_status` FIRST, then `alpine-vm_vm_run_command`, ONE command per call (no chains), all `--locked`, offline:

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` → expect exit 0.
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked made_for_kids` → expect `running 9 tests` / `test result: ok. 9 passed; 0 failed` (module-path filter; the two folded tests keep their module path so the count is unchanged).
3. Full standard gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → expect exit 0, `ALL CHECKS PASSED`, new log recorded under `logs/checks/`, and the OBSERVED test count — expect 378/0 (count identical to baseline; observed governs).
4. Host, workdir `C:\repo\rust-youtube-streamer`: `git status --porcelain` → expect exactly ` M src/youtube/live_ingestion_made_for_kids_tests.rs` plus the new untracked gate log under the ignored `logs/checks/` (nothing else; gitignore-compliance — never stage `dist/`, `logs/`, `config/config.json`, `credentials/*`, `target/`).

If any step fails: revert the single file (`git restore src/youtube/live_ingestion_made_for_kids_tests.rs`) and report; do NOT improvise other edits.

### Commit (only when the caller confirms — repo style, short imperative)

Suggested message (matches the Phase-03 `2d99ebf` / HTTP-411 `b76a0c8` simplification-commit precedent):

    refactor: fold the duplicated made-for-kids pin setup into a helper

Stage ONLY `src/youtube/live_ingestion_made_for_kids_tests.rs`.

## 3. Reviewed and LEAVE-AS-IS (adjudicated, with reasons)

1. **`live_models_made_for_kids_tests.rs` repeated `json!` expected blocks** (≈13 identical `snippet`/`contentDetails` lines ×3): binding plan form — S10 mandates "the FULL exact `serde_json::to_value(...)` JSON" per test and §8 cites these literals as the byte-stability evidence for the absent/true/false wire shapes. A shared expected-body builder would derive the expectation from option-shape logic, defeating the pin's purpose (a golden literal must not be computed). The COMMAND fixture is already folded (`broadcast_command(Option<bool>)`). No change.
2. **`broadcast_made_for_kids_parse_tests.rs` local helper copies** (`write_temp_config`/`expect_load_error` mirror `loader_tests.rs` shapes): forced — `loader_tests` is a private `mod`, its helpers are not importable, and the "do NOT merge the three new rosters into the existing ones" guardrail forbids restructuring. The roster is already fully helper-ized (`example_value`, `load_modified`, `broadcast_of` kill every repetition within it); the wrong-type test's 2-line temp-dir sequence has exactly one call site — a second helper would be over-abstraction. The `value.to_string()` serialization is inherent to writing a temp config file (the `write_temp_config(dir, contents)` 2-arg shape is the plan-S5 copy; swapping to `serde_json::to_writer` would re-shape it for zero readability gain). No change.
3. **Existing-roster `CreateBroadcastCommand` literals** (`live_models_tests.rs:~155`, `google_live_api_transport_tests.rs:~162` — the same five fixture strings a third time): folding would edit two existing 197/195-line cap-edge rosters across the roster-merge boundary. The Task-2 diff there is the compile-required one-liners only. Guardrail. No change (see §4 C2).
4. **Visibility**: no narrowing available. `BroadcastConfig::made_for_kids` `pub` = sibling fields (cross-module reads); `CreateBroadcastCommand::made_for_kids` `pub(crate)` = sibling fields (D3); `BroadcastCreationStatus::made_for_kids` private = sibling `privacy_status` (D4, serde-pinned). New wiring `mod` declarations are private `#[cfg(test)]`. The `pub(super)` sibling helpers reused by the folded file are pre-existing (not widened by Task 2). No change.
5. **Allocations**: plumbing lines are `Copy` moves (no `.clone()` added anywhere in `7fb1cd4`); `.to_string()` on fixture `&str` in test builders is idiomatic and pre-existing pattern. No change.
6. **Pinned serde attrs**: `#[serde(rename = "selfDeclaredMadeForKids", skip_serializing_if = "Option::is_none")]` is REQUIRED (D4) — not touched. O1/O3/O4/O5 forms not touched. `validation.rs` not touched (21 rules stand).
7. **`src/config/loader_tests.rs` (199 lines)**: guardrail — not touched beyond its committed compile-required literal line; this plan performs zero edits there.
8. **Doc comments in `model.rs` / `commands.rs` / `live_ingestion.rs`**: verbatim binding-plan strings (S1/S6/S9). No change.
9. **`request_bodies.rs` grounding line** (S7): not strictly verbatim in the plan; factually correct as written ("the optional `status.selfDeclaredMadeForKids` status field…"). "status … status field" is a mild wording redundancy — cosmetic-only edit of a committed, fmt-stable file; NOT applied here (no factual error → report-only per task rules).
10. **Committed docs (`40e506f`)** — report-only: `docs/configuration.md` bullet and both `docs/production-run.md` additions match the adjudicated S17 text and the §1 research record; every claim verified factual (insert-settable, absent = never sent, non-boolean rejected at deserialize time). NO wording fix required; tightening stays with 4.4/4.5b if the caller wants it.
11. **Ruled items**: `live_ingestion_tests.rs` `assert_eq!(broadcast.made_for_kids, Some(false));` (caller-ruled), the fmt re-wrap (caller-ruled), `config.example.json`/fixture `false` lock (D2): not touched.

## 4. Deferred candidates → future work (report-only; do NOT execute here)

- **C1 — cross-roster command-fixture module**: the `CreateBroadcastCommand` fixture strings now exist in 3 files (§3.3). A shared test-fixture module would de-duplicate them but merges rosters across the guardrail boundary and edits 197/195-line cap-edge files. Medium refactor → needs caller/architector planning if ever scheduled; per the HTTP-411 precedent this duplication IS the current chosen form.
- **C2 — `live_test_fixtures` growth**: `ScriptedLiveApiTransport` capture-assertion sequences (borrow → as_ref → expect) repeat per pin-test by design. Generalizing into assertion helpers on the fake itself is a test-infrastructure redesign touching Task-1-era byte-stable rosters → architector scope, not a simplification pass.

## 5. Boundaries honored by this plan

Only `src/youtube/live_ingestion_made_for_kids_tests.rs` is edited. No roster merge, no existing-file touch, no serde-attr change, no `loader_tests.rs` change, no `validation.rs` change, no O1/O3/O4/O5 change, no Cargo/.gitignore/`src/app`/`src/streaming`/docs touch, no metadata touch (structure-map counts for this file change 43 → 36 lines only — planner-owned metadata step folds it in), no `allow` additions, no commented code, offline tests only, test names and counts unchanged (9 `made_for_kids` filter tests before and after; gate expectation 378/0 unchanged).

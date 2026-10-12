# Code Simplification Plan — Phase 03, TODO Task 3 (sub-step 4.3)

- **Date**: 2026-10-11
- **Caller step**: TODO Task 3, sub-step 4.3 (Code Simplification; code-simplifier)
- **Branch / HEAD reviewed**: `feat/phase03-ffmpeg-local-pipeline` @ `7c840b6`; reviewed code = commits `a0f27f4` (demo command + pipeline demo wiring, merged per ruling D1) + `eaa12dc` (staging-allow retirement, item-level allowances, cfg(test) gates)
- **Scope**: ONLY `### 3. Dedicated demo Command` of `.agent/todos/20261011/20261011-todo-1.md` — the target files listed in the task prompt. Task-2 or earlier code inside those files that Task 3 did not introduce or change is OUT of scope.
- **Governing reference**: `.kilo/plans/20261011-phase03-task3-demo-command.md` + the planner ruling ledger D1–D5, the six `cfg(test)`-gated items, the three sanctioned item-level allowances. NONE of those are churned below (see §3 for the full considered-and-rejected register).
- **Implementer target**: junior, 50% restriction — every edit below is given verbatim; execute exactly, decide nothing.

## 1. Assessment evidence (live tree, not assumed)

- Rule audit of the Task-3 surface (all PASS, no action):
  - File length ≤200: `src/app/mod.rs` 177, `src/app/cli.rs` 153, `src/app/demo/schedule.rs` 121, `src/app/demo/mod.rs` 107, `src/app/demo/source.rs` 57, test files ≤150. All within cap.
  - Method body ≤50 lines, nesting depth ≤2, params ≤2: every Task-3 function complies (`next_message` 13 lines depth 1; `demo_outcome` 7 lines; `build_deliveries`/`scheduled_message` 2–8 lines).
  - Single-section boolean conditions: `due_frame_index <= frame_index`, `frame_index >= final_frame_index()`, `is_long_line_slot(number)` — all single-section.
  - No commented-out code anywhere in `src/`.
  - Prefer-private: `served`, `current_frame_index`, `timeline`, `stop_flag` private; `DemoError` variants `pub(super)` only as needed.
- Ledger verification (all PASS, no action): exactly three `#[allow(dead_code)]` remain (`chat/store.rs:123` `is_empty`, `chat/model.rs:44` `with_timestamp`, `streaming/video_output.rs:27` `Session`) = the sanctioned set; all four module staging blocks gone; re-export blocks present as adopted (D2); `DemoTimeline::fixed_timeline()` with no `interval_nanos` field (D3) applied; test-module placement per D4 (tests declared in owning file, `#[path]` for out-of-file test modules); `rg -U -c "#\[test\]"` sums to exactly **138** (D5).
- `pipeline/test_helpers.rs` is NOT touched by `a0f27f4` or `eaa12dc` (neither commit stat lists it) and stays at its 200-line ceiling; this plan adds nothing there. Task-3 fakes correctly live inside the demo test files.
- Source-vs-pacing dedup candidate is already resolved by ruling D3: the schedule holds no interval/nanosecond state; `interval_nanos` remains only in `pipeline/pacing` (its production home). No leftover duplicate to remove.
- Behavior-locked surfaces untouched by this plan: CLI usage strings (pinned in `cli_tests.rs`), exit-code mapping (2/1/3 per ruling), demo println text (`demo recording finalized: …`), schedule constants 28/30/15/60 + authors cycle + long-line #15, one-message-per-frame consumption rule.

Two semantics-preserving simplifications remain warranted (§2). No others are.

## 2. Steps (execute in order; one commit at the end)

### Step 1 — Inline the unnecessary `served_count()` redirect (`src/app/demo/source.rs`)

The private accessor returns `self.served` unchanged and has exactly ONE call site inside the same `impl`. It is a pure indirection (the house "unneeded helper redirect" category); `due_at`'s parameter is already named `already_served`, so the call site stays self-documenting after inlining. Semantics: identical value, same call order, same ownership.

Edit 1a — in `impl DemoMessageSource`, DELETE this block (keep the `new` constructor untouched):

```rust

    /// Consumer-side accounting for `due_at`.
    fn served_count(&self) -> usize {
        self.served
    }
```

Edit 1b — in `next_message`, replace the single line:

```rust
        let due = self.timeline.due_at(self.served_count(), frame_index);
```

with:

```rust
        let due = self.timeline.due_at(self.served, frame_index);
```

No other change in the file. Expected size: 57 → 52 lines (5 deleted, 1 line replaced in place). The doc block at the top of `source.rs` and the `is_end` naming are ruling-locked — do NOT touch.

Gate 1 (Alpine VM MCP, single command):
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked`
EXPECT: exit 0; `running 138 tests`; `138 passed; 0 failed; 0 ignored`.

### Step 2 — Derive the unpinned `435` literal in the source test (`src/app/demo/source_tests.rs`)

`timeline_delivers_all_messages_in_order_without_any_clock` hardcodes `0..=435` (the last due frame). 435 is a derived value, not a planning anchor — the planning formula 495/496 is already pinned where it belongs, in `schedule_tests.rs::final_frame_index_and_rendered_count_match_the_planning_formula`. Deriving it from the same deterministic production schedule preserves current behavior exactly (`last_due_frame == 435`) and removes the only magic number in the Task-3 files; it also dedups the second `scheduled_messages_for_tests()` call in the test.

Replace the whole test (lines starting `#[test]` through the closing `}` of that function) with:

```rust
#[test]
fn timeline_delivers_all_messages_in_order_without_any_clock() {
    let (mut source, _stop_flag) = source_with_stop_flag();
    let deliveries = scheduled_messages_for_tests();
    let expected: Vec<String> = deliveries
        .iter()
        .map(|entry| format!("{}", entry.message))
        .collect();
    let last_due_frame = deliveries
        .last()
        .expect("non-empty schedule")
        .due_frame_index;
    let mut delivered = Vec::new();
    for _frame_index in 0..=last_due_frame {
        if let Some(display) = next_display(&mut source) {
            delivered.push(display);
        }
    }
    assert_eq!(delivered, expected);
}
```

Test name, count and assertion set are unchanged; imports unchanged (`scheduled_messages_for_tests` already imported; do not touch the `LEAD_IN_FRAMES` import — other tests use it).

Gate 2 (final tree, full standard gate; single command):
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
EXPECT: exit 0; PASS lines `fmt-check`, `check`, `test`, `clippy`; `running 138 tests` → `138 passed; 0 failed; 0 ignored`; summary `ALL CHECKS PASSED`.

## 3. Commit point (exactly one)

Stage ONLY: `src/app/demo/source.rs`, `src/app/demo/source_tests.rs`.
Verify `git status` first (must show only these two modified plus untracked planning file).
Commit message (locked, single logical change — demo-source accounting simplification):

`refactor: drop the demo served-count indirection and derive the test end frame`

No push. No TODO edits (owned by the caller). Do not amend `a0f27f4`/`eaa12dc`.

## 4. Considered and deliberately NOT proposed (no-churn register)

1. **D1** merged commit shape — ruling; do not propose splitting.
2. **D2** re-export blocks (production + cfg(test) split) in `chat/mod.rs`, `streaming/mod.rs`, `renderer/mod.rs`, `app/pipeline/mod.rs` — adopted as-is; the applied set matches the ruling.
3. **D3** `interval_nanos` drop + `for_config` → `fixed_timeline` — already the simplification itself; nothing left over (verified no schedule-side interval state anywhere).
4. **D4** house test-module placement (`schedule_tests`/`source_tests` declared in the owning file, `mod_tests` via `#[path]`) — adopted.
5. **D5** 138 tests — verified by count; both steps keep 138.
6. **Six cfg(test) gates + three item-level allowances** — ledger-settled; touching them (e.g. merging the duplicate "Test-only accessors" comments in `frame.rs`, or gating `due_at`/`final_frame_index`) is churn.
7. **`DemoError::exit_code()` constant branch** — plan-locked T3 mapping table, pinned by `mod_tests` test 1.
8. **`build_demo_source`** — real composition (constructs timeline + source), single meaningful abstraction, ≤2 params; keeping it matches the plan.
9. **`is_end` / raw-index ordering in `next_message`** — LOCKED semantics of plan §4.3; off-by-one governance.
10. **`temp_store()` duplication between `mod_tests.rs` and `runner_tests.rs`** — deliberate per plan §8.1; sharing it would push `test_helpers.rs` (at its 200-line ceiling) over, which is forbidden. Confirmed as a Task-2 watch item, not a Task-3 defect.
11. **Literals `495`/`496` (schedule_tests) and `25` newlines / `26` capacity (mod_tests, schedule_tests)** — intentional planning-formula pins per plan §8.1/§8.2; the source test in §2 Step 2 is different: it is an iteration bound, not a pin.
12. **`at_most_one_message_is_delivered_per_call` inline fixture `due_frame_index: 5` / `0..5`** — self-contained scenario values, not production-derived; extracting constants here would obscure the fixture.
13. **Doc wording "classes through the renderer's horizontal clipping path" (`schedule.rs:16`)** and the `≤2-arguments rule` constructor note — verbatim adopted plan text, substantively accurate; wording edits would be ledger churn.
14. **`renderer/frame.rs` `blend_at` three-channel triplication, `settings.rs` internals** — Task-2 code untouched by the two reviewed commits (only their cfg(test) gates are Task-3); out of scope.
15. **`main.rs`, `cli.rs` parse loop, `app/mod.rs` funnel** — verified compliant and minimal; no proposal.

## 5. Risk / rollback

- Both steps are local (`source.rs` accessor, one test body); no trait, signature, visibility, or file-level change; clippy/fmt surface unaffected beyond the removed method.
- If Gate 1 or Gate 2 fails, STOP and report — do not improvise further edits.
- Rollback = `git restore src/app/demo/source.rs src/app/demo/source_tests.rs` pre-commit, or revert the single commit post-commit.

[PLAN-CHECK-END]

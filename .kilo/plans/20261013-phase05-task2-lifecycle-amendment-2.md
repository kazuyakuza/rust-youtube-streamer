# Phase 05 — Task 2 — Amendment 2 (Planner ruling: readiness-wait line-cap re-cut + widenings)

- Date: 2026-10-14
- Amends: `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md` (+ Amendment 1)
- Trigger: implementer S4 blocker — the mandated `readiness_wait.rs` surface (262 lines
  rustfmt-normalized) and `readiness_wait_tests.rs` (282 lines) cannot satisfy the standing
  200-line hard cap (`.kilo/rules/max-lines-per-file.md`) even at maximal mechanical compaction.

## Ruling 1 — Option B: planner-directed re-cut (cap waiver rejected)

The 200-line cap is a standing project rule, already honored under identical pressure in Task 1
(`live_models/`, `live_error/` directory splits). No waiver. Adopt the Task-1 directory-submodule
pattern:

- `src/youtube/readiness_wait.rs` → directory `src/youtube/readiness_wait/`:
  - `mod.rs` — module docs, re-exports, `ReadinessWait` core (constructor, `#[cfg(test)]
    with_polling` re-injection, attempt-bounded `run`, the typed-const budget contract).
  - `confirmation_wait.rs` — `wait_until_stream_active` + post-start confirmation constants
    (120 s → 24 attempts) + its `#[cfg(test)]` re-injection seam.
  - `observation.rs` — `StreamObservationRequest`, the `stream_observation`/`broadcast_observation`
    mapping helpers, and the transient/fatal attempt matrix.
  - Every file ≤200 lines; all other code rules continue to hold (≤50-line bodies, ≤2 depth,
    ≤2 args, privacy, no commented code).
- `src/youtube/readiness_wait_tests.rs` → split across TWO test siblings, no test dropped,
  renamed, or semantically altered; all 12 roster-C tests keep identical names/assertions:
  - `readiness_wait_tests.rs` — the wait-loop tests that fit ≤200 lines.
  - `readiness_wait_confirmation_tests.rs` — the confirmation-wait tests (sibling split
    precedent: Task 1's `live_error_display_tests.rs`).
  - Both import the shared fakes from `src/youtube/live_test_fixtures.rs` (Amendment 1);
    record the split in each file's doc header.
- The plan's F-T2-3 line projection (≈150) is superseded by this layout; API shapes, constants
  (180 s/5 s → 36; 120 s → 24), mapping semantics, and the roster are unchanged.

## Ruling 2 — compiler-forced `LiveBroadcast` widenings: ACCEPTED

`LiveBroadcast.id`, `LiveBroadcast.status`, `LiveBroadcast.content_details` → `pub(crate)` are
CONFIRMED as compiler-forced completions required by the plan's own decisions D-T2-8/D-T2-9 and
the green-at-each-point binding rule. Recorded in the implementer's summary; no further action.

## Unchanged

Everything else stands (transport design, polling constants, cancellation seam, RTMPS boundary,
S9 retirement act with clippy-arbitrated re-staging fallback, verification commands, commit
plan otherwise).

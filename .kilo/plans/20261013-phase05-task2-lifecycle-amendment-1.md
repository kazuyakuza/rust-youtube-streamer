# Phase 05 — Task 2 — Amendment 1 (Planner ruling on shared-fake topology + S4/S5 merge)

- Date: 2026-10-14
- Amends: `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
- Trigger: implementer S4 blocker — the roster-C test import (`use super::live_ingestion_tests::*`)
  cannot resolve at S4 because the shared `ScriptedLiveApiTransport` host file is parented by
  `live_ingestion.rs`, which lands at S8; and S5's model-visibility widenings are required for
  S4 to compile green.

## Ruling 1 — shared fake moves to a `#[cfg(test)]` test-fixtures file (new Option D)

Adopt the project's own shared test-support precedent (`src/config/test_fixtures.rs`,
`src/app/pipeline/test_helpers.rs`) instead of the credential-service sibling-under-one-parent
topology for this fake:

- New file: `src/youtube/live_test_fixtures.rs` — `#[cfg(test)]`-gated shared test-support
  module hosting `ScriptedLiveApiTransport` (moved verbatim from the plan's F-T2-9 home in
  `live_ingestion_tests.rs`).
- Wired from `src/youtube/mod.rs` as a `#[cfg(test)]` module (same wiring style as
  `src/config/test_fixtures.rs`).
- `readiness_wait_tests.rs` (S4) and `live_ingestion_tests.rs` (S8) both import from it; the
  roster's literal `use super::live_ingestion_tests::*` import is superseded by the fixtures
  path (record the replacement in the test-file docs).
- Rationale: satisfies "C-tests at S4 + green" AND green-at-each-point without the confusing
  cross-parenting of option B and without rewrite churn of option C; precedent-clean.
- The file lands at S4 in the same commit as `readiness_wait.rs`; the orchestrator tests
  (B-tests) append their imports from it at S8 as planned. No test is added, removed, or
  renamed by this amendment; roster count stays 49 new.

## Ruling 2 — S4 commit includes S5's model-visibility widenings (confirmed)

The implementer's proposal is CONFIRMED: fold S5's exact `pub(super)` → `pub(crate)`
widenings and the `stream_cdn` accessor additions into S4's commit, so green-at-each-point
holds (the binding invariant). Commit 5 then contains only its remaining S5 content; note the
fold in the commit message body or in the summary. No other S5/S6 content moves.

## Unchanged

Everything else in the Task 2 plan stands (transport design, polling constants, cancellation
seam, RTMPS boundary ruling, retirement act S9 with clippy-arbitrated re-staging fallback,
verification commands, commit plan otherwise).

# Phase 05 — Task 1 — 4.3 Code Simplification Plan (simplifier side)

- Date: 2026-10-13
- TODO authority: `.agent/todos/20261013/20261013-todo-1.md`, Task 1 + global Scope/Boundaries + Explicitly Out of Scope (Live Chat is Phase 06 — untouched here)
- Design authority (binding, unchanged): `.kilo/plans/20261013-phase05-task1-official-api-research-design.md` (§1 fidelity, §4 file plan, §5 test roster, §8 adherence expectations)
- Reviewed tree: `feat/phase05-production-live-runtime` @ `fc610e4`, diff `7769efd..HEAD` (the 4.2 staged design artifacts: 20 files, +1453/−8)
- Baseline gate (authoritative observed figures): `logs/checks/20261014T015119Z.log` — `running 282 tests`, `ALL CHECKS PASSED`
- Verdict: simplification is REQUIRED and CONTAINED — exactly ONE rule-violating item. No architectural change; no API-shape, Display-text, classifier, roster, or allow-inventory change. Not too extensive: nothing moves to a new TODO file.

## 1. Review scope and rule audit result

All 20 changed files were reviewed against the project code rules. Result:

| Rule | Result |
|---|---|
| ≤200 lines/file | PASS — largest new file `live_models_tests.rs` 196 lines |
| ≤50-line method body | **ONE VIOLATION — see §2 S1** (`every_variant_display_stable_and_secret_free`, body = 70 lines). Every other body is ≤46 lines |
| ≤2 nesting depth | PASS (JSON fixtures are data literals, not control-flow nesting) |
| ≤2 args/method | PASS (`formatter` context helpers follow the accepted Phase-03 `StreamingError` precedent) |
| Private-by-default members | PASS (test-only re-exports are `#[cfg(test)]`; response fields are `pub(super)` only) |
| No commented-out code | PASS |
| Self-documenting names | PASS |
| Magic numbers | PASS for the new code except the deliberate precedent-parity items in §3 (not touched) |
| Secret-safety (D-T1-11/D-G5) | PASS — fixtures are fake `.example.test` shapes; Display lines unchanged by this plan; the `://` regression assertions are preserved verbatim |
| `#[allow(dead_code)]` inventory (§8: expected 15) | PASS now (verified: exactly 15 attribute lines in `src/`) and the fix does not touch any allow attribute |
| Test roster (§5: 28 new tests, T3-9 single test) | PASS now and after the fix (no test added, removed or renamed) |

## 2. Required fix S1 — display-contract test body exceeds the 50-line cap

- File: `src/youtube/live_error_display_tests.rs` (only file changed by this plan)
- Problem: `#[test] fn every_variant_display_stable_and_secret_free` (lines 9–81; body lines 11–80 = 70 lines) violates the max-50-lines-per-method rule.
- Fix: extract the 14 inline variant constructions into a private `all_error_variants()` helper and the 14 pinned strings into a private `expected_display_lines()` helper. The test body shrinks to ~14 lines. Semantics, test name, assertion strings, and assertion order stay byte-identical in behavior.
- Expected effects: file 81 → ~110 lines (≤200 stays green); test count unchanged (1 in this file, 282 total); no public surface, no `use`-list change (the helpers live in the same test module).
- Commit (single, feature branch only — no push, no merge, no version bump): `refactor: keep the live api display contract test under the method line cap`

### S1 exact change — replace the ENTIRE content of `src/youtube/live_error_display_tests.rs` with:

```rust
//! Display-contract tests for the typed Live API failures: every variant
//! pins exactly one stable, end-user-reportable line, and no rendered
//! line may ever carry an RTMPS URL, a stream key, or any URL text
//! (the D-T1-11 secret-surface regression pair; sibling of
//! `live_error_tests.rs` under the line-cap rule).

use super::*;

/// One instance of every `LiveApiError` variant, in the pinned line order.
fn all_error_variants() -> Vec<LiveApiError> {
    vec![
        LiveApiError::InvalidMetadata {
            field: "snippet.title".to_string(),
        },
        LiveApiError::LiveStreamingNotEnabled,
        LiveApiError::ChannelLiveBlocked,
        LiveApiError::InsufficientPermissions,
        LiveApiError::AuthorizationRejected,
        LiveApiError::QuotaLike {
            reason: "userRequestsExceedRateLimit".to_string(),
        },
        LiveApiError::BroadcastState {
            detail: "invalidTransition".to_string(),
        },
        LiveApiError::ApiNotFound {
            resource: "liveBroadcast".to_string(),
        },
        LiveApiError::ApiRejected {
            status: 403,
            reason: "brandNewReason".to_string(),
        },
        LiveApiError::BackendUnavailable {
            reason: "backendError".to_string(),
        },
        LiveApiError::Network {
            reason: "connection refused".to_string(),
        },
        LiveApiError::MalformedResponse {
            field: "status.lifeCycleStatus".to_string(),
        },
        LiveApiError::ReadinessTimeout {
            stage: "readiness".to_string(),
        },
        LiveApiError::DestinationMissing {
            field: "rtmpsIngestionAddress".to_string(),
        },
    ]
}

/// The pinned stable Display line for each variant of `all_error_variants`.
fn expected_display_lines() -> [&'static str; 14] {
    [
        "invalid metadata field 'snippet.title'; fix broadcast config and retry",
        "live streaming is not enabled; see youtube.com/features",
        "this channel cannot stream live now; see youtube.com/features",
        "authorization lacks live permissions; run auth again",
        "access token rejected mid-run; run auth again",
        "YouTube quota or rate limit (reason 'userRequestsExceedRateLimit'); retry later",
        "broadcast state forbids this operation (reason 'invalidTransition')",
        "YouTube liveBroadcast not found; deleted or never created",
        "YouTube rejected request (HTTP 403, reason 'brandNewReason')",
        "YouTube backend temporarily unavailable (reason 'backendError')",
        "unable to reach the YouTube API: connection refused",
        "malformed YouTube API response (field 'status.lifeCycleStatus')",
        "YouTube readiness wait timed out (stage 'readiness')",
        "ingestion response missing required field 'rtmpsIngestionAddress'",
    ]
}

#[test]
fn every_variant_display_stable_and_secret_free() {
    let rendered: Vec<String> = all_error_variants()
        .iter()
        .map(ToString::to_string)
        .collect();
    let expected = expected_display_lines();
    assert_eq!(rendered.len(), 14, "every variant pins exactly one line");
    for (index, (actual, wanted)) in rendered.iter().zip(&expected).enumerate() {
        assert_eq!(actual, wanted, "variant #{index} Display line drifted");
        assert!(!actual.contains("rtmps://"), "no RTMPS URL in: {actual}");
        assert!(!actual.contains("://"), "no URL text in: {actual}");
        assert!(!actual.contains("stream key"), "no stream key in: {actual}");
    }
}
```

Rules for the implementer: paste the block exactly; do not reword any string; if `cargo fmt --check` disagrees with the pasted layout, run `cargo fmt` in the container and keep its output — the formatter is authoritative (a `style: apply rustfmt` step was precedented at `0a459b2`).

## 3. Reviewed and deliberately NOT changed (do not touch)

- `scheduled_start_time.rs` civil-date helpers (inline `400 *`, `skip_whole_years` `&mut` parameter): the duplication of `src/streaming/utc_timestamp.rs` (including its exact helper shapes and the inline 400) is the D-T1-5 approved design decision; changing one copy breaks the documented parity and buys nothing. Both files stay byte-consistent with the sealed precedent.
- `live_error/display.rs` 14 tiny helper fns: mandated by plan F4 ("Display is written per-variant-group helper fns — the Phase-03 `StreamingError` split precedent").
- `StreamHealth.overall` private field: correct under prefer-private; Task 2 widens visibility exactly when it logs the healthStatus.
- `video_output.rs` single-arm `match`: exhaustive-match pattern over a 1-variant enum; standard and plan F6-approved.
- `live_models_tests.rs` (196 lines): inside the 200 cap; the `stream_page_...` body (45 lines) and `create_commands_...` body (41 lines) are inside the method cap — already compressed at `fc610e4`; further splitting would only churn the §5 T1 roster.

## 4. Prohibitions (staged-design boundary)

- NO change to any production type, trait, enum variant, Display string, classifier table, serde rename, constant, or module declaration.
- NO change to `#[allow(dead_code)]` anywhere (inventory must remain exactly 15 in `src/`).
- NO test added, removed, renamed, or semantically altered (roster delta: 0; total stays 282).
- NO touch to `Cargo.toml`/`Cargo.lock`, `src/app/`, `src/streaming/`, config files, `.agent/` docs, or the Live-Chat seam.
- NO push, NO merge, NO version bump; feature-branch commit only.

## 5. Verification (Alpine VM MCP; `vm_status` first; single commands, never chained)

1. `git status --porcelain` — expect only the Task-1 files clean; branch `feat/phase05-production-live-runtime` at `fc610e4`.
2. Apply §2, then `git diff` — expect exactly one changed file: `src/youtube/live_error_display_tests.rs`.
3. Spot suite (must stay green, count unchanged):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked youtube"`
   → exit 0; **105 youtube-name tests** — the observed count in the baseline gate `20261014T015119Z.log` (authoritative; the 100 estimate in the Task-1 plan §6 S9 was superseded by observation).
4. Formatter authority pass (only if step 5 flags fmt):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo fmt"`
5. Full gate (must stay green):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
   → exit 0, `ALL CHECKS PASSED`, `running 282 tests` (observed gate log is authoritative; record its `logs/checks/<UTC>.log` path).
6. Hygiene: `git status --porcelain` lists only `src/youtube/live_error_display_tests.rs` (+ the new gate log if tracked conventions apply); `git grep -n -F "allow(dead_code)" src/` still counts 15; no `.gitignore`-matching file staged.
7. Commit: `refactor: keep the live api display contract test under the method line cap`.

## 6. Handoff

The implementer executes §2 + §5 only. The reviewer side of 4.3 re-checks §3/§4 prohibitions, the 15-item allow inventory, the unchanged test roster (28 new / 282 total), and the fresh green gate log.

Cross-plan note: the reviewer-side 4.3 plan `.kilo/plans/20261013-phase05-task1-review-fix.md` (Fixes A–D) touches `live_error/display.rs`, `live_error/mod.rs`, `live_models/mod.rs`, `.agent/project-structure.md` — ZERO file overlap with S1; both plans can be applied by the implementer in any order (each ends with the same gate). The observed youtube-spot count after both plans remains 105 (Fix B/C change re-export visibility, not test counts).

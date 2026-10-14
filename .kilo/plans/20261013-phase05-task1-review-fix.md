# Phase 05 — Task 1 Review Fix Plan

- Reviewer sub-step: 4.3 (Code Review — reviewer side)
- Branch: `feat/phase05-production-live-runtime`
- Base for review: `7769efd..HEAD` (`fc610e4`)
- Status: fixes required before Task 1 can move to 4.5b adherence

---

## Adjudication of reported deviations

| # | Deviation | Verdict | Rationale |
|---|---|---|---|
| 1 | `live_error` landed before `live_lifecycle` (dependency order reversed from commit plan) | **Acceptable** | The actual dependency direction is `live_lifecycle` → `live_error`; the source order and commit order do not affect compilation or correctness. Tests pass. |
| 2 | F1 (`live_models`) and F4 (`live_error`) were split into directory submodules to stay under the 200-line/file cap | **Acceptable with corrections** | The split is a local structural refactor that preserves behavior, but it inadvertently made the classifier and request/list body types reachable only to tests. The fix section below restores the production visibility the plan intended. |
| 3 | `BroadcastStatus.privacy_status` and the `contentDetails` serde mapping were added beyond the plan sketch | **Acceptable** | `privacyStatus` is a documented `liveBroadcasts` field; adding it is harmless and the test fixture’s extra `enableAutoStart` field correctly exercises the unknown-field-tolerance contract. |
| 4 | `Display` wording was shortened vs. the plan’s actionability examples | **Acceptable** | The rendered lines remain stable, secret-free, and are pinned by `live_error_display_tests.rs`. Exact wording is a minor local detail. |
| 5 | Stale `session-` mention remains in `.agent/project-structure.md` line 80 after `OutputKind::Session` was retired | **Unacceptable** | The retired variant must not be documented as still producing filenames; this can mislead future agents. Remove the stale reference. |

---

## New findings requiring fix

1. **`LiveApiError` does not implement `std::error::Error`.**  
   The plan (F4) requires `Display`, derived `Debug`, and `std::error::Error::source`. The enum has the first two but is missing the `Error` impl, which breaks the module’s existing error-trait convention (`youtube/error.rs`).

2. **`classify_api_error` is only re-exported under `#[cfg(test)]`.**  
   In the directory-submodule layout, `live_error::classify` is private. The plan intended `classify_api_error` to be `pub(crate)` so Task 2’s transport can call it. It must be re-exported unconditionally from `src/youtube/live_error/mod.rs`.

3. **`LiveBroadcastList`, `LiveStreamList`, `BroadcastRequestBody`, `StreamRequestBody`, and `StreamCreationCdn` are only re-exported under `#[cfg(test)]`.**  
   Task 2’s concrete backend needs to parse list pages and serialize insert bodies. These types must be unconditionally re-exported from `src/youtube/live_models/mod.rs`.

---

## Exact fixes (implementer applies these only)

### Fix A — Implement `std::error::Error` for `LiveApiError`

**File:** `src/youtube/live_error/display.rs`

After the closing brace of `impl fmt::Display for LiveApiError { ... }`, add:

```rust
impl std::error::Error for LiveApiError {}
```

No custom `source()` is needed because none of the variants wrap another error.

---

### Fix B — Unconditionally re-export `classify_api_error`

**File:** `src/youtube/live_error/mod.rs`

Replace:

```rust
// Test-only re-export: the offline classifier/Display tests name the
// function; production code never calls it until Task 2's transport.
#[cfg(test)]
pub(crate) use classify::classify_api_error;
```

with:

```rust
// Re-exported for Task 2's concrete transport and for offline tests.
pub(crate) use classify::classify_api_error;
```

---

### Fix C — Unconditionally re-export the list-page and request-body types

**File:** `src/youtube/live_models/mod.rs`

Replace the entire re-export block (lines 10–24) with:

```rust
pub(crate) use broadcast_resource::{LiveBroadcast, LiveBroadcastList};
pub(crate) use commands::{BindCommand, CreateBroadcastCommand, CreateStreamCommand};
pub(crate) use stream_cdn::StreamCreationCdn;
pub(crate) use stream_resource::{LiveStream, LiveStreamList};

// Re-exported for Task 2's concrete transport and for offline model tests.
pub(crate) use request_bodies::{BroadcastRequestBody, StreamRequestBody};
```

Remove the now-redundant `#[cfg(test)]` re-exports so the same names are not declared twice.

---

### Fix D — Remove stale `session-` reference from project structure

**File:** `.agent/project-structure.md`

Replace (line 80):

```text
  - `video_output.rs` - `resolve_output_path`: beside-executable `videos/` directory created on demand; collision-safe `demo-`/`session-` names (`_2.._1000`); conservative existence probe
```

with:

```text
  - `video_output.rs` - `resolve_output_path`: beside-executable `videos/` directory created on demand; collision-safe `demo-` names (`_2.._1000`); conservative existence probe
```

No other edits to `.agent/project-structure.md` are required.

---

## Verification commands (run in order)

1. Format check:

   ```text
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo fmt --check"
   ```

   Expected: exit 0, no diff output.

2. YouTube-module tests:

   ```text
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked youtube"
   ```

   Expected: exit 0; all youtube tests pass (count remains 105 until further test additions).

3. Streaming output regression test:

   ```text
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked streaming::video_output"
   ```

   Expected: exit 0; 10 passed.

4. Full unit/integration test suite:

   ```text
   docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked 2>&1 | tail -n 5"
   ```

   Expected: exit 0; summary reports `282 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`.

5. Final hygiene check:

   ```text
   git status --porcelain
   git diff --stat
   ```

   Expected: only the four files above are modified; no `.gitignore`-matching files staged; no credentials, tokens, stream keys, URLs, videos, or build artifacts committed.

---

## What is NOT in this fix plan

- No changes to `src/app/`, `src/streaming/` beyond the doc fix, `src/chat/`, `src/renderer/`, `Cargo.toml`, `Cargo.lock`, or `config/`.
- No implementation of Task 2 (concrete API backend / polling) or Task 3 (`run` wiring / RTMPS destination / FFmpeg changes).
- No README/docs/config-example updates (Task 4 scope).
- No full `scripts/dev-checks.sh` gate run (that belongs to Task 4; only targeted cargo verification is performed here).
- No commit, push, version bump, or TODO marking.

After applying these fixes, return the branch to the reviewer for re-check.

---

## Planner ruling (2026-10-14) — clippy blocker on Fixes B/C

Observed: with Fixes B/C applied, `cargo clippy --locked -- -D warnings` fails (exit 101) with
5× `unused_imports` because the unconditionally re-exported types/classifier are not yet
consumed by any non-test build target (Task 2's transport does not exist yet). Neither fix plan
authorized a remedy; the Planner decides.

**Ruling: Planner option 1 — scoped `#[allow(unused_imports)]` on the re-export statements**, as
the direct analog of the project's established Phase-04 whole-module staging convention:

- `src/youtube/live_error/mod.rs`: attribute the `pub(crate) use classify::classify_api_error;`
  statement with `#[allow(unused_imports)]` plus a comment naming the retirement contract:
  "Retire when Task 2's concrete transport consumes the classifier."
- `src/youtube/live_models/mod.rs`: attribute the same way the `pub(crate) use`
  statement(s) that carry the not-yet-consumed list/request-body types, with the retirement
  comment: "Retire when Task 2's concrete transport consumes the request/list types."
- The `#[allow(dead_code)]` inventory stays exactly 15 (`allow(unused_imports)` is a different
  attribute and is not part of that inventory); the staged-allow retirement list recorded for
  the `run` phase is extended accordingly.
- Rejected: option 2 (violates the green-gate-before-completion requirement), option 3 (undoes
  the reviewer's production-visibility fix intent).
- The implementer re-runs fmt/clippy/gate after applying, commits the remedy, and tracks the
  two 4.3 plan files under `.kilo/plans/` (`docs:` commit precedented).

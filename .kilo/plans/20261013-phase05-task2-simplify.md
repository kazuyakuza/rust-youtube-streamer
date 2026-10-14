# Phase 05 — Task 2 Simplification Plan (sub-step 4.3, simplifier side)

- Date: 2026-10-14
- Authority: TODO `.agent/todos/20261013/20261013-todo-1.md` Task 2 + global Scope/Boundaries +
  Explicitly-Out-of-Scope (user ruling: Live Chat is Phase 06, absent here); plans
  `.kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md` + Amendments 1–2
- Base: branch `feat/phase05-production-live-runtime`, HEAD `f4835d9`, diff reviewed `cc6daf6..HEAD`
  (all 31 changed files read in full), gate reference `logs/checks/20261014T155701Z.log` (332 tests)
- Verdict: **CONTAINED simplifications warranted — 4 items.** No architectural shape change;
  net ≈ −60 lines across 9 files; zero behavior change; every item is fully test-pinned.
- Implementer: junior, 50% restriction — execute steps S0–S4 exactly as written. No judgment
  calls; every replacement block is verbatim. Any deviation from this plan → stop and ask caller.

## 1. Review Findings (what was examined and what is deliberately NOT changed)

Reviewed: `api_request.rs`, `google_live_api_transport.rs`, `live_ingestion.rs`,
`readiness_wait/{mod,confirmation_wait,observation}.rs`, `rtmps_ingestion.rs`,
`wait_cancellation.rs`, `live_test_fixtures.rs`, `live_models/` edits, `live_error/` edits,
all 9 test files, `src/youtube/mod.rs`.

**Worth simplifying (→ §2 items 1–4):**

1. Dead duplicate constant: `UNREADABLE_BODY_TEXT` at `google_live_api_transport.rs:30–32` is
   defined but never referenced there (its live twin in `api_request.rs:24` is the one used by
   `response_body`). The module's staging `#[allow(dead_code)]` masks the unused warning.
2. Missing plan-intended helpers / triple-duplicated attempt matrix: plan F-T2-3 names
   `fn sleep_between_polls` and D-T2-9 states the transient/fatal matrix exists exactly once;
   the landed code instead repeats the `std::thread::sleep(std::time::Duration::from_secs(..))`
   ceremony 3× and the `Ok/Err-if-transient/Err` match pattern 3× (twice in
   `readiness_wait/mod.rs::poll_cycle`, once in `confirmation_wait.rs::confirmation_poll`).
3. Duplicated generic test helper `fn queued<T>`: defined in `live_ingestion_tests.rs`
   (`pub(super)`) AND again locally in `readiness_wait_confirmation_tests.rs`; natural home is
   the shared `live_test_fixtures.rs` (Amendment 1 topology).
4. Hand-rolled command clones in the fake: `copied_stream_command` / `copied_broadcast_command`
   + a 4-line `BindCommand` literal in `live_test_fixtures.rs` — the command structs are pure
   `String` fields without `#[derive(Clone)]`.

**Deliberately NOT changed (prohibited — do not "fix" these):**

- `send_string(body.to_string())` stays: plan D-T2-4 said `.send_json(..)` but the crate's
  `ureq = "2"` has no `json` feature enabled and enabling it would edit `Cargo.toml`/`Cargo.lock`
  (adherence §8: dependency diff must stay EMPTY). The landed form is wire-identical and
  loopback-pinned by roster A tests. Not a simplification target.
- `live_error/mod.rs` enum multi-line variants stay: rustfmt expanded all struct-variants the
  moment the doc-commented `WaitCancelled` landed; collapsing them fails `cargo fmt --check`.
- The 3xx doc-comment wording difference vs plan D-T2-5 (`Ok` vs `Status(..)` path): the code
  handles BOTH (`is_success_status` guard + `Error::Status` arm) and `redirects_are_not_followed`
  pins the outcome. Adherence/report note for Task 4, not a code change.
- Plan-specified public shapes stay untouched: `ReadinessWait::new` (4 args),
  `with_polling` (6 args, `#[cfg(test)]`), `wait_until_stream_active` +
  `wait_until_stream_active_with_polling` signatures, `StreamObservationRequest` fields.
  These are D-T2-9's "exact API shape" (binding; Task 3 consumes them). The ≤2-args rule tension
  is pre-authorized plan wording, not implementer drift — see §5 (escalation note, NOT a step).
- Redaction/secret machinery untouched: `RtmpsIngestion` (private `url`, redacting `Debug`, no
  `Display`, `into_url` + its item allow), `AccessToken`/Bearer single site, classifier,
  "bodies never embedded" paths, all staging `#[allow(dead_code)]` lines and contract comments.
- No app/streaming/main/Cargo/docs changes; no test added, removed, renamed, or weakened
  (roster coverage 332 tests stays exact); `.agent/project-structure.md` needs NO update
  (no file added/deleted by this plan).

## 2. Steps (sequential; one commit each after S0; green-at-each-point)

Pre-check (VM, `alpine-vm` MCP, `vm_status` first): `git status --porcelain` empty;
`git branch --show-current` = `feat/phase05-production-live-runtime`; HEAD `f4835d9` or a later
commit of this branch.

### S0 — Commit this plan

```text
git add .kilo/plans/20261013-phase05-task2-simplify.md
git commit -m "docs: track the phase 05 task 2 simplification plan"
```

### S1 — Drop the dead duplicate constant (item 1)

File `src/youtube/google_live_api_transport.rs`: delete EXACTLY these three lines (30–32):

```rust
/// Fixed text when a 2xx reply body cannot be read (the Phase-04
/// `UNREADABLE_BODY_TEXT` precedent).
const UNREADABLE_BODY_TEXT: &str = "the YouTube API response body could not be read";
```

plus the one blank line that would leave a double blank between `LIVE_API_BASE_URL` and the
struct. No other edit in the file. Expected: 159 → 155 lines.

Spot-check (VM): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked youtube"` → exit 0.
Assertion (Windows local): `git grep -c "UNREADABLE_BODY_TEXT" src/youtube/google_live_api_transport.rs` → no match / zero.

### S2 — Shared `tolerate_transient` + `sleep_between_polls` (item 2)

Three files. Bodies/behaviors stay identical: a transient failure consumes one attempt and the
loop sleeps then continues; every other error propagates immediately.

**S2a — `src/youtube/readiness_wait/observation.rs`** — insert after the `is_transient` function
(i.e. between it and `attempt_exhausted`):

```rust
/// Attempt-matrix gate (D-T2-9): a transient failure consumes one attempt
/// (`Ok(None)`); any other error is immediate and fatal.
pub(super) fn tolerate_transient<T>(
    outcome: Result<T, LiveApiError>,
) -> Result<Option<T>, LiveApiError> {
    match outcome {
        Ok(observed) => Ok(Some(observed)),
        Err(error) if is_transient(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Sleep between poll attempts — shared by both attempt-bounded waits
/// (plan F-T2-3 `sleep_between_polls`).
pub(super) fn sleep_between_polls(seconds: u64) {
    std::thread::sleep(std::time::Duration::from_secs(seconds));
}
```

Additionally DEMOTE the now-only-internally-used helper: change `pub(super) fn is_transient` to
`fn is_transient` (same file, one-word edit).
Expected: 78 → ≈93 lines.

**S2b — `src/youtube/readiness_wait/mod.rs`** — three edits:

1. Import block (lines 26–28): replace `is_transient` with the two new names, keep all others:

```rust
use observation::{
    attempt_exhausted, broadcast_observation, broadcast_state, sleep_between_polls,
    stream_observation, tolerate_transient,
};
```

2. In `run`, replace the line
   `std::thread::sleep(std::time::Duration::from_secs(self.poll_interval_seconds));`
   with `sleep_between_polls(self.poll_interval_seconds);`.

3. In `poll_cycle`, replace the stream-observation match (current lines 105–113):

```rust
        let observed = self
            .transport
            .lookup_stream(self.stream_id)
            .and_then(stream_observation);
        let (stream, stream_status) = match observed {
            Ok(observation) => observation,
            Err(error) if is_transient(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
```

   with:

```rust
        let observed = tolerate_transient(
            self.transport
                .lookup_stream(self.stream_id)
                .and_then(stream_observation),
        )?;
        let Some((stream, stream_status)) = observed else {
            return Ok(None);
        };
```

4. In `poll_cycle`, replace the broadcast-observation match (current lines 119–127):

```rust
        let lifecycle = self
            .transport
            .lookup_broadcast(self.broadcast_id)
            .and_then(broadcast_observation);
        let broadcast_lifecycle = match lifecycle {
            Ok(observed_lifecycle) => observed_lifecycle,
            Err(error) if is_transient(&error) => return Ok(None),
            Err(error) => return Err(error),
        };
```

   with:

```rust
        let lifecycle = tolerate_transient(
            self.transport
                .lookup_broadcast(self.broadcast_id)
                .and_then(broadcast_observation),
        )?;
        let Some(broadcast_lifecycle) = lifecycle else {
            return Ok(None);
        };
```

   EVERYTHING ELSE in `poll_cycle` (the `match stream_status { .. }` block, the `Revoked` check,
   `is_streamable_setup`, `is_both_ready`, final `Ok(None)`) stays byte-identical.
Expected: 159 → ≈151 lines.

**S2c — `src/youtube/readiness_wait/confirmation_wait.rs`** — two edits:

1. Replace line 7 `use super::observation::{attempt_exhausted, broadcast_state, is_transient,
   stream_observation};` with:

```rust
use super::observation::{
    attempt_exhausted, broadcast_state, sleep_between_polls, stream_observation,
    tolerate_transient,
};
```

   (`use super::*;` on line 8 stays exactly as is.)

2. Replace the whole `confirmation_poll` function (current lines 41–71) verbatim with:

```rust
fn confirmation_poll(
    transport: &dyn LiveApiTransport,
    stream_pair: &StreamObservationRequest<'_>,
    max_attempts: u32,
    poll_interval_seconds: u64,
) -> Result<(), LiveApiError> {
    for _attempt in 0..max_attempts {
        if stream_pair.cancellation.is_cancelled() {
            return Err(LiveApiError::WaitCancelled);
        }
        let observed = transport
            .lookup_stream(stream_pair.stream_id)
            .and_then(stream_observation)
            .map(|(_stream, status)| status);
        match tolerate_transient(observed)? {
            Some(StreamStatusKind::Active) => return Ok(()),
            Some(StreamStatusKind::Error) => return Err(broadcast_state(STREAM_STATUS_ERROR_DETAIL)),
            Some(_) | None => {}
        }
        sleep_between_polls(poll_interval_seconds);
    }
    Err(attempt_exhausted(CONFIRMATION_STAGE))
}
```

   Behavior parity (verify by reading, do not "improve"): previously the transient arm slept and
   `continue`d, skipping the bottom sleep — the new shape sleeps exactly once per consumed
   attempt at the loop bottom. Same total sleeps, same attempt accounting, same immediacy of
   `Active`/`Error`/fatal returns.
Expected: 71 → ≈62 lines.

Commit: `git add src/youtube/readiness_wait/observation.rs src/youtube/readiness_wait/mod.rs src/youtube/readiness_wait/confirmation_wait.rs` +
`git commit -m "refactor: share the wait attempt matrix and poll sleep"`

### S3 — Centralize the `queued<T>` test helper (item 3)

1. `src/youtube/live_test_fixtures.rs`: insert immediately BEFORE the existing
   `fn next_result<T>` (line ~99):

```rust
/// Push one scripted outcome onto a fake operation queue.
pub(super) fn queued<T>(queue: &RefCell<VecDeque<T>>, outcome: T) {
    queue.borrow_mut().push_back(outcome);
}
```

   (`std::cell::RefCell` and `std::collections::VecDeque` are already imported — lines 11–12.)
2. `src/youtube/live_ingestion_tests.rs`: delete the local
   `pub(super) fn queued<T>(...)` block (lines 21–23) and the now-unused imports
   `use std::cell::RefCell;` (line 10) and `use std::collections::VecDeque;` (line 11).
   `queued` resolves via the existing `use crate::youtube::live_test_fixtures::*;` glob.
3. `src/youtube/readiness_wait_confirmation_tests.rs`: delete the local `fn queued<T>` block
   (lines 15–17) and the now-unused imports `use std::cell::RefCell;` (line 8) and
   `use std::collections::VecDeque;` (line 9). Resolves via the fixtures glob (line 11).
4. `src/youtube/readiness_wait_tests.rs`: keep `queued_stream`/`queued_broadcast` and the type
   aliases; replace each one's two-line body with a single `queued` call:

```rust
fn queued_stream(transport: &ScriptedLiveApiTransport, outcome: StreamLookupOutcome) {
    queued(&transport.lookup_stream_results, outcome);
}

fn queued_broadcast(transport: &ScriptedLiveApiTransport, outcome: BroadcastLookupOutcome) {
    queued(&transport.lookup_broadcast_results, outcome);
}
```

No test name, assertion, or queue semantics change anywhere. Expected: fixtures 190→195,
live_ingestion_tests 178→172, confirmation_tests 63→57, readiness_wait_tests 198→194.

Commit: `git add src/youtube/live_test_fixtures.rs src/youtube/live_ingestion_tests.rs src/youtube/readiness_wait_confirmation_tests.rs src/youtube/readiness_wait_tests.rs` +
`git commit -m "refactor: centralize the scripted queue test helper"`

### S4 — `#[derive(Clone)]` on the command structs (item 4)

1. `src/youtube/live_models/commands.rs`: add `#[derive(Clone)]` to EXACTLY the three structs —
   one line immediately above each of `pub(crate) struct CreateStreamCommand {`,
   `pub(crate) struct CreateBroadcastCommand {`, `pub(crate) struct BindCommand {`
   (below their doc comments). No other edit (this is an additive derive; visibility and fields
   untouched; F-T2-6's edit-list extension is recorded in §1 of this plan).
2. `src/youtube/live_test_fixtures.rs`:
   - In `impl LiveApiTransport for ScriptedLiveApiTransport`, replace
     `*self.captured_create_stream.borrow_mut() = Some(copied_stream_command(command));`
     with `*self.captured_create_stream.borrow_mut() = Some(command.clone());`
   - Replace
     `*self.captured_create_broadcast.borrow_mut() = Some(copied_broadcast_command(command));`
     with `*self.captured_create_broadcast.borrow_mut() = Some(command.clone());`
   - In `bind`, replace the 4-line literal
     `*self.captured_bind.borrow_mut() = Some(BindCommand {` + its two field lines + `});`
     with `*self.captured_bind.borrow_mut() = Some(command.clone());`
   - Delete the two now-unused helper functions `copied_stream_command` and
     `copied_broadcast_command` (their doc-less bodies; nothing else references them — verify
     with the grep below).
Expected: commands 42→45, fixtures 195→≈175.

Commit: `git add src/youtube/live_models/commands.rs src/youtube/live_test_fixtures.rs` +
`git commit -m "refactor: clone the captured commands instead of copying"`

## 3. Assertions after S2–S4 (Windows local, single cmds)

```text
git grep -n "fn queued" src/youtube            → only the fixtures definition + the two queued_stream/queued_broadcast wrappers
git grep -n "copied_stream_command\|copied_broadcast_command" src/   → zero hits
git grep -n "is_transient" src/youtube         → only inside observation.rs
git grep -n "std::thread::sleep" src/youtube   → only the one site in observation.rs::sleep_between_polls
git grep -c "allow(dead_code)" src/youtube/mod.rs → 12, unchanged (staging contract intact)
git grep -n "UNREADABLE_BODY_TEXT" src/youtube/google_live_api_transport.rs → zero hits
```

## 4. Verification (Alpine VM, `alpine-vm` MCP, `vm_status` first; one command per invocation; NEVER chained)

After EACH of S1–S4:

```sh
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked youtube"
```

After S4, in order (each its own invocation, exit code authoritative):

```sh
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo fmt --check"
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo clippy --locked -- -D warnings"
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```

- `cargo fmt --check` failing on the §2 blocks → run `sh -c "cargo fmt"` ONLY, re-inspect the diff
  (mechanical formatting; never restructure), amend that step's commit. Formatting churn beyond
  the touched files → revert and ask caller.
- Full gate must end `ALL CHECKS PASSED`, test total EXACTLY 332 (no test added/removed),
  record the fresh `logs/checks/<UTC>.log` path in the completion summary.
- Any unexpected compile error/warning → STOP, do not improvise; return the error text to caller.

## 5. Escalation notes (NOT steps — caller/planner awareness only)

- E1 (plan-vs-rule, pre-authorized): the D-T2-9 "exact API shape" constructors exceed the
  ≤2-args guideline (`ReadinessWait::new` 4, `with_polling` 6 cfg(test),
  `wait_until_stream_active_with_polling` 4, internal `confirmation_poll` 4). Changing them means
  amending the binding plan text and Task 3's consumption surface — do it (if at all) in one
  coordinated Task-3-plan act, not here. If the caller wants it tracked separately, a new TODO
  file entry under Phase 05 Task 3 review is the correct home; nothing needs a new TODO file now.
- E2 (report line, Task 4): record the verified ureq 2 3xx-delivery behavior against plan D-T2-5's
  wording and the `send_string`-instead-of-`send_json` (json-feature-off) equivalence — both are
  already pinned by loopback tests; only the completion-report wording must reflect reality.
- E3: this plan does not weaken redaction, staging-allow counts, or the retirement contracts; the
  §8 adherence greps of the Task-2 plan keep their expected values.

## 6. Prohibitions (junior-boundary)

- No changes to: `src/app/**`, `src/streaming/**`, `src/main.rs`, `Cargo.toml`, `Cargo.lock`,
  any `docs/`/README/`.agent/**` file, `live_lifecycle.rs`, `api_transport.rs`,
  `scheduled_start_time.rs`, `live_error/**` (staging ledger/Display/classifier), any test name,
  assertion, or fixture secret-shape.
- No `Display` added on any new type; no `secret()` call site moved or added; no `into_url`
  call-site changes; no removal or addition of any `#[allow(dead_code)]` line.
- No commit while any §4 command fails; no push; no merge; no version bump.
- If reality diverges from this plan (file moved, line anchors off, grep unexpected): STOP and
  return a question to the caller. NEVER adapt by judgment.

## 7. Expected end state

- 4 implementation commits + 1 docs commit, HEAD moves `f4835d9` → +5 commits on branch.
- Total diff: −60±10 lines net, 9 files touched; all files ≤200 lines; every body ≤50;
  no new nesting depth; no public-surface or behavior change; 332 tests green;
  gate `ALL CHECKS PASSED` with a fresh `logs/checks/<UTC>.log`.

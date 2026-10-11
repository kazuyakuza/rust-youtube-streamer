# Implementation Plan — Phase 02, Task 1: Chat Message Model and ChatStore

- Date: 2026-10-10
- Sub-step: Task 1, sub-step 4.1b (Analysis and Planning → implementation plan)
- Scope authority: `.agent/todos/20261010/20261010-todo-2.md`, section `### 1. Chat Message Model and ChatStore` (TOP PRIORITY). This plan never overrides the TODO.
- Framing: `.kilo/plans/20261010-phase-02-renderer-chat-store.md`. Tasks 2–4 of that TODO are OUT OF SCOPE here (their boundaries: Task 2 = renderer under `src/renderer/`, Task 3 = docs/metadata updates, Task 4 = coverage verification + builds + tracked completion report).
- Branch: `feat/phase02-renderer-chat-store` (already exists; package v0.5.0; `cargo check --locked` green). Do NOT create/switch branches.
- Target implementer: JUNIOR developer under 50% restriction — every structural decision below is final; no judgment calls delegated.

## Reading List (mandatory before implementation)

1. `.agent/todos/20261010/20261010-todo-2.md` — `### 1.` verbatim.
2. `src/config/model.rs`, `src/config/validation.rs`, `src/config/error.rs`, `src/config/mod.rs` — error/style/config precedent.
3. `src/config/validation_tests.rs`, `src/config/loader_tests.rs`, `src/config/test_fixtures.rs` — test style and temp-file mechanics.
4. `src/main.rs` (module declaration style), `.agent/project-structure.md`, `.agent/project-info/brief.md` §8.2, §8.3, §8.4, §11.2, `.agent/project-info/architecture.md` (`src/chat/` row + Boundary Rules).

## High-Level Approach

Create the greenfield `src/chat/` module (currently only `.gitkeep`) with four files mirroring the `src/config/` precedent:

- `src/chat/model.rs` — application-level `ChatMessage` (author + text visible; `message_id` + `timestamp` optional metadata, never shown) and the visible-format `Display`.
- `src/chat/error.rs` — typed `ChatError` mirroring `src/config/error.rs` conventions (hand-written `Display`, `std::error::Error` impl, no panics, no thiserror, no new deps).
- `src/chat/store.rs` — `ChatStoreSettings` (built from already-validated `&AppConfig`, derives visible capacity with overflow-safe u64 arithmetic inside the chat module) and `ChatStore` (bounded FIFO `VecDeque<ChatMessage>` + append-only log file handle, opened eagerly at construction).
- `src/chat/mod.rs` — module docs and the minimal public surface.

Plus two test files wired with the `#[cfg(test)] #[path = "..."] mod ...;` pattern used by `src/config/validation.rs`: `src/chat/model_tests.rs` and `src/chat/store_tests.rs`. They DELIVER the chat portion of the TODO Task 4 test list (Task 4 later only verifies coverage). No new dependencies (std + existing dev-dep `tempfile 3` suffice — justified: brief's log timestamp format is explicit-adjustable, so no date-time crate is warranted).

Code order: `error.rs` → `model.rs` → `store.rs` → `mod.rs` → `main.rs` `mod chat;` → tests → dev-checks gate.

## Encoded Decisions (final; implementer must not choose)

### D1 — Interface

- Capacity derivation lives in `src/chat/store.rs` as a private-ish helper, `pub(crate) fn derive_visible_capacity(video: &VideoConfig, renderer: &RendererConfig) -> Result<usize, ChatError>` (2 params — respects the ≤2-args rule). It recomputes `floor((height − top_margin − bottom_margin) / line_height)` defensively even though config validation already guarantees ≥ 1 line.
- Settings struct (1-param constructor, per the ≤2-args rule):
  ```rust
  pub struct ChatStoreSettings { visible_capacity: usize, log_path: PathBuf }  // fields private, no public accessors
  impl ChatStoreSettings {
      pub fn from_config(config: &AppConfig) -> Result<Self, ChatError>
  }
  ```
  `from_config` calls `derive_visible_capacity(&config.video, &config.renderer)` and takes `config.chat.log_file` as `PathBuf`. Importing `crate::config::model::{AppConfig}` into `src/chat/` is allowed (config is application-level, not a YouTube API type); no `max_messages` field exists anywhere. `ChatStoreSettings` derives `Debug, Clone, PartialEq`.
- Store:
  ```rust
  pub struct ChatStore { settings: ChatStoreSettings, log_file: fs::File, messages: VecDeque<ChatMessage> }  // all private
  impl ChatStore {
      pub fn new(settings: ChatStoreSettings) -> Result<Self, ChatError>          // opens log eagerly, append mode
      pub fn accept(&mut self, message: ChatMessage) -> Result<(), ChatError>     // D2 order
      pub fn visible_text(&self) -> String                                        // `"{author}: {text}"` lines joined by '\n', oldest first
      pub fn is_empty(&self) -> bool
  }
  ```
  That is the ENTIRE public surface. No `len()`, no iterator/mutator access, no extra getters. Private helpers: `write_log_record(&self, &ChatMessage) -> Result<(), ChatError>`, `enqueue(&mut self, ChatMessage)` (push_back, then evict front while `messages.len() > capacity`).
- Message model (all fields private; constructors trimmed to ≤2 params):
  ```rust
  pub struct ChatMessage { author: String, text: String, message_id: Option<String>, timestamp: Option<u64> }
  impl ChatMessage {
      pub fn new(author: &str, text: &str) -> Self                 // normalized (trim + newline flattening), metadata None
      pub fn with_message_id(mut self, id: &str) -> Self           // builder-style, for the future API layer; id stored verbatim
      pub fn with_timestamp(mut self, seconds: Option<u64>) -> Self // epoch seconds (u64) or None; stored verbatim
      pub(crate) fn author(&self) -> &str  // and equivalent accessors, pub(crate) only
      pub(crate) fn text(&self) -> &str
      pub(crate) fn timestamp(&self) -> Option<u64>
  }
  impl fmt::Display for ChatMessage { /* writes exactly `{author}: {text}` — NEVER any metadata */ }
  ```
  Unit of `timestamp`: seconds since the UNIX epoch (`u64` from `SystemTime` in the future API layer; `Option<u64>` keeps the model std-only and testable).
- Normalization (exact algorithm, in `ChatMessage::new`): (1) `\r\n` → one space, (2) remaining `\r` → one space, (3) remaining `\n` → one space, (4) `trim()` both author and text. Consequence: every accepted message occupies exactly ONE visible line and ONE log record line — this is why FIFO line-count capacity and log-record count stay in sync. Do not add any further validation (no blank-text rejection in MVP).

### D2 — I/O-failure semantics (log-append FIRST, reject on failure)

`accept` order is fixed: (1) write the normalized record to the log file (single `write_all` of `line + '\n'`; the handle is unbuffered so errors surface per message); (2) only after the append returns `Ok`, `enqueue` the message (with eviction when exceeding capacity). Enqueue cannot fail. Therefore:

- Log-append failure ⇒ the message is REJECTED: it is NOT enqueued, NOT logged, and `accept` returns `Err(ChatError::LogAppend { path, source })` (context: log path + wrapped `io::Error` + the fact the message was rejected, in `Display`). Queue and log can never diverge: every visible message has its log record; a failed message appears in neither.
- Construction failure (log file cannot be created/opened in append mode) ⇒ `Err(ChatError::LogOpen { path, source })`; no store exists yet so no state ambiguity.
- The log is append-only and is NEVER read back — not even an existence/size check beyond the open — and never repopulates a queue. Each new `ChatStore` starts with an empty `VecDeque` regardless of prior log contents.

### D3 — Log record format and timestamps

- Exact record format (documented here; deviations surface in Task 3 docs, not in this step):
  ```text
  <epoch_seconds> <author>: <text>
  ```
  e.g. `1728381296 alice: Hello!`. UTF-8 only; LF (`\n`) record separator; one record per accepted message; nothing ever appended for failed/rejected messages. This intentionally deviates from the brief §11.2 example timestamp (`2026-10-08T12:34:56Z`): `std` has no RFC 3339 formatter and a date-time crate is an unjustified new dependency for MVP; the brief explicitly permits the format to be adjusted "but it must be documented and unambiguous" — decimal epoch seconds with a single ASCII space separator is unambiguous. Record text is flattened by normalization (D1), so it is always a single line.
- Timestamp source: `message.timestamp` when present (`Some(u64)`), otherwise the record's generation time captured inside `write_log_record` via `SystemTime::now().duration_since(UNIX_EPOCH)`; the duration (`u64`) has no fallback path — if extraction fails it maps to `u64::MAX` via `unwrap_or` ONLY as the never-practically-hitting degenerate saturating value (encode: `map_or(u64::MAX, |d| d.as_secs())`); do not panic and do not treat it as an error case.
- Display text construction: `visible_text` joins one `message.to_string()` per message with `'\n'`; an empty store yields the empty string `""`.

### D4 — Error type

`src/chat/error.rs` mirrors `src/config/error.rs` exactly in style (doc-comment header, hand-written `Display`, `impl std::error::Error` with `source()`, no panics, no derives beyond `Debug`):

```rust
pub enum ChatError {
    Capacity { height: u32, top_margin: u32, bottom_margin: u32, line_height: u32 }, // usable height < 1 line, margins >= height, or line_height == 0
    LogOpen   { path: PathBuf, source: io::Error },   // ChatStore::new could not open the log in append mode
    LogAppend { path: PathBuf, source: io::Error },   // accept's log write failed; Display states the message was rejected and not added to the visible queue
}
```

`Display` strings: `Capacity` → `chat visible capacity must allow at least one line (height {..}, top_margin {..}, bottom_margin {..}, line_height {..})`; `LogOpen` → `unable to open chat log file '{path}' in append mode: {source}`; `LogAppend` → `unable to append chat record to chat log file '{path}': {source}; the message was rejected and was not added to the visible queue`. `source()` returns the wrapped `io::Error` for `LogOpen`/`LogAppend`, `None` for `Capacity`. Public re-export through `src/chat/mod.rs`.

### D5 — Test scope (delivered by Task 1; mapped-from TODO Task 4 first three bullets)

All new tests are unit tests inside the crate, run by the standard gate. Files: `src/chat/model_tests.rs` (wired from `model.rs`) and `src/chat/store_tests.rs` (wired from `store.rs` as a child module — child modules may read the parent's private fields, per the `use super::*;` precedent in `src/config/validation_tests.rs`).

Temp-dir mechanics (every file-behavior test): `let temp = tempfile::tempdir().expect("create temp dir"); let log_path = temp.path().join("chat.log");` then build config fixture with that path. Constants for test geometry: exact integer division `height 100, margins 20/20, line_height 20` → capacity 3; fractional `height 110, margins 20/20, line_height 20` → 3.

`src/chat/model_tests.rs` (5 tests):
1. `display_format_is_username_then_message` — `"alice" / "Hello!"` → `Display == "alice: Hello!"`.
2. `metadata_never_appears_in_display` — with `with_message_id("SECRET-ID-123")` + `with_timestamp(Some(1728381296))` → `Display == "alice: Hello!"` and does not contain `"SECRET-ID-123"` or `"1728381296"`.
3. `normalization_trims_surrounding_whitespace`.
4. `normalization_flattens_all_line_breaks` — one case per `\r\n`, lone `\r`, lone `\n` (each → single space) and `"\r\n"` ordering (CRLF must not become two spaces).
5. `metadata_accessors_return_input_verbatim` — id/timestamp round-trip; absent metadata is `None`.

`src/chat/store_tests.rs` (12 tests):
6. `capacity_uses_floor_division_of_usable_height` — exact fit → 3; fractional remainder case → 3 (assert via `derive_visible_capacity` return value).
7. `capacity_rejects_unusable_or_overflowing_geometry` — margins ≥ height (`checked_sub` overflow path: top+bottom `u32::MAX + u32::MAX` vs height `u32::MAX`), zero line_height, height smaller than occupied `top+bottom+line_height`; each returns `Err(ChatError::Capacity)` — asserts no panic, plus u64 arithmetic survives `u32::MAX` values.
8. `new_store_starts_with_empty_queue` — empty `visible_text`, `is_empty()`.
9. `single_message_display_format` — accept `alice / Hello!` → `visible_text == "alice: Hello!"`.
10. `fifo_evicts_oldest_beyond_capacity` — capacity 3; accept 4 messages `m1..m4`; `visible_text == "m2: ...\nm3: ...\nm4: ..."`.
11. `log_family_and_ordering_preserved_beyond_queue` — same 4-accept run: read the log file back in the TEST only; assert it contains exactly 4 records, oldest-first, INCLUDING the evicted one (proves log/queue independence and append-only ordering).
12. `log_record_format_with_message_timestamp` — accept a message with `with_timestamp(Some(1728381296))` after optional prior accepts; the log file's LAST line is the byte-identical `"1728381296 alice: Hello!"\n` and the file ends with LF.
13. `log_record_uses_generation_timestamp_when_message_has_none` — exact value is nondeterministic: assert the first whitespace-delimited token of the record parses as `u64` and the remainder is `"alice: Hello!"`.
14. `newlines_in_messages_flatten_to_one_record_one_display_line` — text `"line1\nline2&r"` style; log file line count equals accepted count; `visible_text` contains no `\n` inside that record.
15. `log_open_failure_is_typed_and_surfaces_path` — log path inside a nonexistent parent (`<tempdir>/does-not-exist/chat.log`): `ChatStore::new` → `Err(` variant with `LogOpen`) and `Display` contains the path text.
16. `append_failure_rejects_message_and_log_stays_consistent` (the D2 rejection test) — construct a working store, accept one message; then, from the child test module, replace the private `log_file` field with a READ-ONLY `File` (`OpenOptions::new().read(true).open(log_path)`), causing the next `write_all` to fail on the Linux container (all tests run only in the Linux toolchain — pass condition is container-scoped): assert `accept` → `Err(LogAppend { .. })` whose `Display` mentions the log path AND "rejected"; assert the queue still shows only the first message and the log file (read by the test) still contains exactly 1 record.
    Fallback only if a clippy/format issue arises from the field swap: instead simulate via rotating `settings` to a directory path is NOT allowed — the read-only-fd swap is the encoded mechanism.
    If >200 lines become a constraint risk, merge tests IN THIS ORDER ONLY: {8 into 9}, then {12 into 11}; never restructure further.
17. `new_store_ignores_existing_log_history` — pre-create the log file with 3 fake lines BEFORE `ChatStore::new`; assert empty queue (no replay), then one accept appends record #4 after them.

Helpers at file top of each test file: config-fixture builder using `crate::config::test_fixtures::valid_config()` (test-only alias, already the precedent) adjusted per test (`config.video.height`, margins, `chat.log_file = log_path.to_string_lossy().into_owned()`), and a small `accept_all` loop helper to keep test bodies short.

### D6 — mod.rs wiring and public surface

`src/chat/mod.rs`: module doc comment (2–6 lines) stating the responsibility ("application-level chat message model and bounded visible-message queue ChatStore it cannot depend on YouTube API types, network, streaming, or FFmpeg") referencing brief §8.4/§11.2, plus ONLY:
```rust
mod error;
mod model;
mod store;
pub use error::ChatError;
pub use model::ChatMessage;
pub use store::{ChatStore, ChatStoreSettings};
```
No other `pub` items anywhere in the module (prefer-private-members rule). `ChatMessage` privacy: all fields private; only model accessors are `pub(crate)`.

## Atomic Implementation Steps

### Step 1 — `src/chat/error.rs`

Create the file per D4. Body target ≤ 90 lines. Verify mentally against `src/config/error.rs` structure; maximal reuse of its idioms (doc-comment style, `match` in `Display` via separate free functions when a `match` arm would push method body past 50 lines — the `write_validation` precedent).

### Step 2 — `src/chat/model.rs`

Create per D1's message model + normalization + `Display`; append at the bottom:
```rust
#[cfg(test)]
#[path = "model_tests.rs"]
mod model_tests;
```
Body target ≤ 110 lines (excluding the test mod wiring). Normalization is two functions max: `normalize(start_text: &str) -> String` (chained replaces + trim) kept short; no `&&`/`||` anywhere.

### Step 3 — `src/chat/store.rs`

Create per D1/D2/D3. Structure:

- `use std::collections::VecDeque; use std::fs::{self, File}; use std::io::Write; use std::path::PathBuf; use std::time::{SystemTime, UNIX_EPOCH};`
- `pub(crate) fn derive_visible_capacity(video: &VideoConfig, renderer: &RendererConfig) -> Result<usize, ChatError>` — u64 only: `line == 0` early-return `Err(Capacity)`; `occupied = u64::from(top) + u64::from(bottom)` (cannot overflow u64 from two u32s); `inner = height.checked_sub(occupied).ok_or(ChatError::Capacity { .. })?`; `inner / line` then `usize::try_from` mapping any failure to `Err(Capacity)`. Zero nesting depth > 2; method body < 50 lines.
- `ChatStoreSettings::from_config(&AppConfig)` → calls `derive_visible_capacity`, wraps `config.chat.log_file.clone()` into `PathBuf`.
- `ChatStore::new(settings)` → `File` via `fs::OpenOptions` `.create(true).append(true).open(&path)` in a match arm mapping failure to `Err(LogOpen)`; `messages: VecDeque::new()`.
- `ChatStore::accept` → `self.write_log_record(&message)?;` then `self.enqueue(message); Ok(())`.
- `write_log_record` → pick seconds: `message.timestamp()` else `SystemTime::now()...as_secs()` degenerate `u64::MAX` mapping per D3; build line `format!("{} {}: {}\n", seconds, message.author(), message.text())` — note `Display` not used here (visible format never escapes to logs). Actually build line as `format!("{seconds} {author}: {text}\n", ...)` with one-line write `self.log_file.write_all(line.as_bytes())` mapped to `Err(LogAppend { path, source })`.
- `enqueue` → `push_back` + `while self.messages.len() > self.settings.visible_capacity { self.messages.pop_front(); }` — single-section boolean; deterministic single eviction given the ≤ capacity invariant (a `while` is used so the invariant survives any future drift without a second code path).
- Bottom:
```rust
#[cfg(test)]
#[path = "store_tests.rs"]
mod store_tests;
```
Body target ≤ 195 lines excluding the test wiring.

### Step 4 — `src/chat/mod.rs`

Delete `src/chat/.gitkeep`, create `mod.rs` per D6 (≤ 35 lines).

### Step 5 — `src/main.rs`

Add one line `mod chat;` after `mod config;` (no behavior change, no app::run wiring — required so unit tests compile). Nothing else in app/ is touched.

### Step 6 — `src/chat/model_tests.rs` and `src/chat/store_tests.rs`

Create per D5's 5 + 12 test spec. Each test ≤ 15 lines; helpers at each file top: a store builder (build the `tempfile::tempdir()`, build the `AppConfig` fixture with adjusted height/margins/`chat.log_file`, return `(ChatStore, tempdir, log_path)`), and a small accept loop; `use super::*;` + fixture imports. Compact but readable; assertions name-check messages/paths per the config test style (`assert!(message.contains(...))` precedent).

### Step 7 — Verification gate (via Alpine VM MCP `alpine-vm_vm_run_command`)

1. `alpine-vm_vm_status` first (contract), then:
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
   Exit code 0 with four PASS lines (`fmt-check`, `check`, `test`, `clippy`) and `ALL CHECKS PASSED`. Test count grows from 46 to 46 + 17 = 63 (report the actually observed count; if it differs, the diff reflects chat tests only — investigate any unexplained delta before proceeding).
   Fix anything failing before any commit beyond Step 8; failures are per-check with FAIL lines and exit 1.

### Step 8 — Git commits (after the gate is green; before each, `git status` + `.gitignore` re-read; NEVER stage `dist/`, logs, or ignored artifacts)

- Commit 1 (Steps 1–2): `feat(chat): add typed chat errors and chat message model` — files `src/chat/error.rs`, `src/chat/model.rs`, `src/chat/model_tests.rs`.
- Commit 2 (Steps 3–5): `feat(chat): add bounded ChatStore with append-only chat log` — files: `src/chat/store.rs`, `src/chat/mod.rs`, deleted `src/chat/.gitkeep`, `src/main.rs` (exact paths only).
- Commit 3 (Step 6 tests if later split from Step 3): `test(chat): add chat store and model unit tests` — normally already included in Commit 2; split only if commits would exceed a reasonable single diff → in that case Commit 2 is code-only and Commit 3 is tests-only (encode: split at this boundary, not finer).

### Step 9 — Explicitly NOT in this plan (boundaries)

- No renderer code (`src/renderer/` untouched), no config schema changes, no docs/README/structure/context updates (Task 3), no report (Task 4), no version bump (already at 0.5.0 from the earlier step), no `app::run` wiring, no network/YouTube/streaming code, no `max_messages` anywhere. `.agent/project-structure.md` is updated later in Task 3, not now.

## Self-Verification Against TODO Task 1

- Model under `src/chat/`, no YouTube API types → D1/D6 (imports are `std` + `crate::config::model` only). ✔
- Display `(username: message)`, optional ID + timestamp, metadata off-screen → D1, model test 2. ✔
- Bounded FIFO `VecDeque`, visible-only messages → D1 store. ✔
- Capacity derived overflow-safe floor-division from validated geometry, ≥ 1 enforced, no `max_messages` → D1/D4 + tests 6–7. ✔
- On accept: log append + queue update with eviction, defined I/O-failure policy avoiding silent log loss / ambiguous state → D2 + tests 16. ✔
- Log path and I/O errors surfaced, append-only, never read back ✔ → D2/D4 + tests 15–17.
- Fresh stores start empty, no replay/network/YouTube → D2/D6 + test 17. ✔
- Testable file behavior with temp files → D5 tests. ✔

All TODO Task 1 bullets map; no ambiguity left for the implementer.

## Amendments (2026-10-10, Planner rulings after 4.2 report)

Rationale: implementer stopped with two gate blockers outside its authority. These rulings amend the plan; the TODO is untouched and still wins on any conflict.

### A1 — Cross-module config types (inherits BLOCKER 1 decision: implementer Option A)

- `src/config/mod.rs`: change the private declaration `mod model;` to `pub(crate) mod model;` (single line). Crate-internal visibility only; the existing `pub use model::AppConfig;` stays legal in a 2021 binary crate. `src/chat/store.rs` keeps importing `crate::config::model::{AppConfig, RendererConfig, VideoConfig}`.
- `src/config/mod.rs` is added to Commit 2's file list (chat-enabling wiring change; no other config edits).

### A2 — mod.rs re-exports dropped (inherits BLOCKER 2 decision: no lint suppressions)

- D6 amended: `src/chat/mod.rs` contains `mod error; mod model; mod store;` and module docs ONLY — the three `pub use` re-export lines are REMOVED. Justification: binary crate ⇒ no public API ⇒ clippy `-D unused-imports` is correct to flag them; the Phase-01 no-`#[allow]` precedent forbids suppression.
- ChatStore/ChatMessage/ChatError remain reachable to `src/chat/*` children (`super::*` tests unaffected). The re-exports are re-introduced when the first cross-module consumer lands (future app wiring phase; not in this workflow). mod.rs doc comment notes this access-path note.

### A3 — Accepted implementer deviations (report, run 3)

1. `mod chat;` placed alphabetically (`mod app; mod chat; mod config;`) — rustfmt `reorder_modules` requires it; plan's literal placement was wrong (plan-error, fmt-verified).
2. Plan-authorized test merge {8 into 9} applied; expected test count becomes 62 (46 + 16), not 63.
3. Test fixture import is `crate::config::valid_config` (actual `pub(crate)` alias); plan's literal `crate::config::test_fixtures::valid_config` path was private (E0603) — plan-error corrected.
4. `ChatStore` derives `Debug` (needed by `expect_err` in the LogOpen failure test and test clarity).

### A4 — Commit mapping update

- Commit 1: `src/chat/error.rs`, `src/chat/model.rs`, `src/chat/model_tests.rs`.
- Commit 2: `src/chat/store.rs`, `src/chat/mod.rs`, `src/chat/.gitkeep` (deletion), `src/main.rs`, `src/config/mod.rs`, plus `src/chat/store_tests.rs` (tests normally ride along; split only per original Step-8 boundary).

### A5 — dead_code ruling (Blocker 3 from run-5 report; decision R2)

- `src/main.rs`: declare the chat module as `#[allow(dead_code)] mod chat;` (single attribute on the module declaration; lint levels on a `mod` declaration propagate to the module tree, silencing the 9 `dead_code` warnings from unreachable-unwired items in the binary crate).
- Justification over `#[cfg(test)] mod chat;`: `cfg(test)` would exclude chat code from release artifacts and from `cargo clippy` coverage, weakening the review/tooling gate for exactly the new code. One narrowly scoped, documented suppression is the smallest compliant deviation; the Phase-01 state note (no `#[allow(` in `src/`) is acknowledged as superseded HERE in a time-boxed manner — the attribute is retired at the Phase-03 app-wiring step when real consumers exist.
- `src/chat/mod.rs` doc comment gains one short note that `dead_code` is intentionally allowed until wiring.
- Accepted: implementer's `normalize` clippy fix `replace(['\r', '\n'], " ")` (collapsible_str_replace; semantics equivalent to CRLF-first replace).
- Commit mapping: the `src/main.rs` change stays in Commit 2 per A4 (already listed).

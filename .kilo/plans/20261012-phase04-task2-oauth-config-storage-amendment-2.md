# Task 2 Plan — Amendment 2: credential-store file split (200-line cap relief)

- Date: 2026-10-13
- Branch: `feat/phase04-oauth-credentials`, HEAD `c133827`; UNCOMMITTED tree.
- Amends: `.kilo/plans/20261012-phase04-task2-oauth-config-storage.md` §3 File Map, §4.6, D4 wording, §6 Step 3 Commit C.
- Supersedes: nothing else changes; amendment-1 (`...-amendment-1.md`) remains landed (token.rs 200/200).
- Authority: `.agent/todos/20261012/20261012-todo-1.md` Task 2 ONLY. NO behavior change, NO test drops, NO test renames; the 15 store tests + 1 token test stay exactly.

---

## A0. Blocker restated (verified against on-disk uncommitted files)

- `src/youtube/file_credential_store.rs` = **262 lines** (cap 200).
- `src/youtube/file_credential_store_tests.rs` = **314 lines** (cap 200).
- `src/youtube/mod.rs` has the one line `mod file_credential_store;`; the store's
  test-foot block (`#[cfg(test)] #[path = "file_credential_store_tests.rs"] mod
  file_credential_store_tests;`) lives at the BOTTOM of `file_credential_store.rs`
  (not in `mod.rs`) — that is the actual convention; follow it.
- All 195 tests green per gate log `20261013T001713Z.log`. The split must preserve
  that exact test total.

The original §4.6 remedy ("trim doc comments, never invent a 21st file") is
UNSATISFIABLE: both files exceed the cap by far more than any doc trim saves.
DECIDED: split BOTH files.

## A1. DRAFT → DECIDED: new file `src/youtube/persisted_credentials.rs`

Conventions identical to the landed pair: source file carries the sibling
test-foot block; source bottom = 3 lines (`#[cfg(test)] #[path =
"persisted_credentials_tests.rs"] mod persisted_credentials_tests;`).

### A1.1 `src/youtube/persisted_credentials.rs` — owns the on-disk JSON schema + conversions

Module doc (new, ~8 lines, mirroring file_credential_store.rs's schema wording:

```rust
//! On-disk credential-file schema: the private `PersistedCredentials`
//! document and its conversions to/from [`StoredCredentials`]
//! (space-delimited `granted_scopes` round trip per plan §4.6 and
//! amendment-1's `GrantedScopes::to_space_delimited`). Serialization and
//! parse mapping live here; the store module owns only file IO
//! (temp-file creation, permissions, rename) and io-error mapping.
```

Imports: `use std::io;`, `use std::path::Path;`, `use serde::{Deserialize, Serialize};`,
`use super::credentials::{CredentialStoreError, StoredCredentials};`,
`use super::token::{AccessToken, GrantedScopes, RefreshToken};`.

MOVED verbatim from file_credential_store.rs (identical text, including doc comments):
- `struct PersistedCredentials` + its field doc comments (store lines 35–52).
  **Stays PRIVATE (`struct`, not `pub(crate)`)** — no other module names it.
- `impl PersistedCredentials { fn from_stored … }` (store lines 54–66) — **private**.
- `fn stored_from_persisted(parsed: PersistedCredentials) -> StoredCredentials`
  (store lines 182–189) — **private**.
- `fn parse_failure_message(…)` + `fn is_malformed_json(…)` (store lines 202–212) — **private**.
- `fn serialization_failure() -> io::Error` (store lines 214–219) — **private**.

NEW (only two items are exposed):

```rust
/// Serializes stored credentials to the persisted JSON document text.
pub(crate) fn serialized_document(credentials: &StoredCredentials) -> Result<String, io::Error>;

/// Parses persisted file text into stored credentials; malformed or
/// wrong-schema JSON maps to ReadParse with the fixed redacted message.
pub(crate) fn parse_credentials(
    text: &str,
    path: &Path,
) -> Result<StoredCredentials, CredentialStoreError>;
```

`serialized_document` body = the existing
`serde_json::to_string(&Self::from_stored(credentials)).map_err(|_| serialization_failure())`
composition. `parse_credentials` body = the existing `parse_persisted` +
`stored_from_persisted` composition:

```rust
pub(crate) fn parse_credentials(
    text: &str,
    path: &Path,
) -> Result<StoredCredentials, CredentialStoreError> {
    let parsed = serde_json::from_str::<PersistedCredentials>(text).map_err(|error| {
        CredentialStoreError::ReadParse {
            path: path.to_path_buf(),
            message: parse_failure_message(&error),
        }
    })?;
    Ok(stored_from_persisted(parsed))
}
```

### A1.2 `src/youtube/file_credential_store.rs` — retained store-side (extract, ~190 lines)

REMOVED (moved per A1.1): lines 34–66 (schema struct + `from_stored`),
181–189 (`stored_from_persisted`), 201–212 (`parse_failure_message` +
`is_malformed_json`), 213–219 (`serialization_failure`), 157–162 + 156 blank
(`parse_persisted` method — absorbed into `parse_credentials`), the
`use serde::{Deserialize, Serialize};` line (unused after the move), the
`use super::token::{…}` line (unused after the move).

ADDED: `use super::persisted_credentials::{parse_credentials, serialized_document};`

RETAINED unchanged (store/io domain only):
- Full 23-line module doc block (lines 1–23) — UNCHANGED text.
- `FileCredentialStore` struct + `new`.
- `ensure_parent_directory`, `parent_directory`, `temp_path`
  (all three still used directly by store tests), `open_temp_file`,
  `read_credential_text`, `replace_target`.
- `write_temp_file`: body becomes
  `let document = serialized_document(credentials).map_err(|source| CredentialStoreError::FileWrite { path: self.path.clone(), source })?;`
  then the existing `write_and_sync(…)` tail — same `FileWrite` mapping, same
  fixed "credential serialization failed" io message (now constructed in
  persisted_credentials; behavior byte-identical).
- `map_read_error` (io error mapping STAYS in the store), `write_and_sync`,
  `sync_replace` + its doc, both `#[cfg]` `open_temp_with_platform_permissions`
  arms, the test-foot block (path unchanged).

Deviations from the original plan, all sanctioned here:
- §4.6's "never invent a 21st file" clause is SUPERSEDED (Task 1
  amendment-1 `google_token_endpoint` split is the precedent).
- D4's "private to `file_credential_store.rs`" wording: the type is now private
  to `persisted_credentials.rs`; field names/schema text unchanged.
- The `ReadParse` construction moves with the schema (schema-owned mapping);
  `map_read_error`'s NotFound/FileOpen io mapping stays in the store. Display
  strings and `Error::source` behavior are untouched.
- Redaction/offline invariants UNCHANGED (D8/D10 keep; fixed messages byte-identical).

## A1.3 `src/youtube/mod.rs` — exactly one added line

`mod persisted_credentials;` inserted between `mod google_token_endpoint;` and
`mod token;` (alphabetical). NO re-exports (no external consumer yet — Task 3).
Test-foot wiring does NOT go into `mod.rs` (convention: each source file's own
foot).

## A1.4 Test split (movement map, 15 tests over two modules; names never change)

`src/youtube/persisted_credentials_tests.rs` (NEW) — receives 5 tests, bodies
switched from store save/load round trips to DIRECT calls to the two
`pub(crate)` functions; every assertion line preserved verbatim:

| moved test | new mechanism (asserts unchanged) |
| --- | --- |
| `save_writes_the_documented_schema` | `let text = serialized_document(&stored_credentials()).expect("save must succeed");` then the same key-order `.find` chain, sorted-key check, and 4 value asserts on `text` (store writes these bytes verbatim) |
| `load_reports_least_details_for_malformed_json` | write the same `malformed` string to a tempfile file named e.g. `tokens.json` under a `tempfile::tempdir()`, call `parse_credentials(&text, &store_file_path)` where `store_file_path = dir.path().join("tokens.json")`; assert identical `ReadParse.message` + no-fixture-leak |
| `load_reports_fixed_schema_mismatch_message` | same pattern with the `wrong_shape` document |
| `load_round_trips_the_full_credential_shape` | `let text = serialized_document(&stored_credentials())…;` → `parse_credentials(&text, <tempdir tokens.json path>)` → the same 5 asserts (replacing store.save/store.load with the direct pair) |
| `load_round_trips_null_refresh_token_and_empty_scope` | same direct round trip with `refresh_token = None` and empty scopes |

File-local helpers duplicated there (do NOT share): `ACCESS_TOKEN_FIXTURE`,
`REFRESH_TOKEN_FIXTURE`, `fn stored_credentials() -> StoredCredentials`
(identical bodies, ≤2 params convention). Imports:
`use std::path::PathBuf;`? only if a helper requires it — freeze: NOT needed;
`use std::fs;`, `use serde_json::Value;`? NOT needed (no raw `Value` asserts in
the moved set — test 1's `Value` checks pass on the same compact `text` string
via `serde_json::from_str::<Value>`? NO — test 1 already only uses
`document.as_object()` on the parsed `Value` it builds from `text` via
`serde_json::from_str`; keep that one `from_str::<Value>` inside the test and the
`use serde_json::Value;` import), `use super::super::authorization::REQUIRED_SCOPE;`,
`use super::*;`. Projected: module doc ~6 + imports ~6 + consts 3 + helper 10 +
moved bodies ~95 ≈ **120–135 lines**.

`src/youtube/file_credential_store_tests.rs` — RETAINS 10 tests UNTOUCHED
bodies: `save_is_atomic_replacing_in_place`,
`save_removes_a_stale_temp_file_and_survives_an_interrupted_save_shape`,
`load_returns_content_not_available_for_missing_file`,
`load_maps_directory_read_to_file_open`, `save_creates_a_missing_parent_directory`,
`save_fails_with_file_open_for_a_directory_parent_colliding_with_a_file`,
`target_directory_collision_maps_to_file_write`,
`stored_credentials_debug_reveals_no_secrets`,
`#[cfg(unix)] unix_mode_is_owner_only_after_save`,
`replace_overwrites_an_existing_target`. Helpers/constants `stored_credentials()`,
`store_at`, `read_document`, both fixtures stay. Import delta: `use
serde_json::Value;` stays (`read_document`), `use std::error::Error;` stays
(test 5), `use super::super::authorization::REQUIRED_SCOPE;` REMOVED (no
remaining user — helper's use moves to a local fixture copy? NO: the
`stored_credentials()` helper still uses it — RESTORE: the helper stays here and
keeps the import; the duplicated copied helper in persisted_credentials_tests.rs
carries its own import). Projected: 314 − (moved tests 1, 6, 7, 8, 9 ≈ 117
lines incl. adjacents) ≈ **195–198 lines**.

## A2. Projected line counts (hard gate at every step)

| file | projected | rule |
| --- | --- | --- |
| `src/youtube/file_credential_store.rs` | ≈ 190 | ≤ 200 |
| `src/youtube/persisted_credentials.rs` | ≈ 110 | ≤ 200 |
| `src/youtube/persisted_credentials_tests.rs` | ≈ 120–135 | ≤ 200 |
| `src/youtube/file_credential_store_tests.rs` | ≈ 195–198 | ≤ 200 |

Fallback (only if `cargo fmt` + `wc -l` shows any file > 200): additionally
absorb the store's `read_credential_text` 3-liner call site into `load` (save
2 lines) and/or trim the duplicated `stored_credentials()` helper's doc —
target ≤ 198 — then report the final `wc -l` per implementing report. NEVER
drop/rename a test.

## A3. Commit plan (overrules original §6 Step 3 wording)

SINGLE commit, message unchanged, carrying ALL of the split's files plus the
already-on-disk accessor pair:

- `feat: add the atomic file credential store`

Staged (git add, exactly 5 paths — token.rs/mod.rs edits are already on disk
from uncommitted work):
1. `src/youtube/file_credential_store.rs`
2. `src/youtube/file_credential_store_tests.rs`
3. `src/youtube/persisted_credentials.rs`
4. `src/youtube/persisted_credentials_tests.rs`
5. `src/youtube/mod.rs`

(token.rs's A1/A2 accessor changes were documented as part of the same Commit C
in amendment-1 A5; discard token.rs from the staging list only if the caller
explicitly says otherwise. `git status --porcelain` before staging must show no
other unintended paths; `.gitignore` compliance per `.kilo/rules/gitignore-compliance.md`.)

## A4. Verification steps for the implementer (after the split, before commit)

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt`
2. `cargo test --locked youtube` → exit 0, NO new/removed test names
   (28 youtube + 15 store == the same 43 youtube-module tests as the 195-test
   gate; grand total unchanged).
3. `cargo clippy --locked -- -D warnings` → exit 0.
4. `wc -l` on all 4 files + confirm ≤ 200 per the A2 fallback if needed.
5. `git add` per A3; `docker compose run --rm rust sh scripts/dev-checks.sh`
   fresh gate expected `ALL CHECKS PASSED`, 195 tests (unchanged count).
6. Report any cap/count deviation — never trim tests to fit.

## A5. Decision log

- Split is pure structural refactoring: identical behavior, messages, error
  variants, redaction, and test names/count.
- `PersistedCredentials` + all three helpers stay file-PRIVATE; only
  `serialized_document`/`parse_credentials` are `pub(crate)` — the private-
  members rule holds.
- §4.6's "never invent a 21st file" clause and D4's "private to
  file_credential_store.rs" wording are SUPERSEDED by this amendment; precedent
  granted: `.kilo/plans/20261012-phase04-task1-oauth-design-amendment-1.md`
  `google_token_endpoint` split.
- L1/L2/L4/L5 limitations, V5–V7 gitignore checks, §10 boundaries: UNCHANGED.

Plan file saved at `.kilo/plans/20261012-phase04-task2-oauth-config-storage-amendment-2.md`.

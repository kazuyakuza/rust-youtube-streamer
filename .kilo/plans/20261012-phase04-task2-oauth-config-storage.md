# Task 2 Plan — Phase 04: OAuth Configuration and Credential Storage

- Date: 2026-10-12
- Branch: `feat/phase04-oauth-credentials` (HEAD `ae37a62`, v0.7.0, tree clean)
- TODO authority: `.agent/todos/20261012/20261012-todo-1.md` — Task 2 "OAuth Configuration and Credential Storage" ONLY. Tasks 1/3/4/5 boundaries respected (§10).
- Global plan: `.kilo/plans/20261012-phase04-oauth-global-plan.md` §Task 2 pre-analysis.
- Task 1 base layer: `.kilo/plans/20261012-phase04-task1-oauth-design.md` §4.1/§4.5/§5 + `...-amendment-1.md` (credential shapes) + `...-task1-simplify.md` §4 (TokenEndpointBody flag).
- Implementer: JUNIOR developer, 50% restriction — no latitude beyond §10 "Explicitly NOT Done". Every decision below is locked; no judgment calls remain.

---

## 0. Task Pre-Analysis (baseline facts, verified this session)

| Fact | Source |
| --- | --- |
| `src/youtube/credentials.rs` (34 lines): `StoredCredentials { access_token: AccessToken, expires_in_seconds: u64, refresh_token: Option<RefreshToken>, granted_scopes: GrantedScopes }` (all `pub(crate)`, derived redaction-safe `Debug`) + `CredentialStore` trait (`load`/`save`) + `credentials_tests::stored_credentials_debug_reveals_no_secrets` | `src/youtube/credentials.rs`, `credentials_tests.rs` |
| `CredentialStoreError { FileOpen{path,source}, FileWrite{path,source}, ReadParse{path,message}, ContentNotAvailable }` — fixed enum, hand-written Display, `Error::source` Some only for the two io variants | `src/youtube/error.rs:45-50,129-190` |
| `GrantedScopes { scopes: Vec<String> }` — `from_space_delimited(&str)` + `grants(&str) -> bool` only; `scopes` field is private; `token.rs` is at 191 lines (≤200 cap) | `src/youtube/token.rs:154-172` |
| `TokenEndpointBody` is `#[derive(Debug, Deserialize)]` with a RAW `access_token: String` field → current simplifier's §4 flag: would print the raw token if ever formatted; simplifier rejected the change in Task 1, escalated it as "Task 2/3 hardening observation" | `src/youtube/token.rs:129-140`, simplify plan §4 |
| `AccessToken::new/secret()`, `RefreshToken::new/secret()` exist (redacting Debug `AccessToken(<redacted>)` etc.) | `token.rs` (macro-generated wrappers) |
| `src/config`: `AppConfig { youtube, video, renderer, chat, ffmpeg }`, serde derive `deny_unknown_fields`, no serde defaults anywhere, `load_from_path` 3-step read/parse/validate, aggregated `ConfigError::Validation { issues: Vec<ConfigIssue> }`, `blank_issue(field, value)` helper, module `pub(crate) mod model;` | `src/config/{model,validation,loader,error,mod}.rs` |
| Existing validation = 18 checks, NO file-existence checks anywhere (font/FFmpeg precedent) | `validation.rs`, project-structure.md |
| `config/config.example.json` tracked and mirrors the schema exactly; real `config/config.json` gitignored and never implicitly loaded | config/, cli.rs:14-17 |
| CLI surface is FIXED minimal: `auth|demo|run`, `--config <path>` (before/after command, ≤1 occurrence), `--help`; dependency note says a CLI crate is not added "when the standard library suffices"; changing the surface would alter `USAGE` text + exit-code contract | `src/app/cli.rs`, global plan |
| `.gitignore`: `/config/config.json`, `/config/config.windows.json`, `credentials/*` with `!credentials/.gitkeep` | `.gitignore:47-53` |
| `credentials/` tracked contents: ONLY `.gitkeep` | FS read + `git ls-files credentials/` |
| `mod.rs` policy: re-exports are added ONLY when the first external consumer appears (none exist yet); `#[allow(dead_code)] mod youtube;` staging line lives in `src/main.rs` and must NOT be removed here (Task 3 wires; retires there) | `src/youtube/mod.rs:19-26`, Task 1 plan §3/S7 |
| Deps locked: serde 1 (derive), serde_json, tracing, tracing-subscriber, ab_glyph, ctrlc, oauth2 5.0.0, ureq 2.12.x; dev-dep tempfile 3.14.0 (MUST NOT move) | `Cargo.toml`, context.md |
| Verification is container-only via Alpine VM MCP; container runs asroot — chmod-based permission-DENIAL tests are unreliable (root bypasses mode checks), so permission tests must assert SET bits, never denial | context.md Phase-03 notes |
| Error convention: hand-written `Display`, no `thiserror`, no panics/unwrap in library paths; file ≤200 lines, fn body ≤50 lines, depth ≤2, ≤2 params; no commented-out code; no new deps without justification | `.kilo/rules/*`, global plan |

Task 2 is the **configuration + concrete persistence step**: config schema delta, validation delta, and the concrete `FileCredentialStore`. No flow executes anywhere.

---

## 1. Research: cross-platform atomic-replace semantics with std only (verified facts)

1. **POSIX**: `std::fs::rename` atomically REPLACES an existing destination file (single syscall) — rename-within-same-directory is atomic w.r.t. other readers.
2. **Windows**: `std::fs::rename` maps to `MoveFileExW` withOUT `MOVEFILE_REPLACE_EXISTING` for `fs::rename`; when the destination exists it fails (Win32 `ERROR_ALREADY_EXISTS` / `ERROR_FILE_EXISTS`) → Rust classifies as `io::ErrorKind::AlreadyExists`. `std::fs::rename` does NOT replace files on Windows.
3. **Portable replace sequence** (used verbatim in §4.2): `rename(temp → target)`; if `Err` with `kind() == AlreadyExists` → `fs::remove_file(target)` then retry `rename` once. Between remove and rename a reader may see the file absent — the observable failure is a typed `ContentNotAvailable`/`FileOpen` on load, NEVER a truncated/invalid file. This satisfies TODO bullet 4 ("interruption does not leave a truncated valid credential file"): the credential file is only ever replaced by an atomic same-directory rename of a fully-written, `sync_all`-ed temp file.
4. **Temp-file placement**: the temp file MUST be in the SAME directory as the target (same filesystem → rename atomicity; cross-device rename fails with `EXDEV`). Exact naming: dot-prefixed sibling of the target: `.<file_stem><dot><extension?>.tmp` — implemented as `format!(".{}.tmp", file_name)` of the target's `file_name()` (e.g. `credentials/tokens.json` → `credentials/.tokens.json.tmp`). `.tmp` suffix additionally matches the existing `.gitignore` `*.tmp` line (belt-and-braces; the whole `credentials/` dir is ignored anyway).
5. **Stale temp cleanup**: a previous interrupted save may leave `.<name>.tmp` behind; `save` removes a pre-existing temp file BEFORE opening it (best-effort removal, failures flow into `FileOpen` mapping as below).
6. **Unix permission setting**: `std::os::unix::fs::OpenOptionsExt::mode(0o600)` at temp-file CREATION guarantees the new credential file never exists with wider permissions at any instant (satisfies TODO "restrict file permissions where supported", incl. the "before/instantly after first write" requirement). No post-create `set_permissions` step (the file was created with 0600 already; a second op adds no guarantee). Reconstruction case: if the target file already exists from a previous save, every save recreates a fresh 0600 temp — permissions stay restricted after every save.
7. **Windows limitation**: `std` has no ACL management; the more restrictive of the inherited ACLs applies. This is a REAL limitation — no encryption, no ACL hardening. Documentation duty (honest wording, no over-claim) lives in `file_credential_store.rs` module docs (this task) + README/`docs/authentication.md` notes (Task 5).
8. **serde_json error content risk**: `serde_json::Error` messages can embed offending values (e.g. `invalid type: string "..."`), i.e. raw token text could leak through `ReadParse { message }`. → the store MUST NOT copy `error.to_string()` into the variant (§4.4 D9).

---

## 2. Locked Decisions (numbered; all pre-made, none open)

### D1 — Config model shape: new top-level `oauth` section, required fields, exact names/defaults

- New struct in `src/config/model.rs`:
  ```rust
  /// OAuth client and credential file locations.
  #[derive(Debug, Clone, PartialEq, Deserialize)]
  #[serde(deny_unknown_fields)]
  pub struct OauthConfig {
      /// Path of the Google downloadable client-secret JSON file
      /// (`{"installed": {...}}`, Desktop-app client type). Only non-blank
      /// values are checked here; parsing belongs to the youtube module.
      pub client_config_path: String,
      /// Path of the application-persisted credential file (issued
      /// tokens). Only non-blank values are checked here; the file is
      /// created by the credential store.
      pub credentials_path: String,
  }
  ```
- `AppConfig` gains `pub oauth: OauthConfig,` between `youtube` and `video`.
- **Defaults**: default values are DOCUMENTED via the example file, not implemented with `serde(default)`: the existing config schema has zero serde defaults (all sections are mandatory in the JSON document) and `deny_unknown_fields` — adding a defaulting mechanism for two fields would fork the schema conventions. Required fields, example shows the canonical values.
- **Exact default/example values**:
  - `client_config_path` = `"credentials/client_secret.json"`
  - `credentials_path` = `"credentials/tokens.json"`
- **Justification for defaults**: (a) `credentials/` already exists as the designated "OAuth credentials and token storage location" (brief §4 structure, project-structure.md); (b) everything under `credentials/*` is already gitignored → secrets cannot be committed with NO `.gitignore` change (TODO bullet 3); (c) `credentials/tokens.json` is already the path-shaped fixture precedent used by `error_tests.rs::credential_path()`; (d) placing the client-secret file under `config/` would require a NEW gitignore rule (`/config/config.json` is a single-file rule) for a similarly secret file — rejected; OAuth material belongs in one gitignored folder.
- **Paths stay CONFIG-ONLY; no new CLI flags.** Justification vs TODO bullet 1 ("defaults or explicit CLI/config behavior consistent with existing conventions"): the CLI surface is documented as "fixed and minimal" (`cli.rs` module docs), the existing usage/exit-code contract is frozen (usage=2, config/log=1), and the brief §6.2/§6.3 wording is "OAuth client configuration and token storage paths must be configurable" — configurable is satisfied through the JSON config plus the existing `--config <path>` file indirection. Changing `USAGE`/CLI is a Task 3-owned surface decision with its own exit-code analysis; Task 2 adds no flags.

### D2 — Validation delta (exact additions in `src/config/validation.rs`)

- Two new non-blank checks via the existing `blank_issue` helper, wired into `validate_required_strings` (or a new `validate_oauth` function — REQUIRED: new `fn validate_oauth(config: &AppConfig, issues: &mut Vec<ConfigIssue>)` called from `validate()` between `validate_required_strings` and `validate_privacy_status`, issuing `blank_issue("oauth.client_config_path", …)` + `blank_issue("oauth.credentials_path", …)`; keeping the 10-line `validate_required_strings` under the 50-line body cap).
  - Field paths are dotted as everywhere else: `oauth.client_config_path`, `oauth.credentials_path`.
- **NO file-existence checks at validation time** (repo precedent: `renderer.font`, `ffmpeg.executable` are only blank-checked; the file's parse failure is a typed store error at use time — TODO bullet 2's "as required" honored via the non-blank rules).
- **New distinctness check** (decided, applied): the two OAuth paths must NOT be equal (case-sensitive string comparison, as-is, no canonicalization — cross-platform path canonicalization adds a Filesystem dependency we refuse); else one `ConfigIssue` on `"oauth.credentials_path"`: `"must be different from the OAuth client configuration path (the credential file would overwrite it)"`. Justification: with equal paths, the first atomic save would replace the operator's client-secret file — a silent destructive misconfiguration caught in one small rule. Comparison is exact-string (documented limitation: `./a` vs `a` aliases are not canonicalized; realistic operator config uses identical literal values for a real collision).
- Validation check count goes 18 → 21 (2 blank + 1 distinctness); validation.rs projected ≈ 165-175 lines — under the 200 cap.

### D3 — `FileCredentialStore` type and location

- New file `src/youtube/file_credential_store.rs` + sibling `file_credential_store_tests.rs`; `mod.rs` gains `mod file_credential_store;` (alphabetical, after `mod error;`), with the standard foot block `#[cfg(test)] #[path = "file_credential_store_tests.rs"] mod file_credential_store_tests;`.
- NOT inside `credentials.rs`: that file's documented role is types + trait (Task 1 plan §3 map: "types and the trait only"); a separate file keeps both under the per-file cap and matches the module's one-file-per-concern shape (amendment-1 A0 precedent).
- Type only; no re-export in `mod.rs` (first external consumer arrives with Task 3):
  ```rust
  /// Concrete credential store persisting JSON token data at one file
  /// path. Atomic writes (temp file + same-directory rename), Unix
  /// permission 0600; see the module docs for the Windows ACL
  /// limitation.
  #[derive(Debug)]
  pub(crate) struct FileCredentialStore {
      path: std::path::PathBuf,
  }

  impl FileCredentialStore {
      /// Store targeting the given credential-file path (typically the
      /// configured `oauth.credentials_path`).
      pub(crate) fn new(path: std::path::PathBuf) -> Self { ... }
  }

  impl CredentialStore for FileCredentialStore {
      fn load(&self) -> Result<StoredCredentials, CredentialStoreError> { ... }
      fn save(&self, credentials: &StoredCredentials) -> Result<(), CredentialStoreError> { ... }
  }
  ```
  (`derive(Debug)` is safe: `PathBuf` carries no secret.)

### D4 — Persisted-file schema (exact)

Compact internal serde type, private to `file_credential_store.rs`:

```rust
/// JSON document shape of the persisted credential file. Secret VALUES
/// are the file's legitimate content (the file itself is the vault),
/// mirrored one-to-one from StoredCredentials; this type exists so
/// stored credentials serialize with fixed field names and never via a
/// derived Debug/serde path that could drift from the documented schema.
#[derive(Deserialize, Serialize)]
struct PersistedCredentials {
    access_token: String,
    /// Remaining lifetime of `access_token` in seconds at save time
    /// (TTL semantics preserved from Task 1: no absolute timestamps are
    /// persisted; clock use belongs to the Task 4 credential service).
    expires_in_seconds: u64,
    /// Serialized as `null` when absent (Option semantics, no skip).
    refresh_token: Option<String>,
    /// Google's space-delimited scope list, exactly as the token
    /// endpoint reports it (empty string when no scope was granted).
    granted_scopes: String,
}
```

- **Field names**: `access_token`, `expires_in_seconds`, `refresh_token`, `granted_scopes` (snake_case literal fields — matches `TokenResponse` field naming).
- **`granted_scopes` is a single space-delimited STRING**, not a JSON array. Justification: it lets load reconstruct via the EXISTING `GrantedScopes::from_space_delimited` — zero additions to `token.rs` (191 lines; two added methods would push it over the 200-line cap), and the format mirrors Google's own documented `scope` wire field. An empty granted set persists as `""` and reloads to an empty set (never an error — Task 1 plan D1 semantics preserved).
- **`refresh_token` absent ⇒ `null`** in the file (no `skip_serializing_if`; fixed 4-key schema, one exact shape, schema-stable).
- **Pretty vs compact JSON — DECIDED: compact** (`serde_json::to_string`). Justification: the file is machine-written and machine-read; pretty-printing adds no operator value here (operators are steered to the store's typed errors, never encouraged to hand-edit tokens), and compact keeps the file exactly one JSON document with no formatting contract to maintain. Reconstruct-on-load is tolerant to extra whitespace anyway (it is a serde parse), so future hand-inspection of the file is not broken by compact output.
- Save round trip: `SaveInput::from(Loaded credentials) → PersistedCredentials → to_string → bytes → rename`. Load round trip: `read_to_string → from_str::<PersistedCredentials> → StoredCredentials { access_token: AccessToken::new(..), refresh_token: RefreshToken::new(..) opt, granted_scopes: GrantedScopes::from_space_delimited(&p.granted_scopes), expires_in_seconds }` — secret values never pass through any `format!`/log/panic site (D8).

### D5 — `save` exact algorithm (single doc block for the implementer; bodies split ≤50 lines)

```
save(credentials):
  1. parent = self.path.parent().unwrap_or(Path::new(".")) — no unwrap in code:
     self.path.to_path_buf().components() based fallback decided as:
     use parent of the normalized path via `PathBuf` composition; if
     `self.path.parent()` is None (bare filename), the parent is "." (a
     `Path::new(".")` const; expressed with match, never unwrap/expect).
     fs::create_dir_all(parent) — on failure: CredentialStoreError::FileOpen {
         path: self.path.clone(), source }  [decided: errors always carry the
         FINAL configured path in `path`; the io::Error's own Display includes
         the OS-level offending path, see D7]
  2. temp_path = parent.join(format!(".{}.tmp", target_file_name))
     target_file_name = self.path.file_name() — None (path ends in `..`) is a
     configuration defect reachable at runtime: map to FileOpen { path, source: a
     fixed io::Error::new(InvalidInput, "credential path has no file name") }
     (decided: explicit typed failure, no panic).
  3. FS best-op: let _ = fs::remove_file(&temp_path);   (stale temp cleanup; a
     failure here is harmless — open() will surface real failures)
  4. Open temp file: OpenOptions::new().write(true).create_new(true)
     {+ .mode(0o600) on unix} — `create_new` (not `create`) makes a
     temp-collision race a typed failure instead of a silent overwrite.
     Open failure → FileOpen { path: self.path.clone(), source }.
  5. serde_json::to_string(&PersistedCredentials::from(credentials)) → write_all
     → sync_all. write/serialization failure → FileWrite { path: self.path, source }
     ("serialization" errors: serde_json::to_string on a fully-owned struct cannot
     fail; map it through the same FileWrite arm into the io wrapper decided in D7 —
     construct io::Error::new(io::ErrorKind::InvalidData, "credential serialization failed"))
  6. drop(file); then sync_replace(&temp_path, &self.path)  (§1 item 3; behavior identical across platforms).
     Rename/replace failure (incl. the Windows AlreadyExists remove+retry tail) →
     FileWrite { path: self.path.clone(), source } + best-op temp cleanup
     `let _ = fs::remove_file(&temp_path);`
```

### D6 — `sync_replace` helper (exact std-only code)

```rust
/// Atomically replaces the target file with the prepared temp file.
/// POSIX: rename alone replaces the destination. Windows: a rename over
/// an existing file fails with AlreadyExists, so the two-step remove +
/// rename is the required fallback there; a same-directory replace
/// never leaves a truncated credential file behind.
fn sync_replace(temp_path: &Path, target_path: &Path) -> Result<(), io::Error> {
    match fs::rename(temp_path, target_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            fs::remove_file(target_path)?;
            fs::rename(temp_path, target_path)
        }
        Err(error) => Err(error),
    }
}
```

(function has 2 params; body ≤ 12 lines; the guard is a single-section boolean condition.)

### D7 — Error carry-over mapping (fixed mapping table)

| Event | Variant | `path` field | Note |
| --- | --- | --- | --- |
| Parent dir create fail | `FileOpen` | FINAL configured path | `source` = io::Error |
| Path has no file name | `FileOpen` | FINAL configured path | synthetic `InvalidInput` io::Error |
| Temp open fail | `FileOpen` | FINAL configured path | path in errors is always the operator-facing target (D5 note) |
| Write / sync / serialization fail | `FileWrite` | FINAL configured path | includes partial temp write; temp best-effort removed |
| Rename / replace fail | `FileWrite` | FINAL configured path | incl. Windows remove+retry tail |
| load: file absent (NotFound) | `ContentNotAvailable` | — | variant has no path; Display = "no stored credentials are available; run authorization first" — exactly the right user guidance |
| load: other read error | `FileOpen` | actual store path | e.g. target is a directory (`IsADirectory`) |
| load: unparsable / wrong-schema JSON | `ReadParse` | actual store path | **message is a FIXED safe string** (D8) |
| load: schema mismatch (wrong types/missing/unknown) | `ReadParse` | actual store path | fixed safe string |

### D8 — Redaction guarantees for `ReadParse.message` (decided exact behavior)

`serde_json::Error` text can quote offending VALUE strings → never stored. Load maps parse failures to two fixed descriptive strings, keyed on stable serde_json predicates:

```rust
fn parse_failure_message(error: &serde_json::Error) -> String {
    if error.is_syntax() || error.is_eof() {
        "the credential file is not valid JSON".to_string()
    } else {
        "the credential file does not match the expected credential schema".to_string()
    }
}
```

(`is_syntax`/`is_eof` are stable since 1.53; data/IO-classified errors and any unknown classification land in the schema-mismatch message. NO `serde_json::Error` Display text survives this function.) This is the TODO bullet 5 guarantee for the store boundary ("errors and logs never expose … full token endpoint response bodies"): a malformed credential JSON that embeds token text can never surface it.

### D9 — `TokenEndpointBody` hardening (carry-over flag resolution — exact edit)

- `src/youtube/token.rs:129-141`: change the derive line from
  `#[derive(Debug, Deserialize)]` to `#[derive(Deserialize)]` on `TokenEndpointBody` and amend its doc comment to add:
  `/// The raw-string secret fields intentionally make `Debug` unavailable:`
  `/// accidental formatting in a future call site would print tokens; a`
  `/// compile error is the stronger guard (Task 2 hardening, simplify §4 flag).`
- Justification for "remove the derive" over "hand-write redacted Debug": `TokenEndpointBody` is a function-local parse intermediate whose raw `String` fields are the ONE type in the tree that still derives printability over raw secrets; removal turns any future `format!("{body:?}")`/`dbg!`/log line into a compile error rather than relying on an ever-maintained redacted impl. Removing also frees a line in a 191-line file. No current call site formats the type (verified: it escapes nothing; test bodies never `dbg!` it). TokenEndpointDebug absence has no compile impact today (nothing derives/consumes it).
- Cause it to stay testable: token tests untouched (6 tests keep passing; none asserts Debug on the body — verified by reading `token_tests.rs` shape; implementer must confirm with `cargo test --locked youtube` before committing).

### D10 — No logging in Task 2

The store emits NO tracing events (no `info!`/`debug!` lines), mirroring Task 1 (logging belonged to real flows, Task 3+). The module docs state that its failure reporting is via returned typed errors only. TODO bullet 5 is therefore structurally satisfied for this task's code: nothing is ever logged whose provenance is not already upstream.

### D11 — `.gitignore` / `credentials/README.md` policy for Task 2

- `.gitignore` needs NO edits (D1 defaults fall under the existing `credentials/*` rule; V5 commands verify this). Task 5 does the tracked record anyway.
- `credentials/README.md` is NOT created in Task 2 (docs are Task 5's explicit bullet; creating an under-specified README now would have to be rewritten there). `credentials/.gitkeep` stays tracked and untouched. The Windows-ACL limitation note is written into `file_credential_store.rs` module docs now and surfaced in Task 5's documentation pass.

---

## 3. File Map (exact actions)

```
src/config/model.rs                 EDIT  — +OauthConfig struct + AppConfig field (D1)
src/config/validation.rs            EDIT  — +validate_oauth fn + distinctness rule (D2)
src/config/test_fixtures.rs         EDIT  — valid_config() gains the oauth section (default values)
src/config/{validation,loader}_tests.rs  EDIT — new test cases (§6)
config/config.example.json          EDIT  — +oauth section (§4 delta, exact JSON)
src/youtube/token.rs                EDIT  — one derive-line edit + doc comment (D9)
src/youtube/mod.rs                  EDIT  — +`mod file_credential_store;` footer-test decl pair
src/youtube/file_credential_store.rs      NEW  (D3–D8)
src/youtube/file_credential_store_tests.rs NEW (§6)
.gitignore                          NO EDIT (D11)
credentials/                        NO EDIT (D11)
src/app/**, src/main.rs, Cargo.toml, Cargo.lock  NO EDIT
```

Projected line counts (all ≤200): model.rs ≈ 136, validation.rs ≈ 170, file_credential_store.rs ≈ 165-185 (target; split helpers per D5 if growing), token.rs 191 → ~192, test files sized per §6.

---

## 4. Exact Code Snippets (implementer fills bodies per the given skeletons; these ARE the design)

### 4.1 `src/config/model.rs` — struct text (see D1 rust block)

`AppConfig` gains one field and section placed FIRST-edited:

```rust
pub struct AppConfig {
    pub youtube: YouTubeConfig,
    /// OAuth client and persisted-credential file locations.
    pub oauth: OauthConfig,
    pub video: VideoConfig,
    ...
}
```

Struct doc comments for `OauthConfig`/its two fields as drafted in D1. Doc comment for the `AppConfig` youtube/oath ordering is not commented — names are self-documenting.

### 4.2 `src/config/example-file delta` (§4 of the deliverable)

`config/config.example.json` gains exactly this section between `"youtube"` and `"video"`, keeping the file's 2-space indentation:

```json
  "oauth": {
    "client_config_path": "credentials/client_secret.json",
    "credentials_path": "credentials/tokens.json"
  },
```

(35 → 40 lines; file mirrors the model schema — an omitted/misplaced section must fail the schema tests below.)

### 4.3 `test_fixtures.rs` delta

`valid_config()` gains the same two default strings:

```rust
        oauth: OauthConfig {
            client_config_path: "credentials/client_secret.json".to_string(),
            credentials_path: "credentials/tokens.json".to_string(),
        },
```

(importing `OauthConfig` in the fixture use list.)

### 4.4 `file_credential_store.rs` module doc (DECIDED — must appear verbatim-ish in the file)

```rust
//! Concrete `FileCredentialStore`: JSON credential persistence at one
//! configured path.
//!
//! Durability: `save` writes a fully-serialized temp file in the SAME
//! directory and atomically renames it onto the target; on Windows the
//! rename-over-existing step fails with AlreadyExists and the store
//! falls back to remove + rename (the only std-only portable
//! replacement). An interrupted save can therefore leave a stale temp
//! file (removed by the next save) but never a truncated or
//! half-written credential file.
//!
//! Permissions: temp files are created with Unix mode 0600, so every
//! credential file lands with owner-only access on Linux/macOS.
//! Windows file ACLs are NOT hardened by this store (std provides no
//! ACL management): the file inherits the directory's ACL restrictions.
//! This is a documented limitation, not encryption; the credential file
//! is no more protected than its containing directory's defaults.
//!
//! Redaction: parse failures report fixed descriptive messages (never
//! `serde_json`'s value-quoting text) and errors always name the
//! operator-facing target path, so no secret value can reach a log or
//! an error line through this module. No tracing events are emitted
//! here — failure reporting is the returned typed error only.
```

### 4.5 `load` algorithm (fixed)

```
load():
  1. fs::read_to_string(&self.path) failure:
       NotFound → Ok-mapped? no: Err(ContentNotAvailable)
       other    → Err(FileOpen { path: self.path.clone(), source })
  2. serde_json::from_str::<PersistedCredentials>(&text):
       failure → Err(ReadParse { path: self.path.clone(),
                                 message: parse_failure_message(&error) })
  3. Ok(StoredCredentials {
         access_token: AccessToken::new(p.access_token),
         expires_in_seconds: p.expires_in_seconds,
         refresh_token: p.refresh_token.map(RefreshToken::new),
         granted_scopes: GrantedScopes::from_space_delimited(&p.granted_scopes),
     })
```

### 4.6 `save` signature and helper split (bodies stay ≤50 lines even combined — helpers required)

```rust
impl PersistedCredentials {
    fn from_stored(credentials: &StoredCredentials) -> Self { ... }   // 1 param
}

impl FileCredentialStore {
    fn temp_path(&self) -> Result<PathBuf, CredentialStoreError> { ... } // 0 params; encodes D5 step 2
    fn open_temp_file(&self, temp_path: &Path)
        -> Result<std::fs::File, CredentialStoreError> { ... }       // 2 params
    fn write_temp_file(&self, file: std::fs::File, credentials: &StoredCredentials)
        -> Result<(), CredentialStoreError> { ... }                  // 2 params
}

fn sync_replace(temp_path: &Path, target_path: &Path) -> Result<(), io::Error> { ... } // D6
```

(The parent/directory prep lives at the top of `save` — keep `save`'s body ≤ 50 lines; extract `ensure_directory(&self) -> Result<(), CredentialStoreError>` when the limit would be crossed. Follow file ≤200 lines; move `PersistedCredentials` + `parse_failure_message` into the same file only while the cap holds, else split a second `file_credential_store`-adjacent private module is NOT allowed — instead trim doc comments, never invent a 21st file — report overgrowth to the caller instead.)

---

## 5. Test Plan (all deterministic, offline, no Google, no network, no wall clock)

All tests live in `src/youtube/file_credential_store_tests.rs` (store) and `src/config/{validation,loader}_tests.rs` (schema); the store tests use `tempfile::TempDir`, exactly like every other in-tree unit module. Root-proof rule: container tests run as root — NO chmod-denial assertions (bits are asserted `#[cfg(unix)]`, denials are not asserted).

### Config tests (schema delta)

| File / test name (exact) | Asserts |
| --- | --- |
| `validation_tests.rs::oauth_paths_require_non_blank_values` | fixture with both blank / whitespace-only paths → `Validation` error containing both field names in one aggregated Display |
| `validation_tests.rs::oauth_paths_must_be_distinct` | both paths identical → one `ConfigIssue` naming `oauth.credentials_path`; two distinct paths → no issue |
| `loader_tests.rs::example_mirrors_oauth_schema` (or extend an existing round-trip test) | the committed `config/config.example.json` parses through `load_from_path` and yields the exact documented default path strings |
| `loader_tests.rs::unknown_oauth_field_rejected` | `{"oauth": { ... "bogus": true }}` → `Deserialization` (deny_unknown_fields propagates) |
| `loader_tests.rs::oauth_section_missing_rejected` | fixture JSON without the `oauth` member → `Deserialization` |

Existence of these three loader variants: implement in whichever of the existing loader test fns the fixture builder makes natural — the three BEHAVIOR assertions above are mandatory; the dispatcher wiring detail follows the file's existing helper style.

### Store tests (all in `file_credential_store_tests.rs`)

| # | Test name (exact) | Asserts |
| --- | --- | --- |
| 1 | `save_writes_the_documented_schema` | sorted-assert on the FILE TEXT: contains the four member names in the fixed order (`access_token` first, compact JSON via `from_str::<InvokeOrder>` equivalence: build `serde_json::Value::from_str` and assert the 4 keys exactly — `as_object().len() == 4`, exact member set, `expires_in_seconds` value round-trips; $contains$ the fixture access/refresh values ONLY as raw file bytes, never in any error string) |
| 2 | `save_is_atomic_replacing_in_place` | save valid credentials twice with different tokens between saves; after each save the on-disk file parses to exactly the latest save's values (no old token remnants; proves rename-style replacement, not appends/patching) |
| 3 | `save_removes_a_stale_temp_file_and_survives_an_interrupted_save_shape` | pre-create `<temp name>` (the exact `.<basename>.tmp` name) containing garbage; run save; assert target file is fully valid JSON of the new value, the stale temp file is GONE |
| 4 | `load_returns_content_not_available_for_missing_file` | store pointed at a path that does not exist → `Err(ContentNotAvailable)`; `format!("{e}")` = the stable "no stored credentials are available" line |
| 5 | `load_maps_directory_read_to_file_open` | store pointed AT A DIRECTORY → `Err(FileOpen { .. })`, error `source()` is `Some(_)` |
| 6 | `load_reports_least_details_for_malformed_json` | file containing `"{ not json"` → `ReadParse { message == "the credential file is not valid JSON" }` (exact string; asserts no fixture token text appears in the message) |
| 7 | `load_reports_fixed_schema_mismatch_message` | file containing valid JSON but wrong shape (`{"nope": 1}`) → `ReadParse` with the schema mismatch fixed string; again assert the file's own contents cannot appear |
| 8 | `load_round_trips_the_full_credential_shape` | save → load; assert: access/refresh wrappers hold the EXACT fixture secrets via `secret()`, `expires_in_seconds` exact, granted scope membership (`granted_scopes.grants(REQUIRED_SCOPE)`), and `granted_scopes` count 1 |
| 9 | `load_round_trips_null_refresh_token_and_empty_scope` | save credentials with `refresh_token: None` + `granted_scopes: GrantedScopes::from_space_delimited("")` → load → `refresh_token.is_none()`, `grants(REQUIRED_SCOPE) == false` |
| 10 | `save_creates_a_missing_parent_directory` | store at `tmp/nested/credentials/tokens.json` (parent chains missing) → save succeeds; `tokens.json` exists |
| 11 | `save_fails_with_file_open_for_a_directory_parent_colliding_with_a_file` (POSIX: write a FILE at the would-be parent path) | create-dir-all fail → `Err(FileOpen)` on the FINAL path |
| 12 | `target_directory_collision_maps_to_file_write` (POSIX) | target path is an existing DIRECTORY → save → `Err(FileWrite)`; stale temp cleaned |
| 13 | `stored_credentials_debug_reveals_no_secrets` (already in `credentials_tests.rs` — ALSO now extended coverage) | the Loaded credentials built by `load` of a file written by `save` renders `<redacted>` for both wrappers (i.e. round-trip load reconstruction is redaction-preserving) |
| 14 | `unix_mode_is_owner_only_after_save` `#[cfg(unix)]` | after save, `fs::metadata(target).permissions().mode() & 0o777 == 0o600` (set-bit assertion, root-safe) |
| 15 | `replace_overwrites_an_existing_target` | direct `sync_replace` call over a pre-existing target file: destination content is fully replaced and temp file no longer exists (exercises the POSIX arm of D6; the Windows AlreadyExists arm is exercised by the same fn on Windows and CANNOT be executed in the Linux container — recorded in §9 as a documented not-container-testable limitation) |

Fixture conventions: `ACCESS_TOKEN_FIXTURE = "access_token_fixture"`, `REFRESH_TOKEN_FIXTURE = "refresh_token_fixture"`, scope via `use super::super::authorization::REQUIRED_SCOPE;` (S8 precedent); no `dbg!`/`panic!` on values that carry secrets outside the wrapper; test helper count matched to ≤2 params per fn (helpers used where more inputs are required).

---

## 6. Implementation Order for the Implementer (with exact commit points)

All work on `feat/phase04-oauth-credentials`. Host is Windows: every cargo action goes through the Alpine VM MCP (`alpine-vm_vm_status` first, then `alpine-vm_vm_run_command`). Standard fast-loop per step: `cargo fmt` (fold rewrap into the step's commit), `cargo fmt --check`, `cargo clippy --locked -- -D warnings`, `cargo test --locked <module filter>`.

### Step 1 — Config schema + validation + example

- Edits: `src/config/model.rs` (D1), `src/config/validation.rs` (D2), `src/config/test_fixtures.rs` (§4.3), `config/config.example.json` (§4.2), plus the 5 config tests (§5 first table) appended via the existing `#[path]` test modules.
- Verify: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked config` exit 0; then `cargo check --locked` exit 0 (modes/fixture consumers still compile — the fixture change keeps every existing test passing).
- Gitignore compliance: `git status --porcelain` must list EXACTLY: the 6 paths above.
- Commit A: `feat: add oauth path settings to the application configuration`

### Step 2 — TokenEndpointBody hardening (D9)

- Edit `src/youtube/token.rs` as D9.
- Verify: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked youtube` exit 0 (all existing youtube tests pass unchanged).
- Commit B: `fix: make the token endpoint body unformattable`

### Step 3 — FileCredentialStore

- New files `src/youtube/file_credential_store.rs` + `file_credential_store_tests.rs` per §4 + D3–D8; `src/youtube/mod.rs` adds the two lines (`mod file_credential_store;` alphabetical between `mod error;` and `mod google_token_endpoint;`).
- Verify: `cargo test --locked youtube` exit 0 (15 new store tests + all pre-existing 27 preserved by-name).
- Commit C: `feat: add the atomic file credential store`

### Step 4 — Full gate + trailing verification (fresh evidence) 

- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → must exit 0: PASS fmt-check/check/test/clippy, `ALL CHECKS PASSED`; record the observed test total (147+21 = 168 expected: 147 Phase-03 baseline + Task-1's 27 youtube tests already counted? NO — 147 ALREADY INCLUDES the 27 youtube tests; expected total = 147 + 5 config + 15 store = 167). Record ACCURATELY in the implementing report; record the fresh log path.
- V5 gitignore verification commands (below); if any fails, DISCARD the step and report the deviation to the caller (do not edit `.gitignore` — D11 says none is needed; treat ignorance of a rule as a deliverable error).
- `git grep -n "#\[allow(" src/` → exactly the one staging line in `src/main.rs` (unchanged).
- `wc -l` on all touched/created files → all ≤ 200.

### Step 5 — commit hygiene / final state (docs/plans tracking is the caller's step)

- Working tree must end with only this task's planned files tracked/committed; no tmp dirs anywhere in the repo tree; no `config/config.json` or `credentials/*.json` file created on the HOST tree by tests (temp files only under `tempfile`-managed temp dirs — the store never writes to the repo tree).

---

## 7. Verification Commands (V-series; run via Alpine VM MCP, exit status authoritative)

| # | Command | Expected |
| --- | --- | --- |
| V1 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` | exit 0 |
| V2 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked` | exit 0 |
| V3 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked` | exit 0; test count grows 27-youtube + config-additions; ALL PASS |
| V4 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` | exit 0 |
| V5 | (host, after V4) `git check-ignore -v credentials/client_secret.json credentials/tokens.json config/config.json` | exactly 3 rule hits: `.gitignore:52` (credentials/*) twice and `.gitignore:48` once |
| V6 | (host) `git status --porcelain` | only intended files, listed explicitly |
| V7 | (host) `git ls-files credentials/` | `.gitkeep` only — no tokens.json etc.; no newline-escaped real paths ever staged |
| V8 | dev-checks gate (Step 4) | exit 0 + `ALL CHECKS PASSED` + test count as recorded |

## 8. Rule Cap Checklist (enforced at every step + final)

| Rule | Compliance action |
| --- | --- |
| ≤200 lines/file | projection §3; `wc -l` in Step 3+4 |
| ≤50-line fn bodies | helper split per D5 (save/temp/replace/write helpers); test fns stay small |
| ≤2 definition depth (nest) | match arms ≤ 2 levels; helpers used at 3rd |
| ≤2 function params (receiver excluded) | all helpers at 1–2 params; request structs if a third input ever appears (none planned) |
| No panics/unwrap/expect in library paths | D5's two "no file name"-style unwraps replaced by explicit matches; tests may unwrap |
| Hand-written Display; no thiserror | store adds NO new Display (uses Task 1 error enums) |
| No new deps | std-only; `serde` features untouched; NO chrono/time |
| Single-section boolean conditions | D6 guard + blank/distinctness helpers are single conditions |
| No commented-out code; self-documenting names | §4 comments are docs, not dead code |
| Newline rule | multi-line edits use real newlines in the editor, never `\n` literals |

---

## 9. Risks / Documented Limitations (pre-registered)

- **L1** — D6's Windows `AlreadyExists` retry branch cannot execute in the Linux container test suite (POSIX rename replaces silently). Coverage = the POSIX arm (test 15) + the fn's structure; record as a container-only-verification limitation in the implementing report; do NOT claim native Windows store verification.
- **L2** — tests run as root → permission-DENIAL behaviors are untestable by chmod; only SET-bit assertions are used (§5 note).
- **L3** — D1 distinctness comparison is exact-string, not path-canonicalized (`./a` vs `a` aliases are not normalized); documented in the validation snippet and in the example-file docs.
- **L4** — no Windows ACL hardening (std limitation); the honest limitation text lives in the module doc now and in Task 5's docs; never claim encryption.
- **L5** — `credentials/.gitkeep` rule hides nothing else; if an operator configures paths OUTSIDE `credentials/` (e.g. `./tokens.json`), the secret file is not gitignored — the example/docs (Task 5) steer to `credentials/`; a generic protective `.gitignore` change is a Task 5 decision, NOT made here.
- **L6** — Expected test total shoes 147 + 5 (config) + 15 (store) = 167; if any COUNT deviates from 167 at Step 4, report so, do not "adjust tests to fit".

## 10. Explicitly NOT Done by Task 2 (boundaries)

- NO `auth`/`run` behavior change: `src/app/modes.rs`, `src/app/cli.rs`, `src/app/mod.rs`, `src/main.rs` untouched; `--config` is the only file-path CLI generalization (exit codes / usage text unchanged); NOT_IMPLEMENTED_EXIT = 3 for both placeholders stays.
- NO authorization flow wiring, NO listener/state/PKCE changes (Tasks 3), NO refresh logic, NO network/transport changes, NO scope-verification request (Task 4).
- NO changes to `src/youtube/{transport,google_token_endpoint,authorization,client_config,verify}.rs` — the ONLY `src/youtube` edits are token.rs's one derive line (D9), a two-line `mod.rs` declaration, and the two new store files.
- NO `.gitignore`, `README.md`, `docs/**`, `.agent/**` file edits (Task 5 docs cycle).
- NO new dependencies, no `Cargo.toml`/`Cargo.lock` change, no `tempfile` version movement.
- NO tracing events added; no additional logging anywhere.
- NO credentials/clients-secret files written into the repo tree; no real secrets anywhere.

## 11. Plan Self-Check vs TODO Task 2 bullets

- ✔ "Smallest justified configuration model … with defaults consistent with existing conventions" → D1 (one new section, two field paths, exact example defaults; no CLI surface changes — justified).
- ✔ "Update config.example.json, validation, tests, docs as required; never load example implicitly" → §4.2, D2, §5 config tests; example remains unreferenced at load time (the loader test explicitly parses it by test path only).
- ✔ "Keep client secrets and tokens out of version control; preserve/verify gitignore rules" → D1/D11 + V5-V7.
- ✔ "Atomic persistence where practical; restrictive file permissions where supported; test/document behavior" → §1/§4.5/D5/D6, test 14, limitations L1/L4 documented in module docs; README duty deferred to Task 5's explicit docs bullet.
- ✔ "Errors and logs never expose secrets, tokens, auth codes, or full token endpoint response bodies" → D8 fixed strings, D10 no logging, D9 compile-error guard, redaction tests 6/7/13.
- Plan file saved at `.kilo/plans/20261012-phase04-task2-oauth-config-storage.md` — the only file this sub-step has created.

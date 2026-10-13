# Task 2 Adherence Report (sub-step 4.5b) — Phase 04: OAuth Configuration and Credential Storage

- Date: 2026-10-12
- Verdict: **ADHERENT** (no unacceptable findings)
- Branch: `feat/phase04-oauth-credentials`; HEAD `09833cc`; version `0.7.0`; tree clean except the untracked planner file `.kilo/plans/20261012-phase04-task2-review-fix.md` (left untouched).
- Review range: `ae37a62..09833cc` (7 commits: `4d0d637` config schema/validation/example, `c133827` TokenEndpointBody hardening, `24d05fd` store, `42d33fd` amendment tracking, `3997eec` review-fix, `09833cc` docs, plus the planning-doc-only commit `cfc0cc4`).
- Governing documents: Task 2 plan `20261012-phase04-task2-oauth-config-storage.md`, amendment-1 (`GrantedScopes::to_space_delimited`), amendment-2 (`persisted_credentials` split), review-fix plan `20261012-phase04-task2-review-fix.md`; TODO `.agent/todos/20261012/20261012-todo-1.md` Task 2 only.
- Working method: all plan D1–D11 decisions adjudicated against the on-disk source at HEAD; fresh container evidence ran on the full HEAD tree (see §4).

---

## 1. Evidence Matrix (plan decision → observed implementation)

| Ref | Required (plan) | Observed at HEAD | Status |
| --- | --- | --- | --- |
| D1 | New top-level `OauthConfig { client_config_path, credentials_path }`, `deny_unknown_fields`, required (no schema defaults), `AppConfig.oauth` between `youtube` and `video`; defaults documented via example | `src/config/model.rs:42-63` struct + one 7-line doc-comment pair matching D1 text (extended by text-only commit `09833cc`); field placed `model.rs:15` between `youtube` and `video` | ✔ |
| D1 defaults | `client_config_path="credentials/client_secret.json"`, `credentials_path="credentials/tokens.json"` (config-only; no new CLI flags) | `config/config.example.json:9-12` exact values, `oauth` section between `youtube` and `video`, 2-space indent; `src/app/**` untouched (diff-stat confirms zero CLI changes) | ✔ |
| D1 no implicit example load | Example is never implicitly real configuration | Loader tests load it only via an explicit test path (`loader_tests.rs:104`); runtime default remains `config/config.json` (`cli.rs` doc line 15 unchanged); no production source references the example path | ✔ |
| D2 (i) | New `fn validate_oauth` with two `blank_issue` calls on dotted paths, called between `validate_required_strings` and `validate_privacy_status` | `src/config/validation.rs:96-112`, wired at `validation.rs:24` exactly in that position; dotted field names exact | ✔ |
| D2 (ii) | Distinctness check: exact-string comparison, one `ConfigIssue` on `"oauth.credentials_path"` with the fixed message | `validation.rs:106-111` + `OAUTH_PATH_COLLISION_PROBLEM` const (`:13-14`) = plan message verbatim | ✔ |
| D2 (iii) | No file-existence checks at validation time | Confirmed: the only oauth rules are the two blanks + distinctness | ✔ |
| D2 count | 18 → 21 validation checks | Counted: 3 positive numerics + pixel_format + font_size + line_height + vertical_layout + 10 required strings + privacy_status (18 pre-existing) **+2 oauth blanks +1 distinctness = 21** | ✔ |
| D3 | New store type as separate file(s), `pub(crate)`, `#[derive(Debug)]` (PathBuf only), `mod.rs` mod line, test-foot block | `src/youtube/file_credential_store.rs:45-55` (`pub(crate)`, `Debug` over `PathBuf` only); `mod.rs:39,41` single mod lines (alphabetical); test-foot per in-file convention (sanctioned deviation); split into `persisted_credentials.rs` per amendment-2 | ✔ |
| D4 | Fixed 4-key schema `access_token`/`expires_in_seconds`/`refresh_token`/`granted_scopes`; scopes as ONE space-delimited string; `refresh_token` null when absent (no skip); compact JSON | `src/youtube/persisted_credentials.rs:27-39` struct matches D4 text (incl. field docs); `granted_scopes: String`; `Option<String>` without `skip_serializing_if`; `serde_json::to_string` (compact) at `:57`; save uses `to_space_delimited` (`:50`) per amendment-1 | ✔ |
| D5 | save: create_dir_all → FileOpen{final path}; `.name.tmp` temp; stale-temp removal; `create_new` open (+0o600 unix) → FileOpen; write+sync → FileWrite; rename/replace failure → FileWrite + best-op temp cleanup; no unwrap for missing file_name (fixed `InvalidInput`) | `file_credential_store.rs:137-144` save body in direct order (create_dir_all → temp_path → stale removal → open → write → replace); `:84` `format!(".{}.tmp", …)`; `:140` best-op removal; `:115-124` failure cleanup; `:77-83` no-file-name maps to FileOpen with `io::ErrorKind::InvalidInput`, fixed message — no unwrap/expect | ✔ |
| D6 | Exact `sync_replace` (rename; AlreadyExists → remove + retry rename) | `file_credential_store.rs:168-177` — verbatim equivalent (single-section guard) | ✔ |
| D7 | Error mapping: FileOpen always carries the FINAL configured path; load NotFound → `ContentNotAvailable`; other read errors → FileOpen; parse/schema → ReadParse | `file_credential_store.rs:57-64, 88-95, 97-113, 115-124` all map with `self.path.clone()`; `:147-156` `map_read_error` (NotFound→ContentNotAvailable, else FileOpen with the actual store path); `persisted_credentials.rs:63-74` ReadParse with store path | ✔ |
| D8 | ReadParse carries FIXED safe strings only, keyed on `is_syntax`/`is_eof` | `persisted_credentials.rs:85-91` (`"the credential file is not valid JSON"` / `"the credential file does not match the expected credential schema"`); helper `is_malformed_json` extraction is the sanctioned deviation; tests assert the exact strings and no fixture-token leak | ✔ |
| D8/D10 | No secrets in any error/log site; no tracing events in Task 2 | No `tracing`/`info!`/`debug!` in either new file (module doc only states none are emitted); no `format!` of token values outside tests' redaction assertions; TokenEndpointBody `#[derive(Deserialize)]` only (token.rs:133) with the D9 doc comment (`:130-132`) → stray formatting is a compile error | ✔ |
| D10 | Store emits no tracing events; failures via typed errors only | Verified by read + grep | ✔ |
| D11 | `.gitignore` NOEDIT; no `credentials/README.md` in Task 2 | `.gitignore` absent from range diff; `git ls-files credentials/` → `.gitkeep` only | ✔ |
| §10 boundaries | No `src/app/**`, `src/main.rs`, `.gitignore`, `Cargo.toml`/`Cargo.lock`, `credentials/**` changes | Full `git diff --stat ae37a62..09833cc` (§3) lists only the 15 source/config files + 3 planning docs; none of the boundary paths appear | ✔ |
| Docs commit hygiene | `09833cc` text-only | Full diff reviewed: 6 files, all changes are `//!`/`///` doc-comment lines or doc-comment rewires; zero behavior/identity/logic changes | ✔ |
| Test surface | 5 config names + 15 store names + 1 round-trip name preserved | Log confirms `oauth_paths_require_non_blank_values`, `oauth_paths_must_be_distinct`, `example_mirrors_oauth_schema`, `unknown_oauth_field_rejected`, `oauth_section_missing_rejected`, 15 `youtube::file_credential_store/persisted_credentials` tests, and `granted_scopes_round_trip_preserves_exact_membership` — all passing (§4) | ✔ |
| Review-fix | Aggregated-blank test shape = review-fix plan body verbatim | `validation_tests.rs:123-131` matches the required change exactly (both fields blanked in one fixture, both names + `2 issue(s)` asserted) | ✔ |

## 2. Sanctioned-Deviation Log (adjudicated ACCEPTABLE)

| Deviation | Source | Adjudication |
| --- | --- | --- |
| `mod.rs` single-line wiring + test foot inside each source file (per-file foot, not mod.rs) | amendment-2 A0 | ACCEPTABLE (sanctioned; verified on disk) |
| `is_malformed_json` helper extracted from plan D8's inline condition | implementer / amendment-2 A1.1 | ACCEPTABLE (pure refactor of the same predicates; single-section boolean keeps `parse_failure_message` shape) |
| `to_space_delimited` (amendment-1) vs the original "no accessor" path | amendment-1 A1 | ACCEPTABLE (token.rs land at exactly 200/200) |
| `CredentialStoreError` import path (`super::error`) | implementer | ACCEPTABLE (same enum, plan §0 lists the path) |
| Token wrapper imports in store tests via `super::super::token` | implementer | ACCEPTABLE (module-private wiring; no behavior surface) |
| Commit C contents (:24d05fd carries store + split + wiring; :42d33fd tracks amendments) | amendment-2 A3 | ACCEPTABLE (sanctioned single-commit shape for steps 1+2 of A5; amendment tracking is a planner-only doc commit) |
| GrantedScopes accessor + `granted_scopes_round_trip_preserves_exact_membership` | amendment-1 A2 | ACCEPTABLE (on disk, in the gate) |
| `persisted_credentials` module split (persisted schema moved; `serialized_document`/`parse_credentials` pub(crate)) | amendment-2 A1 | ACCEPTABLE (schema text/keys/messages byte-identical; store retained io-only) |
| Aggregated-blank test fix `:3997eec` | review-fix plan | ACCEPTABLE (implemented per the required change verbatim; total stayed 195) |
| Docs commit `09833cc` text-only afterwards | caller record | ACCEPTABLE (verified text-only against the full diff) |
| `config.example.json` is 39 lines, not the projected 40 (no trailing blank counted differently) | measuring detail | ACCEPTABLE (schema content exact; the schema-validity assertions live in the loader tests) |
| token.rs exactly 200/200 heavily at the cap | amendment-1 | ACCEPTABLE (cap holds; noted, not a finding) |

## 3. Unacceptable Findings

None. No source, behavioral, boundary, redaction, naming, count, or gitignore deviation beyond the sanctioned record above.

## 4. Fresh Container Evidence (Alpine VM MCP; `vm_status` run first: running, SSH reachable)

- **dev-checks gate**: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0; four PASS lines (`fmt-check`, `check`, `test`, `clippy`), summary `ALL CHECKS PASSED`.
- **Test total**: `test result: ok. 195 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out` — exactly the post-review-fix count (rejecting neither trim nor inflation).
- **Log path**: `logs/checks/20261013T011250Z.log` (gitignored `logs/checks/`).
- **Per-file line counts (container `wc -l`, all ≤ 200)**: all of `src/youtube/*` — authorization 125(+tests 148), client_config 98(+104), credentials 35(+23), error 194(+195), file_credential_store **199**(+195), google_token_endpoint 198(+38), mod 44, persisted_credentials **106**(+137), token **200/200**(+122), transport 48(+110), verify 35; config changed files — model 141, validation 171, loader_tests 198, validation_tests 160, test_fixtures 50.
- **`git grep "#\[allow("` (container grep on `src/`)** — exactly the 4 pre-existing items, zero new: `src/chat/model.rs:44` (`ChatMessage::with_timestamp`), `src/chat/store.rs:123` (`ChatStore::is_empty`), `src/main.rs:10` (youtube staging), `src/streaming/video_output.rs:27` (`OutputKind::Session`).
- **Boundary grep (range `ae37a62..09833cc`)**: `git diff --stat` full file list contains NO `src/app/**`, `src/main.rs`, `.gitignore`, `Cargo.toml`, `Cargo.lock`, or `credentials/**` entries — mode/cli/main touched by NO commit in the range (none expected; verified stronger than pathspec grep because the stat covers every path).
- **Gitignore rules (host `git check-ignore -v`)**: exactly 3 rule hits as required by V5 — `credentials/client_secret.json` and `credentials/tokens.json` → `.gitignore:52:credentials/*` (×2), `config/config.json` → `.gitignore:48:/config/config.json`.
- **Tracked credentials contents (V7)**: `git ls-files credentials/` → `.gitkeep` only.
- **Tree hygiene (V6)**: `git status --porcelain` → `?? .kilo/plans/20261012-phase04-task2-review-fix.md` only (untracked planner file; untouched per instruction).

## 5. TODO Task 2 Bullet Traceability

- Bullet 1 (smallest config model, defaults consistent) → D1 landed exactly; no CLI changes.
- Bullet 2 (example/validation/tests updated; never implicitly loaded) → §4.2/D2/§5 tests landed; example load is test-only.
- Bullet 3 (secrets out of source control; gitignore preserved/verified) → D11 holds; §4 evidence (V5/V7).
- Bullet 4 (atomic persistence, restrictive permissions, test/documented) → D5/D6 + `unix_mode_is_owner_only_after_save`; Windows-ACL and Windows-rename-arm limitations documented in module docs (`file_credential_store.rs:10-13,17-20`).
- Bullet 5 (errors/logs never expose secrets/tokens/codes/token bodies) → D8 fixed messages (tests `load_reports_least_details_for_malformed_json`, `load_reports_fixed_schema_mismatch_message`, `stored_credentials_debug_reveals_no_secrets` all passing); D9 compile-error guard; D10 no logging.

Report file saved at `.kilo/plans/20261012-phase04-task2-adherence.md`. No source files modified; no commits made.

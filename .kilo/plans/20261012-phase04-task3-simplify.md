# Task 3 Simplification Plan — Phase 04 `auth` Command

- Date: 2026-10-12
- Sub-step: 4.3 code-simplifier (plan generation only; NO source file was modified by this plan)
- Baseline: branch `feat/phase04-oauth-credentials`, HEAD `8a50093`, v0.7.0, tree clean; simplification scope = Task 3 diff `9ebf13f..8a50093`
- Authority: `.kilo/plans/20261012-phase04-task3-auth-command.md` (D1–D16) — every locked surface below is preserved with ZERO deviation
- Verdict: SIMPLIFICATION REQUIRED — small, mechanical, contract-neutral (3 unlocked steps + 1 gated step)
- Implementer: JUNIOR developer, 50% restriction — each step below is fully specified; no judgment calls required or allowed

## 0. Preservation contract (applies to every step)

The following are FROZEN and must not change in any way. All steps below operate strictly inside them:

- `AuthError` variant set/order/payloads (D1) — including the `#[allow(dead_code)]` on `CallbackAbandoned { attempts }` (mod.rs:79) and the explanatory comment (mod.rs:76–78). The Task-4-reserved allow set everywhere (`git grep -n "#\[allow(" src/` → 24 hits incl. the 21 Task-3/youtube allows) is DELIBERATE staging: never add, remove, or restructure any `#[allow(`.
- Every Display message constant and arm output string (D2) — byte-identical, including the three verbatim-delegation arms (they CANNOT be merged; see §3).
- Listener semantics/constants/timeouts/budgets (D6/D7), console output lines (§4.5), exit-code mapping (D3), logging event list + field whitelist (D15), redaction guarantees (D16).
- The 24 auth test names (M1–M4, C1, F1–F11, L1–L8) and the 194 non-auth test names — byte-identical, no add/remove/rename. Assertion STRENGTH unchanged: no weakened test, no removed assertion.
- The deliberate `assert!(a.secret() == b)` style in tests (flow_tests.rs:117/149/151, flow_exchange_tests.rs:36-37/184/186, listener_tests.rs:94/113/145/179): NOT a simplification target — `assert_eq!` would print values into panic output and break the §5 secrecy note. Do NOT "fix" these.
- `src/youtube/` sources: out of bounds (plan §10 — re-exports + narrow allows only; the cfg(test) re-export block in `youtube/mod.rs` is required because the submodule is private; leave).
- File caps after every step: ≤200 total lines; fn body ≤50; depth ≤2; ≤2 non-receiver params; single-section booleans.

## 1. Step S1 — `src/app/auth/flow.rs`: drop two needless clones in `code_exchange_request`

Effect: production path performs two redundant heap allocations. `opened` is owned by value (moved in, consumed — `OpenedFlow` has no Drop impl and is not used after the call at flow.rs:105), so `code` and `redirect_uri` can move instead of clone. `client_secret().clone()` MUST stay (borrowed from `&ClientConfig`).

Exact edit — `src/app/auth/flow.rs`, inside `code_exchange_request` (function at lines 155–163; lines 159–160):

```rust
// FROM
        code: opened.code.clone(),
        redirect_uri: opened.redirect_uri.clone(),
// TO
        code: opened.code,
        redirect_uri: opened.redirect_uri,
```

`CodeExchangeRequest` field mapping stays exactly D8 (identical values). Projected: flow.rs 171 → 171 lines; zero allocations removed from every exchange call.

## 2. Step S2 — `src/app/auth/flow.rs`: `storeable_credentials` match → `ok_or`

Exact edit — `src/app/auth/flow.rs`, lines 137–140:

```rust
// FROM
    let refresh_token = match response.refresh_token {
        Some(refresh_token) => refresh_token,
        None => return Err(AuthError::MissingRefreshToken),
    };
// TO
    let refresh_token = response.refresh_token.ok_or(AuthError::MissingRefreshToken)?;
```

Semantics unchanged: the scope check (134–136) still runs first; `MissingRefreshToken` still returned on `None`; the `info!("required YouTube scope granted")` at line 141 and the field moves at 142–147 stay exactly as-is (`Some(refresh_token)` on line 145 keeps the bound name). Partial move of `response` is valid (`Option::ok_or` consumes the field by value).

Projected: flow.rs 171 → 168 lines (combined with S1: 168).

## 3. Step S3 — `src/app/auth/flow_tests.rs`: merge the write-fixture + parse pair into one helper

Effect: the two-line sequence "write fixture client file, then read_client_config(...).expect" is duplicated verbatim in F1 (lines 114–115) and F11 (lines 176–177). `write_fixture_client_file` (92–96) has exactly those two callers. Net −2 lines on a file at 199/200.

Exact edits — `src/app/auth/flow_tests.rs`:

1. Replace the whole `write_fixture_client_file` function (lines 92–96) with:

```rust
fn parsed_client_fixture(directory: &tempfile::TempDir) -> ClientConfig {
    let client_path = directory.path().join("client_secret.json");
    std::fs::write(&client_path, google_shaped_client_json()).expect("the fixture file is written");
    read_client_config(&client_path.to_string_lossy()).expect("the fixture client parses")
}
```

2. In F1 (`client_file_happy_parse_returns_client_config`), replace lines 114–115 (`let client_path = write_fixture_client_file(&temp_dir);` + `let client_config = read_client_config(...).expect("file parses");`) with:

```rust
    let client_config = parsed_client_fixture(&temp_dir);
```

3. In F11 (`declined_provider_flow_ends_without_persisting_end_to_end`), replace lines 176–177 (the same two-line pair) with:

```rust
    let client_config = parsed_client_fixture(&temp_dir);
```

F1's two assertions, F11's remaining body, both test names, and every other helper stay untouched. Projected: flow_tests.rs 199 → 197 lines.

## 4. Step S4 — GATED (caller/reviewer decision; SKIP by default) — `src/app/auth/mod_tests.rs` M3 table collapse

M3 (`auth_error_display_lines_are_stable`, lines 77–130) repeats ten `assert_eq!(displayed[N], "<continuation-wrapped text>")` blocks. A single ordered `Vec<&str>` of the same ten snapshots + one `assert_eq!(displayed, expected)` removes the boilerplate (projected 146 → ~132 lines; the ten texts move byte-identically).

GATED because plan §5.1 specifies M3 as "per-variant `assert_eq`/starts_with snapshot" and a concurrent adherence reading may count the collection assert as a structural deviation, while the file has headroom and the gain is cosmetic-only. Default action: do NOT execute S4 unless the caller explicitly approves it. If approved, keep the variant ORDER identical to D2/D1 and add `let expected: Vec<&str> = vec![ ... ];` above a single `assert_eq!(displayed, expected);` (type equality `Vec<String> == Vec<&str>` holds).

## 5. Considered and NOT pursued (do not revisit in this cycle)

- Collapsing the three verbatim Display arms (`ClientConfig`/`TokenExchange`/`Persistence`, mod.rs:113/125/126) into an or-pattern arm: INFEASIBLE — an or-pattern binding requires one type; the arms carry three distinct error types.
- Shared browser-thread test helper across `flow_tests`/`listener_tests` (F11's `spawn_declined_browser` vs L-tests' `spawn_browser`/`get_request_bytes`): NOT pursued — they sit in different test-module subtrees (`auth::flow::flow_tests` vs `auth::listener::listener_tests`); sharing requires a new `#[cfg(test)]` support module, a structural change outside the D4 file table (medium redesign → out of a simplifier's authority).
- `flow_under_test`'s `OnceLock<AppConfig>` (flow_exchange_tests.rs:108): keep — it exists precisely to stay within the 2-param cap while handing out an `AuthFlow<'a>`; any alternative widens params or lifetimes.
- `#[cfg(test)]` re-exports in `src/youtube/mod.rs` (AccessToken/GrantedScopes/RefreshToken/RefreshRequest): keep — the youtube submodules are private; without them the app-layer fakes cannot name these types. Widening the modules to `pub(crate) mod` would increase the API surface (violates prefer-private-members).
- `assert!(… == …)` secret comparisons: keep (see §0).
- The compression-commit's doc-header rewraps in `token.rs`/`file_credential_store.rs`: keep — they are what holds those files at ≤200.

## 6. Execution order, verification, hygiene

Steps are independent; apply S1 → S2 → S3 (S4 only if explicitly approved). One commit per step, on `feat/phase04-oauth-credentials`:

1. `chore: move opened-flow fields into the exchange request without cloning` (S1)
2. `chore: simplify the refresh-token option unwrap in storeable_credentials` (S2)
3. `test: share the parsed client fixture helper across F1 and F11` (S3)
4. (only if S4 approved) `test: collapse the M3 display snapshots into an ordered table`

Verification after EACH step (Alpine VM MCP; `alpine-vm_vm_status` first; exit status authoritative):

- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked app::auth`
- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings`

If `cargo fmt` rewraps, fold the rewrap into the same step's commit (Phase-03/04 precedent; rustfmt reflow ≠ cap breach).

Final gate after the last executed step:

- `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0, `ALL CHECKS PASSED`; the observed test total MUST equal the pre-simplification total (no test added/removed); record the fresh `logs/checks/*.log` path.
- Host: `wc -l src/app/auth/flow.rs` ≤ 168 (S2) / ≤170 with S1; `flow_tests.rs` ≤ 197; every auth file ≤ 200.
- Host: `git grep -n "#\[allow(" src/` → still exactly the same 24 hits as at HEAD `8a50093` (no allow added/removed).
- Host: `git status --porcelain` shows only the intended step files (gitignore compliance: nothing from `credentials/`, `config/config.json`, `logs/`, `dist/`, `target/`).

Expected behavior equivalence proof: S1/S2 production paths are covered by F4/F6 (validation outcomes), F7 (request field values — captured client_id/redirect_uri/code/verifier comparisons are value-equal whether cloned or moved), F8/F9/F10, L2–L7 unchanged; M2/M3 unchanged prove mod.rs untouched surfaces. No console line, log event, exit code, socket constant, or Display string is altered by any executed step.

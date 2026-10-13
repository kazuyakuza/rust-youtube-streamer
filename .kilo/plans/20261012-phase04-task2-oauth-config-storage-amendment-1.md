# Task 2 Plan — Amendment 1: `GrantedScopes::to_space_delimited`

- Date: 2026-10-12
- Branch: `feat/phase04-oauth-credentials`, HEAD `c133827` (clean)
- Amends: `.kilo/plans/20261012-phase04-task2-oauth-config-storage.md` (D4/§4.6 blocker only)
- Authority: `.agent/todos/20261012/20261012-todo-1.md`, Task 2 ONLY.

---

## A0. Blocker restated (verified this session)

D4 requires `PersistedCredentials::from_stored(&StoredCredentials)` to emit
`granted_scopes` as one space-delimited `String`. `GrantedScopes`
(`src/youtube/token.rs:159-175`, file currently **194 lines**) exposes only
`from_space_delimited(&str)` and `grants(&str)`; the `scopes` field is private
with no accessor, so serialization from `file_credential_store.rs` is
impossible without touching `token.rs`. Zero-rewrite alternatives (re-export
the field, expose the raw `Vec`, change the persisted schema to a JSON array)
are all rejected: they break D4's fixed 4-key schema, D8's redaction shape, or
encapsulate the collection. The accesssor is the minimal fix.

## A1. DECIDED — one accessor added to `src/youtube/token.rs`

Exact addition, placed inside `impl GrantedScopes` immediately AFTER the
`grants` method (one blank line before it; nothing else in the impl moves):

```rust
    /// Serializes back to Google's space-delimited scope format; the
    /// credential store uses this to persist granted scopes (plan §4.6).
    pub(crate) fn to_space_delimited(&self) -> String {
        self.scopes.join(" ")
    }
```

- `pub(crate)` only — the private-members rule holds; no external surface.
- Semantics mirror `from_space_delimited` exactly: single-space join of the
  stored scope strings in their stored order; empty set serializes to `""`
  (D4's empty-set round-trip rule). `from_space_delimited(to_space_delimited(x))`
  preserves the exact-match set (split_whitespace/`join(" ")` are inverse on
  space-delimited tokens; no scope can contain whitespace — Google scope URLs).
- Line budget: 194 + 6 inserted lines (blank + doc 2 + fn 3) = **exactly 200**
  → within the ≤200 cap. `cargo fmt` must not rewrap; if post-fmt the file
  measures 201, shorten the doc comment to ONE line:
  `/// Space-delimited form for the credential store's persistence round trip.`
  (→ 199). Report the final `wc -l` in the implementing report.

## A2. DECIDED — one new test in `token_tests.rs`

One test IS required: the accessor is new public-behavior surface and its
round-trip identity is its specification.

- File: `src/youtube/token_tests.rs` (the `#[path]`-wired test module of
  `token.rs`).
- Exact name: **`granted_scopes_round_trip_preserves_exact_membership`**
  (verified against all existing youtube test names — no collision).
- Asserts (single test, body ≤ 15 lines):
  1. `GrantedScopes::from_space_delimited("a b c")` (use two real-looking
     scope URLs + a third) → `to_space_delimited()` returns the exact input
     string (`assert_eq!`).
  2. Round trip: `from_space_delimited(&original.to_space_delimited())`
     grants the same members and denies the same non-members as the original
     (`grants()` on 1 member + 1 non-member), i.e. membership identity.
  3. Empty set: `from_space_delimited("")` → `to_space_delimited() == ""`
     (D4's empty-string persistence rule is exercised here, so the store
     tests need no extra test for it).
- token_tests.rs grows 106 → ≈ 120 lines (cap holds). Youtube test count:
  the 27 existing youtube tests + 1 = **28**; the Step-4 expected grand total
  becomes **168** (was 167 per original plan §6 Step 4 / L6). Record
  accordingly; all other counts unchanged (5 config + 15 store tests).

## A3. Superseded constraint

- Original plan D4 line "zero additions to `token.rs`" is **SUPERSEDED** by
  this amendment: token.rs receives exactly ONE addition beyond D9 — the A1
  accessor (+ the A2 test in its test module). D9's one derive-line + doc
  edit is already landed and remains the only other token.rs touch. §10's
  "NO changes to `src/youtube/{transport,...}`" etc. are unchanged; the
  token.rs entry in the §3 File Map now reads:
  `src/youtube/token.rs EDIT — D9 (landed) + one GrantedScopes accessor (A1)`.

## A4. All other Task 2 decisions unchanged

D1–D3, D5–D11, §4 snippets, the 15-test store table, the 5-test config table,
V-series, cap checklist, and §10 boundaries are UNCHANGED by this amendment.
Only D4's `granted_scopes` reconstruction path is amended: save side uses
`GrantedScopes::to_space_delimited(&p.granted_scopes)`; load side keeps the
existing `GrantedScopes::from_space_delimited(&p.granted_scopes)` (§4.5 step 2
unchanged).

## A5. Implementer order for the REMAINING Task 2 work

1. **token.rs accessor (A1)** + the A2 test in `token_tests.rs`.
   Verify: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked youtube` → exit 0, 28 youtube tests pass; `cargo fmt` + `wc -l src/youtube/token.rs` ≤ 200.
2. **`file_credential_store.rs` + `file_credential_store_tests.rs` + `mod.rs` wiring** exactly per original plan Step 3 (D3–D8, §4.4–§4.6, 15-test table; `save`'s serialization now uses `credentials.granted_scopes.to_space_delimited()`).
   Verify: `cargo test --locked youtube` exit 0 (28 + 15 store tests); `wc -l` all touched files ≤ 200.
3. **Commits** with the original plan's messages:
   - Commit: `feat: add the atomic file credential store` (original Step 3 Commit C) — carries steps 1+2 together (the accessor is D4's enabling piece of the same deliverable; no separate commit message is authorized).
4. **Full gate + trailing verification** per original Step 4 + V-series (expected grand total 168 per A2; V5–V7 gitignore checks unchanged).
5. Report any cap/count deviation to the caller — never trim tests to fit.

# Phase 07 Task 4 — Review Fixes (4.3 reviewer pass)

## Defect found

| # | Location | Defect | Severity | Fix owner |
|---|---|---|---|---|
| D1 | `.agent/reports/20261029-phase-07-streamlist-migration.md` §9 hygiene audit table and `.kilo/plans/20261029-phase07-task4-verification-docs.md` L.6 | The `ya29`/`AIza` `.agent` grep count is recorded as **15**, but the actual count is **16** because the Phase-07 completion report's own hygiene-audit row quotes the grep pattern. | Low — documentation truthfulness only; no token literal is leaked and the security property (zero hits in `src`/`docs`/`README.md`) holds. | 4.4 docs-specialist / implementer |

## Junior-executable fix instructions

### D1 — Correct the token-pattern grep count

1. In `.agent/reports/20261029-phase-07-streamlist-migration.md` §9, locate the hygiene-audit row:

   ```text
   | `git grep -nE "ya29\|AIza" -- src docs README.md .agent` | 15 pre-existing lines, all historical audit-prose mentions in `.agent` (unchanged count vs HEAD: 7 + 2 + 5 + 1); ZERO in `src`/`docs`/`README.md`; zero new |
   ```

   Replace the observed-result cell with:

   ```text
   16 hits in `.agent`: 15 pre-existing historical audit-prose mentions (unchanged count vs HEAD: `context.md` 7, Phase-04 report 2, Phase-05 report 5, Phase-06 report 1) plus 1 self-referential hit from this audit row's own quoted grep pattern; ZERO in `src`/`docs`/`README.md`; zero new token literals.
   ```

2. In `.kilo/plans/20261029-phase07-task4-verification-docs.md` L.6, update the same count/breakdown to match:

   ```text
   `git grep -nE "ya29\|AIza" -- src docs README.md .agent` → 16 hits in `.agent` (15 pre-existing historical audit-prose mentions: `context.md` 7, Phase-04 report 2, Phase-05 report 5, Phase-06 report 1; plus 1 self-referential hit from the Phase-07 completion report's hygiene table quoting the grep pattern); ZERO in `src`/`docs`/`README.md`; zero new token literals.
   ```

3. Optionally align `.kilo/plans/20261029-phase07-task4-verification-docs.md` §E.4 first bullet so it no longer asserts the stale "ZERO hits" expectation over the `.agent` scope; record the observed shape instead.

4. Verification commands (must pass before closing):

   ```bash
   grep -rnE 'ya29|AIza' -- .agent | wc -l   # → 16
   grep -rnE 'ya29|AIza' -- src docs README.md | wc -l   # → 0
   ```

No source-code changes are required.

## Reviewer verification matrix

| Scope item | Expected / plan claim | Verified | Notes |
|---|---|---|---|
| **Burst test fidelity** (`chat_worker_burst_tests.rs`) | Full 64-cap prefill; burst items retained and delivered in order after capacity frees; zero sleeps; registration block after `chat_worker_budget_tests`; only these two files touched. | PASS | Targeted `cargo test --locked chat_worker_burst` → `1 passed; 493 filtered out`. Test asserts prefill drains first, then `burst-1`/`burst-2`, no terminal, no extra queued. `send_stop_aware` uses `try_send` + retain-retry; test-build `sleep_slice` is a no-op. |
| **`chat_worker.rs` line count / registrations** | ≤ 200 lines; `#[cfg(test)]` registration block present; gate compiles 494 tests. | PASS | File is 187 lines. Registration block at lines 181–183. Full gate `494 passed / 0 failed` (`logs/checks/20261029T203729Z.log`). `cargo fmt --check` clean. |
| **Docs — `production-run.md`** | 15 encoded edits C.a.1–C.a.15 applied; byte-stable zones untouched; no invented per-event billing. | PASS | Overview, log row 7, startup step 10, prerequisites, expected behavior, latency, diagnostics, failure-recovery table, quota subsection all match §C.a. Quota math is 5 units/connection, ≤ 11 calls ≤ 55 units, unknowns listed honestly. Step 6 + optional 6b landed. |
| **Docs — `README.md`** | Test count 494; versions tonic 0.13.1 / prost 0.13.5 / tokio 1.53.1; production-run sentence updated. | PASS | Count matches gate. Versions match `Cargo.lock`. Tokio/gRPC bullets present. |
| **Docs — `google-cloud-prerequisites.md`** | Two sanctioned truth-restorations (row 44 + bullet 140); no still-true fact deleted. | PASS | Row 44 now names `streamList` selected and `list` retired. Bullet 140 explains per-connection cost. Line 130 table fact unchanged. |
| **Docs — `tech.md` confinement** | ONE runtime site in `src/app/run/chat_worker.rs` + one in tests; select-biased stop tick; 10 s connect bound. | PASS | `tokio::runtime` appears in 2 files (`chat_worker.rs`, `stream_test_fixtures.rs`), 3 lines. `CHAT_STREAM_CONNECT_TIMEOUT_MS = 10_000` in `src/youtube/stream_client.rs`. `chat_stream_worker.rs` uses `tokio::select! { biased; ... sleep(stop_tick_slice_ms()) ... }`. |
| **Docs — `context.md` Phase-07 block** | Gate 494/0, log path, artifact sizes `12336636` B Windows / `7665712` B Linux, `ls -l`-proven; operator-confirmation note with `logs/run/20261029-153416.log`; duplicate-`1.` next step. | PASS | Block at lines 37–43 contains exact numbers. `ls -l` cross-check on VM confirms sizes. Phase-06 report addendum inserted after Manual Verification. |
| **Docs — `project-structure.md`** | Run 31 files / 58 tests; YouTube 92 files / 231 tests; dead-code inventory 8 + 1 clippy; no comment-only mention. | PASS | `git ls-files src/app/run | wc -l` → 31; run test count → 58. `git ls-files src/youtube | wc -l` → 92; YouTube test count → 231. `allow(` in `src` → 9 (8 dead_code + 1 manual_clamp). |
| **Truthfulness sweep** | No stale claim that polling is selected beyond historical/probe contexts and the honest retired-surface wording; no secrets. | PASS with D1 | `grep -rnE 'ya29|AIza' -- src docs README.md` → 0. `key=` in `src docs README.md` → 5 (doc comments + negative assertions). `rtmps://` hits are only fixtures/doc-context. The only doc-truth issue is the off-by-one `.agent` grep count. |
| **Hygiene re-observe — commit stats** | File sets per commit match claims; `Cargo.toml`/`Cargo.lock` untouched by Task 4. | PASS | `git show --stat` confirms commit file sets. `git diff ca9c55f..c160ad5 -- Cargo.toml Cargo.lock` is empty. `dist/`/`videos/` untracked. |
| **§L addendum** | All observed records carried truthfully. | PASS with D1 | Gate/build sizes/counts, structure numbers, secret-grep counts, anchor repairs, and deviations are recorded. Only the `ya29`/`AIza` count needs the D1 correction. |

---

Fix plan written: `.kilo/plans/20261029-phase07-task4-review-fixes.md`

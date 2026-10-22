# Simplification Plan — Task 1 (HTTP 411 fix): 4.3 code-simplifier output

Plan date: 2026-10-22 · Author: Code-Simplifier Agent (Critical Workflow step 4.3, SIMPLIFICATION half)
Scope reviewed (Task-1-changed files ONLY, per commits `f9d91ac` + `7ccd319`):
`src/youtube/google_live_api_transport.rs`, `src/youtube/api_request.rs`,
`src/youtube/google_live_api_transport_bind_wire_tests.rs`,
`src/youtube/google_live_api_transport_bind_checks_tests.rs`,
plus the three metadata files of `7ccd319` (`.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md`).
Binding inputs: `.kilo/plans/20261022-fix-http-411-task1-implementation.md` (D1–D12, §5.1c arm shape, §5.3/§5.4 test specs),
`.kilo/plans/20261022-fix-http-411-bind-content-length.md`, TODO `.agent/todos/20261022/20261022-todo-1.md` Task 1.

---

## 1. Verdict summary

- 2 micro-edits to APPLY (both in `google_live_api_transport_bind_wire_tests.rs`): drop two needless whole-head `String` copies in the capture helpers. Behavior-identical, plan-neutral, clippy-safe. Exact hunks in §2.
- Everything else in the changed set is ADJUDICATED LEAVE-AS-IS (§3) — each candidate was measured against the binding contracts (D1–D12, §5.1c arm shape, §5.3/§5.4 test specs, ≤2 params, ≤50-line bodies, ≤200-line files, max depth 2, no new `allow`) and every alternative form loses a rule or the plan-conformance. The recorded precedent "capture-helper duplication across sibling test files IS the plan's chosen form" was respected (cap analysis in §3.4).
- 3 deferred candidates for FUTURE work recorded in §4 (NOT to be executed with this fix; caller/Planner decides; no TODO file was created by this agent per the overwrite-todo-file-prevention rule).
- The 4.2 implementer tree is otherwise clean: file caps hold (transport 187 ≤ 200; wire 172 ≤ 200; checks 94 ≤ 200; api_request 158 ≤ 200).

## 2. Steps to APPLY (junior-executable; NO judgment calls required)

Both steps are in file `src/youtube/google_live_api_transport_bind_wire_tests.rs` (172 lines at HEAD `7ccd319`).

### Step 1 — `record_request`: keep the lossy decode borrowed

Find the single line (current file line 111, inside `fn record_request`):

    let text = String::from_utf8_lossy(&bytes).to_string();

Replace it with exactly:

    let text = String::from_utf8_lossy(&bytes);

Do NOT touch any other line. Line 113 (`text.lines().next().unwrap_or_default().to_string()`) and line 114 (`header_value(&text, "content-length")`) stay byte-unchanged — both consume `text` through `Deref<Target = str>` / deref coercion, which the `Cow<str>` returned by `from_utf8_lossy` provides; that is why no other edit is needed.

### Step 2 — `read_body_tail`: keep the head re-decode borrowed

Find the single line (current file line 120, inside `fn read_body_tail`):

    let head_text = String::from_utf8_lossy(head).to_string();

Replace it with exactly:

    let head_text = String::from_utf8_lossy(head);

Do NOT touch line 133 (`String::from_utf8_lossy(&body_bytes).to_string()`) — there the owned `String` IS needed (it is the returned `ServedRequest.body` value).

### Why these two and nothing bigger

`String::from_utf8_lossy` returns `Cow<'_, str>`; for valid-UTF-8 bytes (the HTTP request head) it is already a borrow, so the trailing `.to_string()` forces a full second heap copy of the captured request head — once per `record_request`, once per `read_body_tail`. The `header_value` double-parse across the two helpers CANNOT be folded away without a 3rd parameter (the legacy sibling `google_live_api_transport_tests.rs::read_body` does exactly that with 3 params — a shape the new file is forbidden to copy by the D12/≤2-params rule), so the re-parse stays; making both re-decodes allocation-free is the maximal fix the rule allows. Test names, assertions, wire facts and the enforcing-server logic are untouched — all D7/§5.3/§5.4 contracts byte-identical.

### Verification after applying (exact commands)

Container commands go through the Alpine-VM MCP: call `vm_status` FIRST, then `alpine-vm_vm_run_command`, ONE command per call (no chains):

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` → expect exit 0.
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked google_live_api_transport_bind` → expect the 3 D7 tests pass (`running 3 tests` / `test result: ok. 3 passed; 0 failed`). This is the true arbiter for this change: it compiles the `cfg(test)` code, where the Cow substitution lives.
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` → expect exit 0 (gate form; test-code lints are compiled only via step 2's build — no new warnings expected).
4. Full standard gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → expect exit 0, four PASS lines, `ALL CHECKS PASSED`, recorded log path under `logs/checks/`, and the OBSERVED test count (expect 360 — observed governs, per D7).
5. Host, workdir `C:\repo\rust-youtube-streamer`: `git status --porcelain` → expect exactly ` M src/youtube/google_live_api_transport_bind_wire_tests.rs` (nothing else; `dist/`/`logs/` stay ignored — gitignore-compliance rule).

If any step fails: revert BOTH steps (`git restore src/youtube/google_live_api_transport_bind_wire_tests.rs` — only this file is touched by this plan) and report; do NOT improvise other edits.

### Commit (only when the caller confirms — repo style, short imperative)

Suggested message (matches the Phase-03 `2d99ebf` simplification-commit precedent):

    refactor: borrow the decoded request head in bind wire test capture

Stage ONLY `src/youtube/google_live_api_transport_bind_wire_tests.rs`. Never stage `dist/`, `logs/`, `config/config.json`, `credentials/*`, `target/` (gitignore-compliance rule).

## 3. Reviewed and LEAVE-AS-IS (adjudicated, with reasons)

1. **`Box<ureq::Error>` at the single call site** (`google_live_api_transport.rs:81,114–122`): no smaller clippy-clean form exists.
   - Unboxed per §5.1c literally → `clippy::result_large_err` fails the gate's `-D warnings` (this is the recorded 4.2 deviation, forced, not elective).
   - `#[allow(clippy::result_large_err)]` → forbidden by D12.
   - Folding `respond` into the helper (`fn send_bodyless(&self, ...)`) → 3 params incl. `self` (≤2-args violation) AND restructures the D11 byte-stable `respond` chain.
   - `(body, method)` tuple match inside `request()` → drops the helper but adds a 3rd nesting level (max-depth violation) and rewrites the binding §5.1b/1c shape (guardrail: §5.1c arm shape is binding).
   - `Box::into_inner` for the unbox → unstable on the pinned Rust 1.82 toolchain.
   - Verdict: current shape (single box per arm, single `map_err(|boxed_error| *boxed_error)` unbox at the one call site) is the minimal rule-compliant form. No change per the task's own instruction ("if not, leave it").
2. **Per-arm `.map_err(Box::new)` × 2 vs one trailing form**: cosmetics; both shapes deviate from §5.1c by exactly one boxing decision; no complexity gain either way; touches the binding arm-shape area. No change.
3. **`fixture_bind_command` used once + the `pub(super)` fixture re-export chain**: both are the §5 Step 3/4 prescribed shapes (the re-export is what makes the plan's Step 4 two-line `use` spec compile); the alternative (direct import in the checks file) is weight-neutral. No change.
4. **Capture-helper duplication across the three loopback test files**: cap-confirmed plan form — folding them into a shared helper module would edit the 194-line siblings (200-line cap risk) and fight D3; recorded precedent. No change (see §4 C2 for the future-work note).
5. **Assert blocks in the checks file**: verbatim §5.3/§5.4 binding specs (folding into an `assert_served(...)` helper is forbidden for this task). No change.
6. **Module doc (transport lines 10–12), helper doc, `api_request.rs:54` doc line**: exact binding plan text (§5.1a/§5.1c/Step 2), long-line wrap included. No change.
7. **Metadata (`7ccd319`)**: every line matches Step 11/§8 of the plan; counts re-verified (29 siblings, 159 youtube-module tests, 360 gate). Wording is the plan's own; nothing to tighten without creating new drift. No change. (The `NOT done ... 4.3 review/simplify` bullet in context.md stays truthful until the caller's metadata step folds in these results.)

## 4. Deferred candidates → future TODO (report-only; do NOT execute here)

- **C1 — helper-free `request()`**: eliminating `bodyless_post_or_get` + the Box entirely requires either a 3-level-nested match or a `respond`-inside-helper 3-param signature — needs a planner plan-amendment (§5.1c arm shape change + D12 exception ruling). Only worth a phase if the ureq error-size problem recurs elsewhere.
- **C2 — shared loopback-capture module**: unify `record_request`/`headers_end`/`header_value`/read-body logic currently duplicated (by plan choice) across `google_live_api_transport_tests.rs`, the new `*_bind_wire_tests.rs`, and `google_authorization_verifier_tests.rs`. Big refactor (touches files at the 200-line cap) → if ever scheduled, request the architector sub-agent for a complete implementer guide.
- **C3 — legacy 3-param helper**: the Phase-05 sibling `read_body(stream, head, text)` violates the ≤2-params rule; fixing it edits a byte-untouchable-under-D3 file. Hygiene-only; needs its own TODO, never bundled into a fix branch.

## 5. Boundaries honored by this plan

Only `src/youtube/google_live_api_transport_bind_wire_tests.rs` is edited. No test merge, no server inlining into siblings, no mechanism change (`send_bytes(&[])` stays pinned — untouched by §2), no D7 rename, no offline-rule change, no app/streaming/chat/renderer/Cargo/docs touch, no metadata touch, no `allow` additions, no commented code.

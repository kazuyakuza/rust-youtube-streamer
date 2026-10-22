# Global Plan — Fix HTTP 411 (Length Required) at the Broadcast Bind Step

Plan date: 2026-10-22 · Owner: Planner Agent (Critical Workflow)
TODO: `.agent/todos/20261022/20261022-todo-1.md` · Type: FIX (semver PATCH) · Front-end: NO

## Global Pre-Analysis

Incident evidence: `logs/run/20261022-033554.log` — the operator's native Windows `run` reached
"stored authorization verified" (the GET verifier probe succeeded against real Google), then
aborted with `error: YouTube rejected request (HTTP 411, reason 'unspecified')`, exit 1.

Root-cause chain (code-verified):

1. `src/youtube/live_error/classify.rs`: 411 is not in the documented table → catch-all
   `LiveApiError::ApiRejected { status: 411, reason: "unspecified" }` (UNSPECIFIED_REASON when
   the body carries no parseable `reason`); `display.rs:display_rejected` prints the exact
   observed line.
2. The `run` startup performs O1 `liveStreams.insert` and O2 `liveBroadcasts.insert` as POSTs
   WITH JSON bodies (`send_string` → ureq sets `Content-Length`), then O3
   `liveBroadcasts.bind` as the ONLY bodyless POST (`LiveApiRequestSpec::posting` with
   `body: None` → `google_live_api_transport.request` → `pending.call()`).
3. ureq 2.12.1 (locked in `Cargo.lock`; tracked, read only via `git show`): `Request::call()`
   sends a POST with NO body bytes and NO `Content-Length`/`Transfer-Encoding` header. Google's
   front end requires `Content-Length` on POST and answers 411 Length Required.
4. Verifier (GET), O4/O5 lookups and readiness/confirmation polls (GET) never need the header.
   Every documented fact is consistent with O1+O2 succeeding and O3 being rejected inside the
   observed 2.3 s window; no other bodyless-POST exists in the run path (token endpoint POSTs
   carry form bodies).

Consequences recorded for the operator (communication, not code): the failed run leaves the
newly created `ready` broadcast + liveStream pair orphaned in YouTube Studio (documented
resource-fate policy; the app never deletes/transition them). Operator cleanup per
`docs/production-run.md` §7 before the next attempt avoids `userBroadcastsExceedLimit`.

Technical decision encoded (no open judgment for later agents): the bind POST must transmit
ZERO body bytes WITH `Content-Length: 0` — HTTP-correct reading of the bind documentation's
"do not provide a request body". Mechanism selection is a deterministic procedure, decided by
loopback-observed wire output, preference order: (a) `send_bytes(&[])` on the POST agent
request, (b) `send_string("")`, (c) explicit `.set("Content-Length", "0")` retained through
ureq's reserved-header rules — the first mechanism whose loopback capture shows
`Content-Length: 0` with no body bytes wins and is pinned by a test. If ureq's crate source
(available in the Compose container registry cache) is needed to confirm behavior, inspect it
inside the container, never assume.

## Execution Map (steps 2–6; each task gets its own 4.1–4.6 cycle)

- Step 2: Git Feature Branch Setup => implementer — branch `fix/http-411-bind-content-length` from `main`
- Step 3: Version Update => implementer — 0.9.0 → 0.9.1, commit `chore: bump version to 0.9.1`, lockfile refresh verified `--locked`
- Task 1: 4.1b Analysis & Implementation Plan => architector (no front-end → no 4.1a)
- Task 1: 4.2 Implementation => implementer
- Task 1: 4.3 Code Review & Simplification => code-reviewer + code-simplifier concurrently; 4.3-fix => implementer
- Task 1: 4.4 Documentation => docs-specialist
- Task 1: 4.5b Overall Plan Adherence => architector (no front-end → no 4.5a)
- Task 1: 4.6 Task Completion => implementer
- Step 5: TODO File Completion => implementer — `-DONE` rename, tmp hygiene, merge to `main`, push `origin` ONLY

## Per-Task Pre-Analysis (Task 1)

Goal: the O3 bind POST carries `Content-Length: 0` with zero body bytes; wire forms of O1/O2
and all GET paths stay byte-stable; the fix is pinned offline (loopback), never via real Google.

Key code surfaces (junior-executable anchors):

- `src/youtube/google_live_api_transport.rs` — `request()` lines 74–79: the `spec.body` match;
  the `None` arm currently does `pending.call()` for BOTH Get and Post. Split the Post/None arm
  onto the chosen body-send mechanism; keep Get/None as `.call()`.
- `src/youtube/api_request.rs` — `LiveApiRequestSpec` (`getting`/`posting`/`posting_json`);
  the `posting` constructor keeps its name/semantics ("bodyless POST"); only the wire
  transmission changes (no model/path/query edits).
- Loopback tests live in the existing transport test files
  (`google_live_api_transport_tests.rs`, `google_live_api_transport_outcomes_tests.rs`) —
  extend with: (a) bind-form POST arrives with `Content-Length: 0` and empty body;
  (b) anchor test: a POST without `Content-Length` maps to the observed
  `ApiRejected { status: 411, reason: "unspecified" }` Display line
  (`YouTube rejected request (HTTP 411, reason 'unspecified')`); (c) the Get form still
  arrives without body-content-length drift. Maintain the standing rule: no test contacts
  Google.
- Test-engine constraints: ureq is synchronous; loopback servers are the existing
  `TcpListener`-based pattern used by the verifier/transport tests — follow those helpers.
- Failure accounting: any 411 re-appearance with the same signature after the fix must be
  reported to the Planner (do not improvise a second mechanism without re-verification).

Verification duties in later sub-steps: dev-checks gate (`ALL CHECKS PASSED` + log path),
builds via `scripts/build-linux.sh` / `scripts/build-windows.sh` with `ls -l` size cross-checks,
hygiene/secret audit (`git status --porcelain`, no `dist/`, `videos/`, credentials, tokens,
stream keys, RTMPS URLs staged), docs truthfulness (no real-Google verification claims; manual
E2E stays operator-run; the 411 incident documented as an operator-reported log fact).

## Risks and Mitigations

- ureq reserved-header edge (c): if `.set("Content-Length", ...)` is ignored/blocked by ureq,
  mechanism (a)/(b) applies; loopback observation arbitrates — never ship an unverified wire form.
- Lockfile drift: after the version bump run the container `cargo check --locked` path (existing
  workflow) so `Cargo.lock` re-verification stays meaningful.
- Operator expectation: real-Google success is proven ONLY by the recorded operator manual E2E
  (`docs/production-run.md` §8), not by this fix's tests.

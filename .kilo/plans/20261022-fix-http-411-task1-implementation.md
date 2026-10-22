# Implementation Plan — Task 1: Bodyless-POST `Content-Length: 0` Fix (transport + loopback regression tests)

Plan date: 2026-10-22 · Author: Architector Agent (Critical Workflow step 4.1b)
TODO: `.agent/todos/20261022/20261022-todo-1.md` — Task 1 (§ Tasks) · Type: FIX (semver PATCH, already bumped)
Global plan (binding, approved — BINDING DECISIONS THERE; this file does not re-litigate them): `.kilo/plans/20261022-fix-http-411-bind-content-length.md`
Front-end: NO (no 4.1a spec exists; no front-end verification in 4.5a)

Workflow positions covered by THIS plan: step 4.2 (implementer, code+tests+gate+builds+audit) and the handoff notes for 4.3/4.4/4.5b. Steps 2 (branch), 3 (version bump), 4.6 (DONE marks), 5 (merge/push) are NOT in this plan's scope.

---

## 0. Task 1 requirements (verbatim contract from the TODO)

1. Route the bodyless-POST form (O3 bind, the `posting` spec constructor) through a body-send path that transmits exactly zero body bytes WITH a `Content-Length: 0` header, verified against ureq 2.12.1 behavior; the chosen mechanism must be pinned by a loopback test, not assumed.
2. Keep `LiveApiMethod::Get` requests on the current bodyless `.call()` path; keep O1/O2 JSON-POST behavior and all five path builders, query encodings and request bodies byte-stable.
3. Loopback unit tests: (a) the bind-form POST arrives at the local test server with header `Content-Length: 0` and an empty body; (b) a POST arriving without `Content-Length` classifies as the observed 411 rejection shape, anchoring the incident; (c) existing pinned request-shape tests updated only where the wire form legitimately changed.
4. Re-run the standard dev-checks gate on the final tree (`ALL CHECKS PASSED`, recorded log path); rebuild both release artifacts with `ls -l` sizes; run the repo hygiene/secret audit.
5. Docs update where the change touches them (production-run 411 troubleshooting/cleanup note; structure map; context file; README drift check).

Acceptance criteria (TODO § Acceptance Criteria): the five `[x]` items — bind POST `Content-Length: 0` + zero body bytes; loopback regression tests; gate green with log path + test count; both artifacts rebuilt; hygiene/secret audit clean; docs updated. (Checkbox flips themselves are step 4.6 — NOT this plan.)

---

## 1. Research Record (evidence cited; binding)

### 1.1 Incident evidence (why O3 bind is the failing stage)

- `logs/run/20261022-033554.log` lines 1–6: `run` printed `stored authorization verified` (14:35:54.45), then `error: YouTube rejected request (HTTP 411, reason 'unspecified')`, then `mode finished command="run" exit_code=1` (03:35:56.77). The ~2.3 s window is consistent with O1+O2 (JSON POSTs) succeeding and the O3 bind being rejected.
- `src/youtube/google_live_api_transport.rs` lines 74–79: the `spec.body` match arm `None => pending.call()` serves BOTH `LiveApiMethod::Get` and `LiveApiMethod::Post`; `pending` is a `ureq::Request` built on a POST for the bind (line 71, `LiveApiMethod::Post => self.agent.post(&url)`).

### 1.2 ureq 2.12.1 documented behavior (root cause + mechanism candidates) — CONCLUSIVE at the docs level

Lockfile confirmation (read-only, never modified in this task): `git show HEAD:Cargo.lock` → `name = "ureq"`, `version = "2.12.1"`, `source = "registry+https://github.com/rust-lang/crates.io-index"`. (`Cargo.lock` is tracked but blocked for direct file reads by the repo's tooling rules — always read it via `git show`.) The working tree is at commit `9afb63d chore: bump version to 0.9.1`, branch `fix/http-411-bind-content-length`, clean tree (observed 2026-10-22).

Authoritative pinned-version citation — https://docs.rs/ureq/2.12.1/ureq/struct.Request.html (fetched 2026-10-22):

- `Request::call(self)`: *"Sends the request with no body and blocks the caller until done. Use this with GET, HEAD, OPTIONS or TRACE. It sends neither Content-Length nor Transfer-Encoding."* → CONFIRMS the root cause: the bind POST via `call()` sends no `Content-Length` and no body bytes; Google's front end requires `Content-Length` on POST and answers 411.
- `Request::send_bytes(self, data: &[u8])`: *"Send data as bytes. The Content-Length header is implicitly set to the length of the serialized value."* → mechanism (a) `send_bytes(&[])` is documented to transmit `Content-Length: 0` with zero body bytes.
- `Request::send_string(self, data: &str)`: *"The Content-Length header is implicitly set to the length of the serialized value. Defaults to utf-8."* → mechanism (b) `send_string("")` is documented the same way.
- `Request::set(...)`: sets a header field; the ureq README ("Overriding") documents user-set `Content-Length`/`Transfer-Encoding` being respected "before sending the body" — i.e. on the SEND paths. Whether a user-set `Content-Length` survives `call()` (a no-body path that documents it sends neither header) is NOT documented → mechanism (c) is the last resort and must be arbitrated by loopback observation only.
- `send_bytes`/`send_string` do NOT set `Content-Type` (only `send_json`/`send_form` do) → the existing loopback pin `bind_sends_bodyless_post_with_both_query_ids` (`content_type == None`) stays true after the fix; no existing request-shape test breaks.

Context7 corroborates (`/algesten/ureq`, fetched 2026-10-22): "call … Sends neither Content-Length nor Transfer-Encoding"; "bodies with a known size are sent with Content-Length" — but that mirror reflects the ureq v3 `main` branch; the docs.rs 2.12.1 page above is the version-authoritative citation and is quoted throughout.

**Research verdict (mechanism):** choose mechanism **(a) `send_bytes(&[])`** — documented to emit `Content-Length: 0` with zero body bytes, no content type, no new dependency. The loopback tests created in this task are the binding arbiter: they must OBSERVE `Content-Length: 0` on the wire; if they do not, the encoded fallback procedure (§4) applies mechanically. The container-source inspection fallback (§4 item 4) is recorded but expected to be unnecessary.

### 1.3 Single-caller evidence for `posting` (scope confinement)

`grep -n "posting"` over `src/` (2026-10-22): exactly two definitions (`api_request.rs:55` `posting`, `api_request.rs:64` `posting_json`) and ONE call site: `google_live_api_transport.rs:129` inside `fn bind(...)`. No other request uses the bodyless-POST form; the `None`-Post arm change therefore cannot affect any other request. (Token-endpoint POSTs carry form/JSON bodies via the separate Phase-04 transport and are out of this scope; verifier/O4/O5/poll paths are GETs.)

### 1.4 Existing loopback test patterns (what the new tests must reuse)

- Shared fixtures served by the sibling `src/youtube/google_live_api_transport_tests.rs`: `pub(super)` constants `ACCESS_TOKEN_FIXTURE`, `STREAM_ID_FIXTURE`, `BROADCAST_ID_FIXTURE`, `STREAM_RESOURCE_BODY`, `BOUND_BROADCAST_BODY` (lines 15–21) are importable by sibling test modules (precedent: `google_live_api_transport_outcomes_tests.rs` line 16, `use super::google_live_api_transport_tests::*;`).
- The established capture style (`RecordingJsonServer`, lines 32–125): `TcpListener::bind("127.0.0.1:0")` → one accept per `start` → request captured by reading bytes until the `\r\n\r\n` head terminator (`headers_end`), parsing `request_line` / `authorization` / `content_type` via `header_value`, then reading the body up to the request's own `Content-Length` (`read_body`); the canned reply always carries `Content-Length: <len>` + `Connection: close`.
- The `RecordedRequest` struct in that sibling does NOT capture `Content-Length`. Decision D3 (§3): the existing 194-line siblings are NOT modified (both are at the 200-line cap); the new coverage gets its own sibling file that duplicates the small capture helpers with the one added `content_length` capture — the per-file loopback-server duplication precedent is `google_authorization_verifier_tests.rs` (its own `FakeApiServer` + `record_request`, lines 18–83).
- Failure-extraction precedent without `expect_err` (resources are Debug-free): `google_live_api_transport_outcomes_tests.rs` lines 29–34 (`api_failure`).

### 1.5 Existing 411 classification/Display coverage (what must NOT be duplicated)

- `src/youtube/live_error/classify.rs` lines 39–42: 411 falls to the catch-all `_` arm → `ApiRejected { status, reason: UNSPECIFIED_REASON }` when the body carries no parseable `reason`.
- `src/youtube/live_error_tests.rs::backend_and_unknown_map_status_first` (lines 104–135) pins the catch-all mapping generically (403 unknown reason + `classify_api_error(503, None)`). `src/youtube/live_error_display_tests.rs` (lines 28–31, 61) pins the `ApiRejected` Display FORMAT with a 403 instance, sharing `display_rejected` with 411.
- Verdict: the classifier/Display contracts are already covered GENERICALLY. No new classifier test is added (a duplicate would violate no-duplication hygiene). What is NOT covered anywhere: a REAL 411 reply flowing through the concrete transport's ONE `respond` path and rendering as the exact incident line. The new test (b) in §5 covers exactly that gap — it is a transport-level test, not a classifier-display duplicate. The 4.3 reviewers should treat `live_error_tests.rs`/`live_error_display_tests.rs` as "note: 411 already generally covered; the new bind test is the transport-level anchor".

---

## 2. High-Level Approach

1. **Git handling (read-only):** confirm branch `fix/http-411-bind-content-length` at `9afb63d` (`chore: bump version to 0.9.1`) with a clean tree; work in place; two commits at the end (code+tests; agent-metadata refresh). No branch creation, no push, no version bump, no lockfile change (all restricted to steps 2/3/5 which are already done or out of scope).
2. **Code change (one file, tiny):** split the `None` arm of `GoogleLiveApiTransport::request` so the GET form keeps `pending.call()` and the bodyless POST routes through `pending.send_bytes(&[])` (mechanism (a)); extract the arm into a small private free function to keep the body under rule limits; update the module doc line of `google_live_api_transport.rs` and the `posting` constructor doc of `api_request.rs`.
3. **New loopback tests (two sibling test files, pre-split for the 200-line cap):** an enforcing single-accept loopback server that decides its reply from the request's own wire facts (Google-like behavior: a bodyless POST WITH `Content-Length: 0` → 200 accepted; any POST WITHOUT `Content-Length` → 411 rejected; a GET → accepted regardless of the lengthless form), plus three tests: (a) bind zero-length wire pin, (b) pre-fix lengthless POST classification anchor (the incident shape), (c) GET form stays lengthless.
4. **Fallback decision procedure (encoded, deterministic):** mechanism order send_bytes(&[]) → send_string("") → set("Content-Length","0"), each arbitrated strictly by the new loopback wire tests; if all three fail the wire observation, STOP and report to the Planner (container-source inspection commands provided in §4 item 4 — never ship an unverified wire form).
5. **Console cmds:** targeted `cargo test` first, then the standard gate (`scripts/dev-checks.sh` — fmt/check/test/clippy, `ALL CHECKS PASSED`, recorded log path + observed test count), then both release build scripts with `ls -l` size cross-checks, then the hygiene/secret audit.
6. **Docs (4.4 handoff + tracked metadata):** `docs/production-run.md` gets the 411 troubleshooting/cleanup note; `docs/` targets and the structure-map/context/tech refreshes are listed with exact anchors; README only drift-checked. No new doc files. No doc claims real-Google verification.
7. **Review handoff (4.3):** constraints for code-reviewer/code-simplifier (byte-stability checklist, redaction guarantees, cap checks).

---

## 3. Encoded Decisions (binding — no implementer judgment allowed)

These mirror the global plan verbatim where required.

- **D1 (mechanism, chosen):** bodyless POST = `pending.send_bytes(&[])`. Preference order (global plan, verbatim): (a) `send_bytes(&[])` on the POST agent request, (b) `send_string("")`, (c) explicit `.set("Content-Length", "0")` retained through ureq's reserved-header rules — the first mechanism whose loopback capture shows `Content-Length: 0` with no body bytes wins and is pinned by a test. If ureq's crate source is needed to confirm behavior, inspect it inside the container (§4 item 4), never assume.
- **D2 (GET stability):** `LiveApiMethod::Get` with `spec.body == None` keeps `pending.call()` — byte-unchanged wire form, no `Content-Length` introduced.
- **D3 (test file layout):** two NEW test sibling files; the two existing transport test files (both 194 lines, at the 200-line cap) stay byte-unchanged; the existing pinned request-shape tests are updated ONLY if the wire form legitimately changed (D6 says it does not for O1/O2; the bind test's assertions remain true).
- **D4 (enforcing-server semantics):** the single-accept loopback server decides its reply from the request itself: POST without `Content-Length` header → `HTTP/1.1 411 Length Required` + a plain-text body (fails `extract_reason` → `unspecified`, exactly the incident shape); anything else → `HTTP/1.1 200 OK` + the caller-supplied canned success body. The pre-fix lengthless POST wire form is reproduced IN THE TEST (`ureq::AgentBuilder::new().redirects(0).build().post(url).set("Authorization", ...).call()`) — never in production code — to prove the enforcement+classification outcome. Rationale: after D1 no production code path performs a lengthless POST, so the only honest wire-negative anchor is the in-test reproduction of the incident wire form.
- **D5 (classifier reuse):** the raw ureq reply of the in-test reproduction is classified with the same function `respond` uses for `Err(ureq::Error::Status(...))` — `crate::youtube::api_request::response_reason_error(status, response)` (pub(super) in `api_request.rs`, visible to `youtube` descendants). Do not replicate the matcher logic inline beyond this one-arm helper.
- **D6 (scope confinement):** only these files change: `src/youtube/google_live_api_transport.rs` (code + module doc + test-mod registrations), `src/youtube/api_request.rs` (doc comment line only), plus the two new test files. `src/app/**`, `src/streaming/**`, `Cargo.toml`, `Cargo.lock` stay byte-untouched; `auth`/`demo` exit codes and behavior unchanged by construction.
- **D7 (test count + names, binding):** exactly three new tests with these names —
  1. `bind_post_zero_content_length_is_accepted_by_enforcing_server`
  2. `pre_fix_lengthless_bind_post_classifies_as_observed_411_shape`
  3. `get_lookup_stays_lengthless_with_lengthless_body`
  Expected total: 357 (v0.9.0 gate) + 3 = **360** (the OBSERVED gate-log count governs).
- **D8 (docs targets, no new files):** `docs/production-run.md` (411 bullet in the "Common ingestion/API failures" list + Studio cleanup pointer), `.agent/project-structure.md` (transport entry + test-file inventory), `.agent/project-info/tech.md` (one sentence on the bodyless-POST zero-length mechanism), `.agent/project-info/context.md` (truthful refresh), `README.md` (drift check only — expected NO edit).
- **D9 (commits, repo style short imperative):**
  1. `fix: send zero-length Content-Length on bodyless Live API post`
  2. `docs: refresh structure and context for the HTTP 411 fix`
- **D10 (offline rule):** every new test talks only to a `127.0.0.1` loopback listener; no test ever contacts Google; the real-Google confirmation remains the operator's manual E2E (`docs/production-run.md` §8).
- **D11 (redaction/exit-code/fate stability):** the `respond` classification path, error redaction (reason-only surfacing), the run exit-code contract, shutdown order and the resource-fate policy stay byte-stable; this fix touches only the request-transmission arm.
- **D12 (functional-edit style):** no new `#[allow(...)]` attributes; no commented-out code; self-documenting names; single-section boolean conditions at call sites (multi-part wire rule lives inside a named predicate method); ≤2 params per function; ≤50-line bodies; ≤200-line files.

---

## 4. The Mechanism Fallback Procedure (binding; global plan encoding)

Run ONLY while the loopback wire test `bind_post_zero_content_length_is_accepted_by_enforcing_server` does not pass with the current mechanism. Each step's arbitration is the targeted test run (§5 Step 5 item 2; command inventory §6 row 2).

1. **Mechanism (a)** `pending.send_bytes(&[])` — the default already written by Step 1. If the wire capture shows `Content-Length: 0` and an empty body → mechanism settled; record in the summary.
2. **Mechanism (b)** — one substitution at the SAME site (the `LiveApiMethod::Post` arm inside `bodyless_post_or_get`, §5 Step 1c):

   ```rust
   LiveApiMethod::Post => pending.send_string(""),
   ```

   Re-run the targeted test. Same success bar.
3. **Mechanism (c)** — next substitution at the same site:

   ```rust
   LiveApiMethod::Post => pending.set("Content-Length", "0").call(),
   ```

   Re-run. This variant MUST be confirmed by the loopback capture (docs do not guarantee the header survives `call()`), and its doc comment must state the explicit-header mechanism.
4. **If all three fail the wire observation:** STOP. Do not invent mechanism 4. Capture the exact wire output, run the container-source inspection (the commands below), and return the observations to the caller (Planner). The inspect-step commands:

   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fetch --locked` — ensure the registry source is present in the fresh container.
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "find /usr/local/cargo/registry/src -maxdepth 2 -type d -name 'ureq-2.12.1'"`
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "grep -n 'content-length' /usr/local/cargo/registry/src/*/ureq-2.12.1/src/unit.rs"` (and `stream.rs` if needed; keep greps narrow to avoid output limits)

   All container commands go through the Alpine-VM MCP: call `vm_status` FIRST, then `alpine-vm_vm_run_command` with the `docker compose ...` string (allowed prefix `docker`). Exit code is authoritative; captured output may omit cargo's `Finished` lines.
5. **Failure accounting (global plan):** if a 411 re-appears with the identical signature after the fix ships, report to the Planner — do NOT improvise a second mechanism without re-verification.

Mechanism-settled outputs to record in the task summary: chosen mechanism letter, the exact loopback-observed wire evidence (test name + captured header), and the standard gate log path.

---

## 5. Implementation Steps (sequenced, junior-executable)

### Step 0 — Read-only state verification (no changes)

Commands (host, `bash` tool, single commands, workdir `C:\repo\rust-youtube-streamer`):

- `git status --porcelain` → expect EMPTY output (clean tree).
- `git branch --show-current` → expect `fix/http-411-bind-content-length`.
- `git log --oneline -3` → expect `9afb63d chore: bump version to 0.9.1` at HEAD.

If any expectation fails: STOP and return the observed state to the caller (no branch/switch/status repair is in this step's scope).

### Step 1 — Transport code change (`src/youtube/google_live_api_transport.rs`)

**1a. Module doc line (top comment block, lines 1–19):** the existing limits sentence ends at line 9 — exact current text:

- Line 8: `//! verifier's private extraction. Limits (D-T2-16): no request-level`
- Line 9: `//! timeout (verifier parity); no transition/delete calls anywhere.`

Keep lines 1–9 byte-unchanged; INSERT this one doc-comment sentence (three rustfmt-wrapped `//!` lines) immediately after line 9:

```
//! The bodyless POST (O3 bind) transmits zero body bytes WITH Content-Length: 0 via
//! send_bytes(&[]) (the HTTP 411 fix, loopback-pinned); GET forms stay on the no-body
//! call() path.
```

*Note:* the sentence is split across doc-comment lines (rustfmt-wrapped shape); lines 10+ shift down by 3; no other doc text moves.

**1b. The `request()` match arm — exact before/after shape (the whole `request` fn is lines 66–81; the `spec.body` match is lines 74–79):**

BEFORE:

```rust
    let reply = match spec.body {
        Some(body) => pending
            .set("Content-Type", "application/json")
            .send_string(body.to_string().as_str()),
        None => pending.call(),
    };
    self.respond(reply)
```

AFTER:

```rust
    let reply = match spec.body {
        Some(body) => pending
            .set("Content-Type", "application/json")
            .send_string(body.to_string().as_str()),
        None => bodyless_post_or_get(spec.method, pending),
    };
    self.respond(reply)
```

`spec.method` is `Copy` (the enum derives `Clone, Copy`) so passing it by value hazards nothing.

**1c. New private free function (place it directly after the `request`/`respond` methods, alongside `is_success_status` — same module, file-private by default per the prefer-private rule):**

```rust
/// The no-body reply path: GET keeps the bare no-body `call()`; POST
/// sends exactly zero body bytes WITH `Content-Length: 0` (mechanism (a);
/// pinned by the bind wire tests), so a Google-like enforcing server
/// accepts the bind form instead of answering 411 Length Required.
fn bodyless_post_or_get(
    method: LiveApiMethod,
    pending: ureq::Request,
) -> Result<ureq::Response, ureq::Error> {
    match method {
        LiveApiMethod::Get => pending.call(),
        LiveApiMethod::Post => pending.send_bytes(&[]),
    }
}
```

Rules check: 2 params; 5-line body; single-section conditions; no comments inside logic; `ureq::Request`/`ureq::Response`/`ureq::Error` spelled as full paths (the file never imports ureq items).

**1d. Register the two new test modules at the end of the file (after the two existing `#[cfg(test)] #[path = ...] mod ...` blocks):**

```rust

#[cfg(test)]
#[path = "google_live_api_transport_bind_wire_tests.rs"]
mod google_live_api_transport_bind_wire_tests;

#[cfg(test)]
#[path = "google_live_api_transport_bind_checks_tests.rs"]
mod google_live_api_transport_bind_checks_tests;
```

Line-count projection for the file: 161 + ~3 doc + ~0 net arm + ~12 fn + ~11 mod block ≈ **187** — under the 200-line cap. Verify after writing (`Let the gate's fmt check catch nothing; count lines if unsure`).

### Step 2 — `posting` doc line (`src/youtube/api_request.rs`, line 54–55)

Replace the constructor doc comment:

- BEFORE: `/// Bodyless POST (O3 bind: "Do not provide a request body").`
- AFTER: `/// Bodyless POST (O3 bind: zero body bytes transmitted WITH Content-Length: 0; the documented "do not provide a request body" semantics).`

No code change in this file; the three constructors keep their names and semantics (`LiveApiRequestSpec::posting` still means "bodyless POST" — only the transmission form changed).

### Step 3 — New test file A: the enforcing loopback server (`src/youtube/google_live_api_transport_bind_wire_tests.rs`)

Purpose: request-fact capture AND Google-like enforcement in one shape, following the sibling's `RecordingJsonServer` style (one accept per `start`, canned reply bytes, channel + join). Write the file with this structure (identifiers are binding; local micro-details belong to the implementer's 50% latitude):

Module doc (self-documenting, ~8 lines): what it pins, loopback-only statement, plan/TODO pointers (`Plan: .kilo/plans/20261022-fix-http-411-task1-implementation.md`; `TODO .agent/todos/20261022/20261022-todo-1.md Task 1`).

Uses:

```rust
use std::io::{Read, Write};
use std::sync::mpsc;

use super::google_live_api_transport_tests::{
    ACCESS_TOKEN_FIXTURE, BOUND_BROADCAST_BODY, BROADCAST_ID_FIXTURE, STREAM_ID_FIXTURE,
};
use super::*;
```

(All five public-in-super fixtures exist; `super::*` brings `GoogleLiveApiTransport`, `AccessToken`, `LiveApiError`, `BindCommand`, `LiveApiMethod` and the transport's own imports, exactly like the outcomes sibling.)

Constants:

```rust
const ENFORCED_STATUS_LINE: &str = "HTTP/1.1 411 Length Required";
const ACCEPTED_STATUS_LINE: &str = "HTTP/1.1 200 OK";
const ENFORCED_BODY: &str = "Length Required";
```

(`ENFORCED_BODY` is plain text on purpose: `extract_reason` cannot parse it → `unspecified` — the exact incident shape.)

Captured request facts (binding fields):

```rust
pub(super) struct ServedRequest {
    pub(super) request_line: String,
    pub(super) content_length: Option<String>,
    pub(super) body: String,
}
```

Server struct + impl (single accept, decide-then-reply, record facts):

```rust
pub(super) struct EnforcingBindServer {
    pub(super) port: u16,
    served: mpsc::Receiver<ServedRequest>,
    handle: std::thread::JoinHandle<()>,
}
```

- `pub(super) fn start(accept_body: &str) -> Self` (2 params incl. self): `TcpListener::bind("127.0.0.1:0")` → capture port → channel → one worker thread: `accept` → `record_request(&mut stream)` → `sender.send(served_request)` (record BEFORE replying — the sibling's order; the mpsc buffer absorbs the race) → `stream.write_all(reply_for(&served_request, accept_body).as_bytes())`.
- `pub(super) fn transport(&self) -> GoogleLiveApiTransport` — mirror the sibling's builder:
  `GoogleLiveApiTransport::with_base_url(format!("http://127.0.0.1:{}/youtube/v3/", self.port), AccessToken::new(ACCESS_TOKEN_FIXTURE))`
- `pub(super) fn served_request(&self) -> ServedRequest` — `self.served.recv().expect("the request fact was recorded")`.
- `pub(super) fn finish(self)` — `self.handle.join().expect("the enforcing server completes")`.

Decision + reply (single-section call site; named predicate holds the multi-part rule):

```rust
fn reply_for(served_request: &ServedRequest, accept_body: &str) -> String {
    if is_enforced_request(served_request) {
        canned_reply(ENFORCED_STATUS_LINE, ENFORCED_BODY)
    } else {
        canned_reply(ACCEPTED_STATUS_LINE, accept_body)
    }
}

/// The Google-like enforcing rule for this probe: a POST arriving
/// WITHOUT a Content-Length header is 411-class; every other form —
/// the fixed zero-length bind POST or any GET — is accepted.
fn is_enforced_request(served_request: &ServedRequest) -> bool {
    served_request.request_line.starts_with("POST") && served_request.content_length.is_none()
}

fn canned_reply(status_line: &str, body: &str) -> String {
    format!(
        "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}
```

Capture helpers (duplication of the sibling's private fns is the recorded precedent — the verifier file did the same; adapt with ONE added capture line):

- `fn record_request(stream: &mut std::net::TcpStream) -> ServedRequest` — read chunks into a `Vec<u8>` until `headers_end` (windows-4 `\r\n\r\n` scan) or EOF; split head/body at the terminator; parse `content_length` from the head via a `header_value` clone; read the body tail up to the parsed length (absent length → read the tail already buffered).
- `fn headers_end(bytes: &[u8]) -> Option<usize>` — `bytes.windows(4).position(|w| w == b"\r\n\r\n")` + 4.
- `fn header_value(head: &str, name: &str) -> Option<String>` — case-insensitive `lines()` split-on-`:` scan (clone of the sibling's fn).
- Zone the body read in one helper `fn read_body_tail(...)` mirroring the sibling's `read_body` (chunk read until the parsed content-length or EOF).
- `ServedRequest.body` is the decoded tail as `String::from_utf8_lossy`; `content_length` is `Option<String>` (the verbatim header value, e.g. `Some("0")`).

Pre-fix wire helper + classifier (used only by test (b), defined in this file so the helper stays under the cap):

```rust
/// Reproduces the INCIDENT wire form in the test only: a POST whose
/// transmission is the ureq no-body `call()` — the exact shape that
/// produced `HTTP 411` at Google before the fix. Never production code.
pub(super) fn pre_fix_lengthless_call(url: &str) -> Result<ureq::Response, ureq::Error> {
    ureq::AgentBuilder::new()
        .redirects(0)
        .build()
        .post(url)
        .set("Authorization", &format!("Bearer {ACCESS_TOKEN_FIXTURE}"))
        .call()
}

/// Classifies a raw ureq reply through the SAME arm the transport's ONE
/// `respond` applies to non-2xx statuses (D-T2-5), keeping the pinned
/// classification honest without widening production visibility.
pub(super) fn classify_through_status_arm(
    reply: Result<ureq::Response, ureq::Error>,
) -> LiveApiError {
    match reply {
        Err(ureq::Error::Status(status, response)) => {
            response_reason_error(status, response)
        }
        _ => panic!("expected the enforcing rejection to arrive as a status error"),
    }
}
```

with `use crate::youtube::api_request::response_reason_error;` at the top of this file (pub(super) in `api_request.rs` = visible in the `youtube` tree; precedent: outcomes tests import `encode_query_value` the same way).

Line-count projection: ≈ 150–175. Under the cap. Verify after writing.

### Step 4 — New test file B: the three wire tests (`src/youtube/google_live_api_transport_bind_checks_tests.rs`)

Uses:

```rust
use super::google_live_api_transport_bind_wire_tests::*;
use super::*;
```

Shared bind command helper (2 params incl. self context-free — a free fn):

```rust
fn fixture_bind_command() -> BindCommand {
    BindCommand {
        broadcast_id: BROADCAST_ID_FIXTURE.to_string(),
        stream_id: STREAM_ID_FIXTURE.to_string(),
    }
}
```

**Test (a) — `bind_post_zero_content_length_is_accepted_by_enforcing_server`** (the D1 pin; TODO bullet (a)):

```rust
#[test]
fn bind_post_zero_content_length_is_accepted_by_enforcing_server() {
    let server = EnforcingBindServer::start(BOUND_BROADCAST_BODY);
    let broadcast = server
        .transport()
        .bind(&fixture_bind_command())
        .expect("the zero-length bind POST is accepted by the enforcing server");
    assert_eq!(broadcast.id.as_deref(), Some(BROADCAST_ID_FIXTURE));
    let served = server.served_request();
    assert!(
        served
            .request_line
            .starts_with("POST /youtube/v3/liveBroadcasts/bind?")
    );
    assert_eq!(served.content_length.as_deref(), Some("0"));
    assert!(served.body.is_empty(), "bind sends no body bytes");
    server.finish();
}
```

Assertable contract: the transport sends the bind POST with `Content-Length` exactly `0` and an empty request body; a Google-like enforcing server accepts it (200 + parses the bound broadcast). (`LiveBroadcast.id` spelling is `id` in the live model — matches `BOUND_BROADCAST_BODY`; verify once against `src/youtube/live_models/broadcast_resource.rs` before finalizing.)

**Test (b) — `pre_fix_lengthless_bind_post_classifies_as_observed_411_shape`** (the incident anchor; TODO bullet (b)):

```rust
#[test]
fn pre_fix_lengthless_bind_post_classifies_as_observed_411_shape() {
    let server = EnforcingBindServer::start(BOUND_BROADCAST_BODY);
    let url = format!(
        "http://127.0.0.1:{}/youtube/v3/liveBroadcasts/bind?id={}&streamId={}",
        server.port,
        BROADCAST_ID_FIXTURE,
        STREAM_ID_FIXTURE
    );
    let error = classify_through_status_arm(pre_fix_lengthless_call(&url));
    let LiveApiError::ApiRejected { status, reason } = &error else {
        panic!("expected ApiRejected, got {error}");
    };
    assert_eq!(*status, 411);
    assert_eq!(reason, "unspecified");
    assert_eq!(
        error.to_string(),
        "YouTube rejected request (HTTP 411, reason 'unspecified')"
    );
    let served = server.served_request();
    assert!(
        served.content_length.is_none(),
        "the reproduced pre-fix wire form carries no Content-Length"
    );
    server.finish();
}
```

Assertable contract: a lengthless POST arriving at the enforcing server is rejected 411; the reply classifies through the transport's status arm into `ApiRejected { status: 411, reason: "unspecified" }` whose Display is byte-identical to the incident line (log line 5). `pre_fix_lengthless_call` and `classify_through_status_arm` are free functions imported by the glob `use super::google_live_api_transport_bind_wire_tests::*;`. Doc-note in the test that the in-test reproduction is the only honest wire-negative pairing under D4.

**Test (c) — `get_lookup_stays_lengthless_with_lengthless_body`** (D2 pin; global-plan bullet (c)):

```rust
#[test]
fn get_lookup_stays_lengthless_with_lengthless_body() {
    let server = EnforcingBindServer::start(r#"{"items":[]}"#);
    let lookup = server
        .transport()
        .lookup_stream(STREAM_ID_FIXTURE)
        .expect("a GET lookup needs no Content-Length");
    assert!(lookup.is_none());
    let served = server.served_request();
    assert!(
        served
            .request_line
            .starts_with("GET /youtube/v3/liveStreams?")
    );
    assert!(served.content_length.is_none(), "GET stays on the no-body call() form");
    assert!(served.body.is_empty());
    server.finish();
}
```

Assertable contract: the GET form (verifier/O4/O5 class) still transmits NO `Content-Length` header (no drift) and a Google-like server accepts it (proving the enforcement rule is POST-specific, mirroring Google's real distinction).

Line-count projection: ≈ 110–125. Under the cap.

### Step 5 — Local test loop (fmt + clippy + targeted tests, before the full gate)

All cargo commands run inside the Compose `rust` service via the Alpine-VM MCP (call `vm_status` FIRST, then `alpine-vm_vm_run_command`; exit code authoritative):

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` — if it flags formatting, apply `cargo fmt` ONCE (repo practice: rustfmt normalization), then re-check.
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked google_live_api_transport_bind` — runs the two new modules' tests. Expect the 3 new tests pass. If test (a) fails at the WIRE observations (missing `Content-Length`, body bytes present) → execute §4 fallback (mechanism (b), then (c)) and re-run until pass or STOP-rule triggers.
   - If a failure is in the TEST machinery (panics/parse errors/unwraps) — fix the TEST, never the mechanism; the mechanism verdict comes only from the wire assertions in (a)/(b)/(c).
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` — repo-fresh code must produce zero warnings.

### Step 6 — Byte-stability spot-checks (before the gate; cheap and mandatory)

- `git diff --stat` shows EXACTLY: `src/youtube/google_live_api_transport.rs`, `src/youtube/api_request.rs`, and the two new test files. Any other touched file → STOP, report.
- O1/O2 body stability (both must pass UNCHANGED, proving the `Some(body)` arm and the locked request bodies keep their exact wire forms): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked stream_insert_posts_documented_path_with_bearer_and_json`, then `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked broadcast_insert_posts_documented_path_with_locked_body`. Note: a cargo-test filter matches test-name substrings — the full-name filters above select exactly those two test fn names.
- Existing bind wire test must pass unchanged: `cargo test --locked bind_sends_bodyless_post_with_both_query_ids`.

### Step 7 — Standard dev-checks gate (the recorded gate)

`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`

Record and keep: exit code, the four PASS lines (`fmt-check`, `check`, `test`, `clippy`), `ALL CHECKS PASSED`, the UTC log filename under `logs/checks/` (gitignored), AND the OBSERVED test count from the gate log's `test result:` line (expect 360 per D7; observed governs).

Retry-loop: gate failure → fix per the failure naming (fmt → `cargo fmt` once; clippy → code hygiene per D12; test → re-verify the failing test) and re-run. Maximum 3 gate cycles before returning to the caller.

### Step 8 — Release artifact rebuilds (both scripts, both sizes)

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` — exit 0; success message names `dist/rust-youtube-streamer-service` and its byte size.
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh` — exit 0; success message names `dist/windows/rust-youtube-streamer-service.exe` and its byte size.
3. `ls -l` cross-check (container view of the shared mount):
   `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist`
   Confirm both printed sizes match the scripts' messages. Record both byte counts (they WILL differ from v0.9.0's 5197208/8613503 — sizes change with code; never hardcode sizes in docs).

### Step 9 — Hygiene/secret audit (gitignore-compliance rule)

Host, `bash` tool, single commands:

- `git status --porcelain` — exactly the four expected paths from Steps 1–4; no `dist/`, no `videos/`, no `config/config.json`, no credential files.
- Ignore-rule check: `git check-ignore -v dist/rust-youtube-streamer-service dist/windows/rust-youtube-streamer-service.exe` — both map to `.gitignore`'s `dist/` rule; and `git ls-files dist/ videos/ config/config.json credentials/` — expect EMPTY (the `credentials/.gitkeep` tracking is pre-existing and unchanged; nothing new may appear).
- Secret-pattern audit over `src/` (single commands, each followed by reading the count): `git grep -n -F "ya29" -- src/` (expect 0 hits), `git grep -n -F "AIza" -- src/` (expect 0 hits), `git grep -n -F "key=" -- src/` (expect exactly 1 hit — the pre-existing doc-comment at `google_live_api_transport.rs` line 3), `git grep -n -F "allow(dead_code)" -- src/` (expect exactly the pre-existing 8 attribute lines; the new code adds none).
- Redaction guarantee: read the final `respond` (unchanged) and confirm the new arm touches ONLY transmission (no formatting, logging, or error-embedding of any body/URL/token material). This is a review gate for 4.3 as well.

### Step 10 — Commits (repo style short imperative; gitignore rule: review `git status` before staging; stage only intended files)

Commit 1 (code + tests; the full logical change):

```
git add src/youtube/google_live_api_transport.rs src/youtube/api_request.rs src/youtube/google_live_api_transport_bind_wire_tests.rs src/youtube/google_live_api_transport_bind_checks_tests.rs
git commit -m "fix: send zero-length Content-Length on bodyless Live API post"
```

Commit 2 (tracked agent metadata refresh; AFTER Step 11's gate/build records exist so the text is truthful):

```
git add .agent/project-structure.md .agent/project-info/tech.md .agent/project-info/context.md
git commit -m "docs: refresh structure and context for the HTTP 411 fix"
```

(README edit is expected NONE — if the drift check finds one, include it here; never a separate README commit for this task.)

### Step 11 — Tracked metadata refreshes (encoded; written BEFORE Commit 2)

- `.agent/project-structure.md`:
  - Update the `google_live_api_transport.rs` bullet (line 69): add the bodyless-POST zero-length fact — e.g. append "the bodyless POST (O3 bind) transmits zero body bytes WITH Content-Length: 0 via send_bytes(&[]) (HTTP 411 fix; loopback-pinned)".
  - Update the test-file bullet (line 76): "27 siblings → 29" with the two new files and "156 → 159 tests in the youtube module" (MODULE arithmetic: the 156 total counts youtube tests at the last audit; the 3 new tests raise it — recompute by counting the test fns per file, not by assumption).
- `.agent/project-info/tech.md`: extend the Phase-05 dependency paragraph (line 27) with one sentence: the bodyless bind POST transmits `Content-Length: 0` with zero body bytes (ureq 2 `send_bytes(&[])`, loopback-pinned after the HTTP 411 incident log of 2026-10-22) — no dependency change.
- `.agent/project-info/context.md`: add a dated bullet under "Recent Changes (2026-10-22)" describing: the incident, the fix (mechanism (a) chosen; fallback order recorded), the 3 new loopback tests with names, the gate result (log path + observed count), both artifact sizes, the audit outcome. Keep it secret-free and factual; note the real-Google E2E remains operator-run.
- `README.md`: DRIFT CHECK ONLY — read the Production Run section (lines ~120) and the status paragraph; nothing there describes the transport wire form, so expect NO edit. If a factual drift exists (e.g. a transport-behavior claim), fix the minimal line and include it in Commit 2.
- `docs/production-run.md` is the 4.4 docs-specialist's target, BUT the TODO assigns it to Task 1: this plan handoffs the exact edit (see §7) — 4.4 performs it; the implementer does not pre-empt it (keeps the single-section ownership clean; the adherence check in 4.5b verifies).

### Step 12 — Review handoff constraints for 4.3 (encode into the summary returned to the Planner)

- Code-reviewer checklist: (i) mechanism (a) present; the exact arm shape matches §5.1c; (ii) GET arm on `pending.call()` unchanged; O1/O2 arms and all five path/query/body builders byte-unchanged; (iii) the three new tests exist with the D7 names and assertable contracts; (iv) no test contacts Google (loopback `127.0.0.1` only); (v) redaction surfaces unchanged; (vi) no new `allow` attributes; (vii) caps: google_live_api_transport.rs ≤200 lines; both new test files ≤200 lines; `bodyless_post_or_get` ≤50 lines, ≤2 params.
- Code-simplifier guardrails: do NOT merge the two new test files, do NOT inline the enforcing server into the existing siblings (cap risk), do NOT change the pinned mechanism without a failing loopback test first (§4 rules), do NOT touch unrelated files.

### Step 13 — Visibility of the "not-done" list in the returned summary

The implementer MUST end with: what was done (each §5 step), what was NOT done (TODO checkbox flips, 4.3 fixes, 4.4 docs/production-run edit, 4.5 adherence, step-5 merge/push), the mechanism verdict + wire evidence, the gate log path + observed count, both artifact sizes, and the audit result.

---

## 6. Console Command Inventory (exact invocations, in order)

| # | Command (VM/container via Alpine-VM MCP unless noted) | Success |
| --- | --- | --- |
| 1 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` | exit 0 |
| 2 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked google_live_api_transport_bind` | 3 tests pass |
| 3 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings` | exit 0 |
| 4 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked stream_insert_posts_documented_path_with_bearer_and_json`; then `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked broadcast_insert_posts_documented_path_with_locked_body` | both pass unchanged |
| 5 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked bind_sends_bodyless_post_with_both_query_ids` | passes unchanged |
| 6 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` | exit 0, `ALL CHECKS PASSED`, record log + test count |
| 7 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` | exit 0, size recorded |
| 8 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh` | exit 0, size recorded |
| 9 | `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ls -l /rust-youtube-streamer/dist` | sizes cross-check |

Host-side (bash tool): `git status`, `git log`, `git diff --stat`, `git add`, `git commit`, `git check-ignore -v`, `git ls-files`, `git grep` — all single commands, no chaining, workdir = repo root.

MCP reminder: `vm_status` before any container command; compose file path absolute `/rust-youtube-streamer/docker-compose.yml`; `--rm` on every run; `--locked` on every cargo invocation (lockfile untouched).

---

## 7. Docs Spec for 4.4 (exact targets, no new files)

1. `docs/production-run.md` — in the "Common ingestion/API failures" list (lines 107–114), INSERT one bullet (same style as its neighbors, alphabetical order not required — insert after the "Authorization/scope" bullet to keep authorization/live-blocking first):
   - Text (guidance): `Length Required (HTTP 411) — YouTube rejected request (HTTP 411, reason 'unspecified')`: surfaced before this fix when the bind request lacked a Content-Length header; FIXED in v0.9.1 (the transport sends `Content-Length: 0` on the bodyless bind POST; pinned by loopback tests). If it re-appears identically, capture the log line and report; delete the leftover `ready` broadcast/stream pair in YouTube Studio (the failed run leaves both — see the Broadcast/Stream Fate section above) before re-running.
   - Also ensure the statement "no automated test contacts Google; the manual verification procedure stays operator-run" is not contradicted anywhere in the file (it already holds — verify only).
2. `.agent/project-structure.md` — see Step 11.
3. `.agent/project-info/tech.md` — see Step 11.
4. `.agent/project-info/context.md` — see Step 11 (the 4.4 pass may extend the implementer's bullet with the docs-update fact).
5. `README.md`, `docs/configuration.md`, `docs/authentication.md`, `docs/build.md`, `docs/build-windows.md`, `docs/demo.md`, `docs/google-cloud-prerequisites.md` — drift check: none should mention the bodyless-POST wire form; expected NO edits anywhere; record "checked, no drift" in the 4.4 summary.
6. Prohibition: NO new docs files; NO doc may claim real-Google verification of this fix (the 2026-10-22 log is an operator-supplied incident fact; keep it attributed as such).

---

## 8. Rule-Compliance Constraints (binding)

- **Max 2 params/function:** `bodyless_post_or_get` (2: method, pending); `EnforcingBindServer::start` (self + accept_body); `reply_for` (2); `classify_through_status_arm` (1); `pre_fix_lengthless_call` (1). No signature may exceed 2 params in this task.
- **≤50-line bodies:** every new/edited function body well under; count when in doubt.
- **≤200-line files:** the two edited source files and the two new test files must stay ≤200 (projection per file: `google_live_api_transport.rs` ≈187, bind_wire ≈150–175, bind_checks ≈110–125). After rustfmt re-wraps, re-count; if a test file exceeds 200 post-fmt, FIRST condense the three tests' inline capture lines into smaller file-local helper functions (never blank-line padding), and as the authorized last resort move test (b) into a third sibling `google_live_api_transport_bind_checks_classify_tests.rs` registered the same way in `google_live_api_transport.rs` — this exact split is pre-authorized with this exact name.
- **Prefer private / no allow / no commented-out code:** free helpers are file-private; only the helpers consumed by the sibling test file carry `pub(super)` (the established test-module pattern).
- **Self-documenting names:** the D7 test names, `EnforcingBindServer`, `ServedRequest`, `is_enforced_request`, `pre_fix_lengthless_call`, `classify_through_status_arm` are binding identifiers.
- **Single-section boolean conditions:** `is_enforced_request` holds the two-part wire rule; call sites use the single call.
- **Offline-only tests:** loopback `127.0.0.1` only; never Google (D10).
- **Redaction guarantees unchanged** (D11); **no exit-code/demo/auth drift** (D6); **lockfile untouched** — `Cargo.lock` was already refreshed by the version-bump step (commit `9afb63d`); every cargo command in this task carries `--locked` so the refresh stays verified; NO `Cargo.toml` change of any kind.
- **gitignore compliance:** read `.gitignore` and run `git status` before either commit; never stage `dist/`, `videos/`, `logs/*`, `config/config.json`, `credentials/*`(except `.gitkeep`), `target/`.

---

## 9. Verification Matrix (plan ↔ TODO Task 1 ↔ acceptance criteria)

| TODO Task 1 bullet / AC | Covered by |
| --- | --- |
| Bodyless POST transmits `Content-Length: 0` + zero body bytes; ureq 2.12.1 verified; loopback-pinned | Steps 1, 3, 4(a); research §1.2; fallback §4 |
| GET keeps `.call()`; O1/O2 byte-stable; five path builders/query/bodies unchanged | D2; Step 6 spot-checks; test (c) |
| (a) bind POST arrives with `Content-Length: 0` + empty body | Test (a) `bind_post_zero_content_length_is_accepted_by_enforcing_server` |
| (b) lengthless POST classifies as observed 411 shape (incident anchor) | Test (b) `pre_fix_lengthless_bind_post_classifies_as_observed_411_shape` (enforcing server + `response_reason_error`) |
| (c) existing request-shape tests updated only where wire legit changed | D3/D6: existing bind test still true (no header-value-dependent assertion); O1/O2 untouched; verified by Step 6 |
| Dev-checks gate `ALL CHECKS PASSED` + log path + observed count | Step 7 |
| Both artifacts rebuilt with `ls -l` sizes | Step 8 |
| Hygiene/secret audit clean | Step 9 |
| Docs: production-run 411 note + cleanup reminder; structure map; context; README drift check | §7 / Steps 11 (implementer metadata), 4.4 (operator-facing docs) |
| No new-dependency; lockfile untouched | D6; `--locked` everywhere |
| Offline-only tests (fakes or `127.0.0.1`) | D10, tests (a)/(b)/(c) |
| auth/demo/exit-code/redaction/shutdown/fate stability | D6, D11; app/ byte-untouched |

---

## 10. Stop-and-Report Conditions (for the implementer)

1. Step 0 state mismatch (branch/HEAD/dirty tree).
2. All three mechanisms fail the loopback wire observation (§4 item 4) — capture + container-source inspection + report; never improvise.
3. Any file outside the D6 set showing modified in `git diff --stat`.
4. Gate fails 3 cycles.
5. Any touch of `Cargo.toml` or `Cargo.lock` — the version-bump step (`9afb63d`) already owns that state; this task has zero changes to either.
6. Anything ambiguous or outside Task 1's scope → return the question to the caller; never assume, never invent.

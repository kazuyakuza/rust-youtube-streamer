# Phase 05 — Task 2 Implementation Plan — Broadcast and Ingestion Lifecycle

- Date: 2026-10-13 (plan authored; implementer runs 2026-10-14)
- TODO authority: `.agent/todos/20261013/20261013-todo-1.md`, Task 2 (`### 2. Broadcast and
  Ingestion Lifecycle`) + the global Scope/Boundaries + Explicitly-Out-of-Scope sections
- Global plan (binding): `.kilo/plans/20261013-phase05-production-youtube-live-runtime.md` —
  D-G1..D-G11 govern; §1.3 ruling: Live Chat acceptance-criteria lines are an error; Live Chat
  is Phase 06 and is absent from every artifact of this task
- Foundation plan (binding, read FIRST): `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
  — its §1 research findings, D-T1-1..D-T1-13 decisions, F1–F5 staged files and staging
  contract are consumed verbatim below; this plan may not contradict them
- Status: APPROVED (per-task plans auto-approved by the user's "Approve Global and Tasks
  Plans" ruling)
- Branch: `feat/phase05-production-live-runtime`; version 0.8.0; base tree: HEAD `cc6daf6`,
  clean, 282 tests green (Task 1 landed)
- This sub-step: 4.1b (Analysis & Planning). The implementer executes §6 in 4.2; reviewers
  use §8 in 4.3–4.5b.

---

## 1. Verified Grounding (official docs, re-checked 2026-10-14)

Task 1 (§1 of its plan) verified the five operations O1–O5, the scope sufficiency
(`youtube.force-ssl` alone, `REQUIRED_SCOPE` unchanged), the exact response fields, the
lifecycle label sets, the RTMPS destination format and the §1.5 failure table. Task 2 adds
one fresh verification relevant to the concrete transport:

- **live\* API methods use OAuth 2.0 Bearer tokens, never API keys.** The Live Streaming API
  authentication page (https://developers.google.com/youtube/v3/live/authentication,
  "Last updated 2026-09-14 UTC", fetched fresh this planning step) states: "The YouTube Live
  Streaming API uses OAuth 2.0 for authorizing access to private user data", and that
  service-account authorization is NOT supported ("Since there is no way to link a Service
  Account to a YouTube account"). Each live\* reference page (liveBroadcasts/liveStreams)
  states the request "requires authorization with at least one of the following scopes"
  (`youtube` or `youtube.force-ssl` — Task 1 §1.2). An API key identifies a project and
  grants access only to public data; no `key=` query parameter is ever sent. Every request
  in this task sends exactly one credential: `Authorization: Bearer <access_token>` built
  from the Phase-04 `AccessToken` secret wrapper.
- Bind with no body: https://developers.google.com/youtube/v3/live/docs/liveBroadcasts
  (bind row: "Do not provide a request body when calling this method").
- Reason-code table: https://developers.google.com/youtube/v3/live/docs/errors.
- RTMPS acquisition: https://developers.google.com/youtube/v3/live/guides/rtmps-ingestion
  (insert response `cdn.ingestionInfo.rtmpsIngestionAddress`; liveStreams reference:
  concatenation form `STREAM_URL/STREAM_NAME`).
- Quota costs (insert/bind = 50 units, list ≈ 1 unit; $0/10,000 per day):
  `docs/google-cloud-prerequisites.md` + Task 1 §2 D-T1-12.

---

## 2. Binding Design Decisions (D-T2-1 .. D-T2-16)

Refining Task 1's D-T1-1..D-T1-13 for the concrete implementation. Any deviation requires a
planner adjudication.

- **D-T2-1 — Test-transport architecture.** Offline tests never implement HTTP: fake
  transports implement the staged `LiveApiTransport` trait (the credential-service
  shared-fakes precedent, `credential_service_tests.rs` / `..._refresh_tests.rs`). The
  concrete ureq assembly is verified with the Phase-04 loopback-server precedent
  (`google_authorization_verifier_tests.rs`: a canned `127.0.0.1` server, one connection per
  `start`, recorded request line/headers). Ruling: loopback tests live ONLY in the transport
  test file; orchestration/wait/extraction tests use fakes only. NO test contacts Google.
- **D-T2-2 — Concrete transport identity.** One struct `GoogleLiveApiTransport` in
  `src/youtube/google_live_api_transport.rs` implementing `LiveApiTransport`. This supersedes
  the Task-1 parenthetically-planned `google_live_api.rs` name (D-T1-3 wording
  "`google_live_api` planned there"): the suffix matches the concrete-backend naming line
  (`google_token_endpoint.rs`, `google_authorization_verifier.rs`). The staged doc comments
  mentioning `google_live_api` (in `api_transport.rs` and the staged `live_models` files) are
  edited to the final name in the same commit that lands the transport.
- **D-T2-3 — Transport construction and token holding.**
  `GoogleLiveApiTransport::new(access_token: AccessToken)` (one argument) builds
  `ureq::AgentBuilder::new().redirects(0).build()` (the SSRF/disabled-redirect agent
  precedent, `GoogleAuthorizationVerifier::new`) and holds the token in a private
  `AccessToken` field. A `#[cfg(test)]` constructor
  `with_base_url(base_url: String, access_token: AccessToken)` substitutes the base URL for
  loopback servers (the `with_token_url`/`with_request_url` test-only precedent; production
  code never calls it). The Bearer header value is formatted per request from
  `access_token.secret()` and never stored, logged or formatted.
- **D-T2-4 — Request forms per operation.**
  | Op | HTTP | Path+query | Body |
  |---|---|---|---|
  | O1 create_stream | POST | `liveStreams?part=snippet,cdn` | `StreamRequestBody::from_command` JSON |
  | O2 create_broadcast | POST | `liveBroadcasts?part=snippet,contentDetails,status` | `BroadcastRequestBody::from_command` JSON |
  | O3 bind | POST, NO body | `liveBroadcasts/bind?id=<b>&streamId=<s>` | none — `ureq::Request::call()` (documented "Do not provide a request body") |
  | O4 lookup_stream | GET | `liveStreams?part=status,cdn&id=<id>` | none |
  | O5 lookup_broadcast | GET | `liveBroadcasts?part=status&id=<id>` | none |
  Base: `LIVE_API_BASE_URL = "https://www.googleapis.com/youtube/v3/"`; paths append after
  the trailing slash. JSON posts use `.send_json(serde_json_value)` with
  `Content-Type: application/json`. Serialization failure of the request-body structs maps to
  `LiveApiError::MalformedResponse { field: "request body" }` (the structs are pure
  String/bool so this is structurally unreachable; it exists so no call site ever `expect`s).
  Query values (broadcast/stream ids) pass through a private percent-encoding helper
  (`encode_query_value`: percent-encode every byte outside RFC 3986 unreserved
  `A-Za-z0-9-._~`, uppercase hex) — defense in depth; Google ids are unreserved in practice.
- **D-T2-5 — Response handling (status-first, one shared path).** A single private helper
  `fn respond(&self, reply: Result<ureq::Response, ureq::Error>) -> Result<String,
  LiveApiError>`: `Ok(response)` (2xx) → `response.into_string()` (read-failure →
  `Network { reason: "the YouTube API response body could not be read" }` fixed text,
  Phase-04 `UNREADABLE_BODY_TEXT` precedent); `Err(ureq::Error::Status(status, response))` →
  read the body best-effort, extract the non-secret server `reason`, route through the STAGED
  `classify_api_error(status, reason)` — the only error classifier; `Err(other)` →
  `Network { reason: other.to_string() }`. A 3xx arrives as `Status(..)` (redirects(0)) and
  falls through `classify_api_error`'s final arm into
  `ApiRejected { status, reason }` — uniform status-first mapping, no special case; tested.
  Response bodies are NEVER embedded into any error: only the extracted reason code is
  surfaced.
- **D-T2-6 — Error-body reason extraction.** The new transport file carries its own private
  `ApiErrorDocument`/`ApiErrorDetail`/`ApiErrorItem` serde views + `extract_reason(body)
  -> Option<String>` (~20 lines), a deliberate local duplicate of the verifier's private
  version: `google_authorization_verifier.rs` is a stable Phase-04 file whose two-line staging
  contract belongs to Task 3's run wiring; refactoring it for sharing risks the credential
  service tests and widens the diff for ~20 lines. The duplicate is documented in the new
  file's docs (the same duplication-over-cross-module-contract ruling as `utc_timestamp`).
- **D-T2-7 — Resource parsing.** O1/O2/O3 replies are single resources: the body parses as
  `LiveStream`/`LiveBroadcast` directly (insert/bind return the resource, not a list —
  official reference pages). O4/O5 replies parse as `LiveStreamList`/`LiveBroadcastList`
  then `items.first()` → `Ok(None)` only when there is verifiably no item for the requested
  id (the staged trait contract). Parse failures → `MalformedResponse { field: ... }` with
  stable field spellings: `"liveStream resource"`, `"liveBroadcast resource"`,
  `"liveStreamList"`, `"liveBroadcastList"`. Multi-item list answers are tolerated:
  `items.first()` wins (an id-restricted answer with >1 item is a provider anomaly; no
  second-choice logic is added).
- **D-T2-8 — Orchestration primitive (file decided: `src/youtube/live_ingestion.rs`).** One
  entry type consumed by Task 3:
  ```rust
  pub(crate) struct LiveBroadcastPreparer {
      stream_command: CreateStreamCommand,
      broadcast_command: CreateBroadcastCommand,
  }
  impl LiveBroadcastPreparer {
      pub(crate) fn from_config(config: &AppConfig, now_epoch_secs: u64) -> Self;
      pub(crate) fn prepare(&self, transport: &dyn LiveApiTransport, cancellation: &WaitCanceller)
          -> Result<PreparedBroadcast, LiveApiError>;
  }
  pub(crate) struct PreparedBroadcast {
      broadcast_id: String,          // non-secret operator-diagnostics id
      ingestion: RtmpsIngestion,     // redacting wrapper; Debug never reveals
  }
  ```
  `from_config` (2 args) builds both commands from the validated `AppConfig`:
  title/description/privacy from `config.youtube.broadcast.*`; the stream title REUSES the
  validated broadcast title (D-T1-6); CDN labels come from
  `StreamCreationCdn::from_video_config(&config.video)` read through the two new pub(crate)
  label accessors (F-T2-6); `scheduled_start_time` =
  `scheduled_start_time_from_epoch(now_epoch_secs)` (staged F3: now + 60 s). `prepare`
  sequence (D-T1-8 steps 3–6):
  1. O1 create_stream → require `id` (absent → `MalformedResponse { field: "id" }`);
  2. O2 create_broadcast → require `id` (absent → `MalformedResponse { field: "id" }`);
  3. O3 bind → require `contentDetails.boundStreamId` (absent →
     `MalformedResponse { field: "contentDetails.boundStreamId" }`); assert it equals the
     stream id (mismatch → `BroadcastState { detail: "boundStreamId mismatch" }`);
  4. `ReadinessWait` bounded poll (D-T2-9) → the confirmed `LiveStream` observation;
  5. `RtmpsIngestion::from_stream(&confirmed_stream)` → destination extraction (D-T2-12);
     returns `PreparedBroadcast`.
- **D-T2-9 — Readiness polling (file decided: `src/youtube/readiness_wait.rs`).**
  Constants (Task-2 owned per D-T1-12):
  `READINESS_BUDGET_SECONDS: u64 = 180`; `POLL_INTERVAL_SECONDS: u64 = 5`;
  `MAX_READINESS_ATTEMPTS = READINESS_BUDGET_SECONDS / POLL_INTERVAL_SECONDS = 36`;
  `CONFIRMATION_BUDGET_SECONDS: u64 = 120`; `MAX_CONFIRMATION_ATTEMPTS = 24`.
  **The wait is attempt-count-bounded, not wall-clock-bounded**: the loop executes at most
  max_attempts poll cycles, sleeping `interval` between them, so total wall time is bounded
  by attempts × interval plus per-call latency. Decision rationale: attempt-counting keeps
  the loop deterministic and offline-testable (a wall-clock deadline with `Instant` cannot
  be time-compressed without a clock-injection type); the wall-time bound still holds. One
  poll cycle per attempt:
  - O4 lookup_stream → decode `status.streamStatus` via the staged
    `StreamStatusKind::from_api_label` (unknown label → propagate its
    `MalformedResponse { field: "status.streamStatus" }` immediately);
    `Ok(None)` on lookup → `ApiNotFound { resource: "liveStream" }` immediately (the
    one-shot flow's resource vanished — fatal, no further polling);
    `StreamStatusKind::Error` → `BroadcastState { detail: "streamStatus error" }`
    immediately (never polled to the deadline — an errored stream never becomes ready);
  - O5 lookup_broadcast → decode `status.lifeCycleStatus` via the staged
    `BroadcastLifecycle::from_api_label`; same None → `ApiNotFound { resource:
    "liveBroadcast" }`; `BroadcastLifecycle::Revoked` →
    `BroadcastState { detail: "lifeCycleStatus revoked" }` immediately;
  - success predicate (D-T1-13): `StreamStatusKind::Ready` observed AND
    `BroadcastLifecycle::Ready` observed (each at least once during the wait); on success
    return the CURRENT O4-observed `LiveStream` (its `cdn` part carries the ingestion info —
    O4 requests `part=status,cdn` deliberately so the confirmation lookup re-confirms the
    destination source).
  Exact API shape:
  ```rust
  pub(crate) struct ReadinessWait<'a> {
      transport: &'a dyn LiveApiTransport,
      stream_id: &'a str,
      broadcast_id: &'a str,
      cancellation: &'a WaitCanceller,
      max_attempts: u32,
      poll_interval_seconds: u64,
  }
  impl<'a> ReadinessWait<'a> {
      pub(crate) fn new(transport, stream_id, broadcast_id, cancellation) -> Self; // constants
      #[cfg(test)] pub(crate) fn with_polling(...max_attempts, poll_interval_seconds) -> Self;
      pub(crate) fn run(&self) -> Result<LiveStream, LiveApiError>;
  }
  /// D-T1-8 step 8: post-start O4-only confirmation wait (called by Task 3).
  pub(crate) fn wait_until_stream_active(
      transport: &dyn LiveApiTransport,
      stream_pair: &StreamObservationRequest,   // { stream_id, cancellation } param object
  ) -> Result<(), LiveApiError>;
  ```
  `run` loop body: `if cancellation.is_cancelled() → Err(WaitCancelled)`; poll; both-ready →
  return; attempts exhausted → `Err(ReadinessTimeout { stage: "readiness" })`; else sleep.
  `wait_until_stream_active`: O4-only until `StreamStatusKind::Active` → `Ok(())`;
  `Error`/None → same immediate mappings; exhausted →
  `ReadinessTimeout { stage: "ingestion-confirmation" }` (D-T1-13 stage spellings); cancelled
  → `WaitCancelled`. The two waits share private observation helpers
  (e.g. `fn stream_observation(stream: Option<LiveStream>) -> Result<StreamStatusKind,
  LiveApiError>`) so both stay tiny and the mapping exists exactly once. A
  `BroadcastLifecycle::Live` observation is NOT a gate and NOT an error — ignored silently
  (the informational logging of it is Task 3's event line, not wait logic).
  NOTE on within-wait error tolerance (binding): matrix of what the loop does with a
  failed poll observation —
  - `Network { .. }` (connect/transport failure) and 5xx `BackendUnavailable`: TRANSIENT —
    consume one attempt and continue polling while attempts remain (brief §12.2's bounded
    retry policy; the only two tolerated classes);
  - any other classified API error (`ApiNotFound`, `InvalidMetadata`, `QuotaLike`,
    `ApiRejected`, `AuthorizationRejected`, `BroadcastState`, `MalformedResponse`,
    `InsufficientPermissions`, `LiveStreamingNotEnabled`, `ChannelLiveBlocked`): IMMEDIATE
    and fatal for the wait call (the classifier output is already actionable; invalid or
    state reasons are never retried, and a vanished resource never reappears).
  No request duplication around insert/bind in any case (brief §12.2).
- **D-T2-10 — Cancellation seam (file decided: `src/youtube/wait_cancellation.rs`).** A
  youtube-local `WaitCanceller` (the `StopFlag` shape mirrored WITHOUT any
  youtube→`crate::app` dependency — the sealed boundary rule forbids importing app):
  ```rust
  #[derive(Clone)]
  pub(crate) struct WaitCanceller { cancelled: Arc<AtomicBool> }
  impl WaitCanceller {
      pub(crate) fn new() -> Self;               // Task-3-consumed: see staging ledger
      pub(crate) fn request_cancel(&self);       // Task-3-consumed (Ctrl+C link)
      pub(crate) fn is_cancelled(&self) -> bool; // consumed by the poll loops today
  }
  ```
  `SeqCst` ordering exactly like `StopFlag`. The Task-3 contract recorded in the module docs:
  the app layer's Ctrl+C handler flips BOTH the app `StopFlag` and a `WaitCanceller` clone
  (or equivalent adaptation at the app layer); `youtube` itself never reads app types.
- **D-T2-11 — Typed cancellation outcome.** A cancellation is NOT a timeout and is not a
  network/backend failure, so Task 2 adds ONE staged-enum variant:
  `LiveApiError::WaitCancelled` (hand-written Display:
  `the YouTube readiness wait was cancelled by the stop request`). The concrete poll loops
  return `Err(LiveApiError::WaitCancelled)` on cancellation; Task 3 maps it to the
  normal-shutdown path (not a fatal failure). The classifier is untouched (a cancellation
  never originates from an API response).
- **D-T2-12 — RTMPS destination extraction (file decided:
  `src/youtube/rtmps_ingestion.rs`).** The youtube-side redacting wrapper:
  ```rust
  pub(crate) struct RtmpsIngestion { url: String }   // assembled destination; PRIVATE
  impl RtmpsIngestion {
      pub(crate) fn from_stream(stream: &LiveStream) -> Result<Self, LiveApiError>;
      #[allow(dead_code)] /* retire at Task 3 app-glue wiring */
      pub(crate) fn into_url(self) -> String;   // the ONE reveal point
  }
  impl fmt::Debug for RtmpsIngestion { "RtmpsIngestion(<redacted>)" }  // no Display, ever
  ```
  - **Assembly rule (locked, D-T1-11 + official concatenation): the url is EXACTLY
    `<rtmpsIngestionAddress>/<streamName>`** — a single `format!("{address}/{name}")`. No
    scheme validation, no port insertion, no URL surgery of any other kind.
  - **No undocumented fallback**: if `rtmpsIngestionAddress` is absent, the assembled
    destination is NOT substituted with `ingestionAddress` (rewriting an rtmp:// address's
    scheme would be undocumented surgery). Missing fields type as
    `DestinationMissing { field }` with the exact API resource-path spellings:
    `"cdn"` (no cdn part), `"cdn.ingestionInfo"`, `"cdn.ingestionInfo.rtmpsIngestionAddress"`,
    `"cdn.ingestionInfo.streamName"`.
  - **Boundary ruling (binding for Task 3): the destination crosses the
    youtube→streaming gap through the `into_url()` reveal point ONLY.** The streaming-owned
    secret wrapper `RtmpsDestination` stays EXACTLY as D-T1-10/D-T1-11 assigned it — a Task 3
    file `src/streaming/rtmps_destination.rs` with the same wrapper shape (redacting `Debug`,
    no `Display`, unformattable internals, used by `src/streaming/` which never imports any
    youtube type). Why two wrapper types: `youtube` may not depend on `streaming` (its
    module-boundary rule) and `streaming` may not depend on `youtube` (its sealed rule), so
    no single shared type can exist without breaking a boundary; the ONE plain `String` the
    `into_url()` call produces exists only inside the Task-3 app-glue expression
    `RtmpsDestination::new(prepared_ingestion.into_url())` (before every consumer —
    argv, errors, logs — re-wraps or never formats it). `PreparedBroadcast`'s derived
    `Debug` stays secret-safe because its only secret-typed field is the redacting
    `RtmpsIngestion` wrapper.
- **D-T2-13 — Local invalid-metadata ruling (TODO bullet "invalid metadata/privacy").
  NO local re-validation.** The config loader already validates non-blank
  title/description and the `private|public|unlisted` enum before any mode runs; duplicating
  those checks here would freeze two validation sources (brief §5.3's single-funnel intent).
  Task 2's `InvalidMetadata` failures are exclusively server-observed, flowing through the
  staged `classify_api_error` (e.g. 403/400 `invalidTitle` →
  `InvalidMetadata { field: "snippet.title" }` with the API resource-path spelling).
  Title/description length limits are likewise the server's contract (invalidTitle is
  typed); no local length pre-check is added.
- **D-T2-14 — Staging retirement, compiler-arbitrated (binding).** The five staged
  module-level `#[allow(dead_code)]` declarations in `src/youtube/mod.rs`
  (`api_transport`, `live_error`, `live_lifecycle`, `live_models`, `scheduled_start_time`)
  and ALL FIVE `#[allow(unused_imports)]` re-export attributes
  (`live_models/mod.rs` ×4, `live_error/mod.rs` ×1) DELETE in this task, together with their
  contract comments. Task 2's transport/orchestrator genuinely reference their items, so the
  re-export `unused_imports` reasoning they documented is gone. HOWEVER the Rust dead-code
  lint is reachability-from-roots: Task 2's new modules are themselves dead-until-Task-3
  (nothing prod-reachable constructs the transport yet), and dead callers do NOT make their
  callees live — so deleting the five allows alone would re-introduce dead-code warnings
  cascading through the whole staged sub-graph (which the five allows currently suppress),
  breaking `clippy -- -D warnings`. Binding resolution, deterministic for a junior
  implementer (the compiler is the arbiter, zero judgment involved):
  1. Delete the five allows + five unused-imports attributes in ONE retirement act (step S9
     below, after every consumer — the concrete transport and the orchestrator — exists so
     no `unused_imports` warning can fire in between); add the NEW task-2 modules with their
     own module-level `#[allow(dead_code)]` staging declarations from their first commits
     (contract comment: "staged for the Task 3 run wiring; retire the allow there" — the
     same credential_service/Phase-04 precedent).
  2. Run `cargo clippy --locked -- -D warnings` (S10).
  3. Transcribe: for EVERY module the compiler still flags with dead-code-style warnings
     (expected: the five newly-unstaged modules, possibly only some of them), re-apply ONE
     module-level `#[allow(dead_code)]` line on that `mod` declaration with the UPDATED
     contract comment: "items consumed by Task 2's backend/orchestrator; truly live when
     Task 3's run wiring roots the chain — retire the allow there". If clippy is clean with
     zero flags, nothing is re-applied and the retirement stands complete. Either way the
     observed outcome is recorded in the adherence step.
  This honors the retirement contract as far as the compiler permits (the staging burden
  moves from the five Task-1 files to the Task-2 consumption roots) and defers the residual
  formality to exactly the wiring act that makes the chain live, mirroring the Phase-04
  credential-service staging semantics.
- **D-T2-15 — No new dependencies.** `Cargo.toml`/`Cargo.lock` are untouched (D-T1-1); the
  transport uses the pinned `ureq 2` + `serde_json` + std only.
- **D-T2-16 — Known transport limitations (documented, NOT silently claimed).**
  (a) `ureq` calls carry no request-level timeout (Phase-04 verifier parity — a stalled TCP
  connection can hang one call; attempt-bounding caps iterations, not a single hung call).
  (b) Dead-code staging shape per D-T2-14. (c) No `transition`/`delete` API calls exist
  anywhere (Task 1 NOT-selected record). (d) The readiness wait tolerates only the two
  transient classes inside the budget; all other API errors abort immediately (no
  request duplication around insert/bind — brief §12.2's duplicate-broadcast risk).

---

## 3. File Plan (F-T2-1 .. F-T2-9; all ≤200 lines, ≤50-line bodies, ≤2 depth, ≤2 params,
private-by-default members; no commented-out code)

### F-T2-1 — `src/youtube/google_live_api_transport.rs` (~190 lines)

Struct + constructors (D-T2-3), the five-operation `impl LiveApiTransport`, shared private
plumbing:

```rust
pub(crate) struct GoogleLiveApiTransport {
    agent: ureq::Agent,
    base_url: String,
    access_token: AccessToken,
}
impl GoogleLiveApiTransport {
    pub(crate) fn new(access_token: AccessToken) -> Self;
    #[cfg(test)] pub(crate) fn with_base_url(base_url: String, access_token: AccessToken) -> Self;
    // ONE private request engine; ≤2 args via the spec parameter object:
    fn request(&self, spec: &LiveApiRequestSpec) -> Result<String, LiveApiError>;
}
struct LiveApiRequestSpec<'a> { path_query: String, body: Option<&'a serde_json::value::Value> }
// spec constructors encode WHICH form:
//   LiveApiRequestSpec::getting(path_query)                       → GET, no body
//   LiveApiRequestSpec::posting(path_query)                       → POST via .call(), no body (O3 bind)
//   LiveApiRequestSpec::posting_json(path_query, &body: &'a Value) → POST + send_json
// The single private engine `fn request(&self, spec: &LiveApiRequestSpec)` matches on the
// spec's body option exactly then routes through the ONE shared `respond` helper — no
// per-operation HTTP logic anywhere else.
impl LiveApiTransport for GoogleLiveApiTransport {
    fn create_stream(&self, command: &CreateStreamCommand) -> Result<LiveStream, LiveApiError>;       // ≤8 lines
    fn create_broadcast(&self, command: &CreateBroadcastCommand) -> Result<LiveBroadcast, LiveApiError>;
    fn bind(&self, command: &BindCommand) -> Result<LiveBroadcast, LiveApiError>;
    fn lookup_stream(&self, id: &str) -> Result<Option<LiveStream>, LiveApiError>;
    fn lookup_broadcast(&self, id: &str) -> Result<Option<LiveBroadcast>, LiveApiError>;
}
// shared: respond(body/classify), parse_resource_body, parse_list_body + first_item,
// extract_reason (D-T2-6 duplicate), encode_query_value, bearer construction
```
Module docs cite §1 (OAuth-not-API-key), D-T1-1..D-T1-5, D-T2-2..D-T2-7, D-T2-16 limits.

### F-T2-2 — `src/youtube/live_ingestion.rs` (~110 lines)

`LiveBroadcastPreparer` + `PreparedBroadcast` + private per-step helpers
(`fn inserted_stream_id`, `fn inserted_broadcast_id`, `fn bound_stream_asserted`,
`fn confirmed_ingestion` — each ≤15 lines, each ≤2 args). Module docs: D-T1-8 steps 3–6,
D-T2-8, D-T2-13, redaction contract (no Display on `PreparedBroadcast`; derived `Debug` is
safe because the secret field's wrapper Debug redacts).

### F-T2-3 — `src/youtube/readiness_wait.rs` (~150 lines)

Constants (D-T2-9), `ReadinessWait`, `wait_until_stream_active`, the shared private
observation helpers (`fn stream_observation`, `fn broadcast_observation`,
`fn sleep_between_polls`, `fn attempt_exhausted`). `wait_until_stream_active` carries an
item-level `#[allow(dead_code)]` (Task-3-consumed only) with the retirement comment.

### F-T2-4 — `src/youtube/wait_cancellation.rs` (~45 lines)

`WaitCanceller` (D-T2-10) + module docs recording the Task-3 Ctrl+C-link contract and the
youtube-local-seam ruling (sealed boundary). ITEM-LEVEL STAGING INSIDE THE MODULE: `new()`
and `request_cancel()` carry individual `#[allow(dead_code)]` attributes with the
"retire when Task 3's Ctrl+C link flips this seam" comment (only `is_cancelled` has a
non-test consumer today — the poll loops; the module itself is therefore NOT whole-module
staged).

### F-T2-5 — `src/youtube/rtmps_ingestion.rs` (~85 lines)

`RtmpsIngestion` (D-T2-12): `from_stream` chain extraction (cdn → ingestion_info → two
required fields → single concatenation), missing-field `DestinationMissing` guards, the
redacting `Debug`, the `into_url` reveal point. No `Display`.

### F-T2-6 — `src/youtube/live_models/` (EDITS, staged files consumed)

1. `stream_cdn.rs`: add two `pub(crate)` read accessors
   `fn resolution(&self) -> &str` and `fn frame_rate(&self) -> &str` (consumed by the
   `CreateStreamCommand` builder in `live_ingestion`; the three fields themselves stay
   `pub(super)` — untouched; the widenings in item 2 cover the remaining out-of-tree
   consumers).
2. Visibility widenings `pub(super)` → `pub(crate)` (precedent: the documented
   `CompletedVideo::new` widening in `streaming/mod.rs`) — EXACTLY these fields, nothing
   else: `LiveStream { id, status, cdn }`, `StreamCdn { ingestion_info }`,
   `IngestionInfo { stream_name, rtmps_ingestion_address }`,
   `StreamStatus { stream_status }`, `BroadcastStatus { life_cycle_status }`,
   `BroadcastContentDetails { bound_stream_id }`.
3. `mod.rs`: delete the four `#[allow(unused_imports)]` attributes + their "Retire when
   Task 2's concrete transport consumes..." comment lines IN THE S9 RETIREMENT ACT
   (D-T2-14 step 1); update the staging paragraph to point at the landed
   transport/orchestrator files in the same act.
   (`LiveStream`/`LiveBroadcast` families and the request bodies are referenced by
   F-T2-1 imports; `StreamCreationCdn` by F-T2-2 — the re-exports become used.)

### F-T2-7 — `src/youtube/live_error/` (EDITS)

1. `mod.rs`: add the `WaitCancelled` variant to `LiveApiError` (D-T2-11; the ADD lands at
   step S3) + delete the `#[allow(unused_imports)]` re-export attribute + comment IN THE
   S9 RETIREMENT ACT (the transport consumes `classify_api_error` by then).
2. `display.rs`: add the `WaitCancelled` display helper arm (one line in the match + one
   ≤10-line helper; `impl std::error::Error` already blanket).
3. `../live_error_tests.rs` + `../live_error_display_tests.rs`: append the new tests listed
   in §4 (mapping-untouched assertion + exact Display line).

### F-T2-8 — `src/youtube/mod.rs` (EDIT)

- Module docs: rewrite the Phase-05 staging paragraph (the five staged files are now
  consumed by the concrete backend/orchestrator; residual module-level allows — whichever
  the compiler demanded per D-T2-14 — retire at Task 3's run wiring; the standing
  no-network test rule holds).
- Declarations: delete the five staged `#[allow(dead_code)]` lines + the three-line
  Task-1 staging comment block (the staged section near the bottom of the file); add the
  new declarations in one alphabetized block with `google_live_api_transport` carrying its
  staging allow (D-T2-14 step 1):
  `mod google_live_api_transport;` (staged), then plain
  `mod live_ingestion; mod readiness_wait; mod rtmps_ingestion; mod wait_cancellation;`
  plus the staged five (as the compiler arbitrates them back per D-T2-14).
- Add any `pub(crate) use` re-exports ONLY where Task 3's app layer will need them —
  NONE in Task 2 (app remains untouched; Task 3 owns wiring/re-export retirement).

### F-T2-9 — `src/youtube/` test siblings (all new, offline)

`google_live_api_transport_tests.rs`, `live_ingestion_tests.rs` (also hosts the shared
`ScriptedLiveApiTransport` fake + the future-import point for `readiness_wait_tests`),
`readiness_wait_tests.rs`, `rtmps_ingestion_tests.rs`, `wait_cancellation_tests.rs`.
Each wires via the established `#[cfg(test)] #[path = "..._tests.rs"] mod ...;` mapping
inside its parent source file.

NO changes to: `src/youtube/api_transport.rs` bodies (only its doc-comment name
correction, D-T2-2), `src/youtube/live_lifecycle.rs` (consumed as-is),
`src/youtube/scheduled_start_time.rs` (consumed as-is), `src/app/**`, `src/streaming/**`,
`src/main.rs`, `Cargo.toml`, `Cargo.lock`, any `docs/` or README file (Task 4), and no
`config/` example change.

---

## 4. Test Roster (offline; every network boundary is either a fake transport or a
`127.0.0.1` loopback server; no Google egress anywhere)

Fixture-secret confinement: all fixture ids/names/urls are `*-fixture` shaped strings
(`rtmps://ingest.example.test/...`, `stream-name-fixture`) — never real-shaped credentials.

### A — `google_live_api_transport_tests.rs` (loopback; fake `127.0.0.1` server)

Loopback helper `RecordingJsonServer` (mirrors the verifier's `FakeApiServer`, plus a
Content-Length-aware body capture so POST bodies are asserted):

1. `stream_insert_posts_documented_path_with_bearer_and_json` — records
   `POST /youtube/v3/liveStreams?part=snippet,cdn`, exact `Authorization: Bearer
   access_token_fixture`, `Content-Type: application/json`, and the body JSON equal to the
   locked `StreamRequestBody` shape.
2. `broadcast_insert_posts_documented_path_with_locked_body` — same for O2, asserting the
   autoStart/autoStop/monitorStream body shape (the Task-1 T1-8 fixture re-decoded from the
   wire for end-to-end proof).
3. `bind_sends_bodyless_post_with_both_query_ids` — request line
   `POST /youtube/v3/liveBroadcasts/bind?id=broadcast-id-fixture&streamId=stream-id-fixture`
   with NO body payload and NO Content-Type.
4. `stream_insert_reply_resource_parses` — canned 200 `liveStream` resource fixture →
   the returned `LiveStream` carries id + ingestion info (fake-shaped values).
5. `forbidden_live_streaming_not_enabled_classifies_from_body_reason` — 403 + reason
   `liveStreamingNotEnabled` → `LiveStreamingNotEnabled`; the rendered error contains the
   reason code and NEVER the message/body text.
6. `unauthorized_maps_to_authorization_rejected` — 401 → `AuthorizationRejected`.
7. `unknown_forbidden_reason_maps_to_api_rejected_reason_only` — 403 + brand-new reason →
   `ApiRejected { status: 403 }`; assert the body TEXT is absent from `Debug`/`Display`.
8. `redirects_are_not_followed` — canned 301 → `ApiRejected` carrying status 301
   (uniform classify path, D-T2-5).
9. `connection_failure_maps_to_network` — connect to a closed port → `Network`.
10. `malformed_insert_body_maps_to_malformed_response` — 200 + `"<html>"` →
    `MalformedResponse { field: "liveStream resource" }`.
11. `empty_stream_list_is_none_while_absent_body_is_error` — 200 `{"items":[]}` →
    `Ok(None)`; 200 `{}` (no items key) → item-missing → `MalformedResponse`.
12. `malformed_list_body_maps_to_malformed_response` — 200 `{"items": 5}` →
    `MalformedResponse { field: "liveStreamList" }`.
13. `query_values_are_percent_encoded` — pure unit test of `encode_query_value`
    (`46-4.5/_~` passthrough case, a reserved char like `/?&=` encoded), no server.

### B — `live_ingestion_tests.rs` (fake transport only)

Shared fake `ScriptedLiveApiTransport` (top of this file): per-operation queued
`Result`s (`VecDeque`-ish `RefCell`s), a recorded operation label sequence
(`Vec<String>` like `["create_stream", ...]`), captured `CreateStreamCommand` /
`CreateBroadcastCommand` / `BindCommand` instances, and a `canceller_flip:
RefCell<Option<WaitCanceller>>` + `cancel_after_broadcast_lookups: Cell<u8>` so the
cancel-during-wait test flips the shared flag mid-loop deterministically. Tests:

1. `prepare_returns_broadcast_id_and_redacted_ingestion` — full happy script
   (insert-stream, insert-broadcast, bind, O4 ready + O5 ready): `broadcast_id` accessor
   matches; `into_url()` == `rtmps://ingest.example.test/ingest/stream-name-fixture`.
2. `prepare_records_exact_commands_from_validated_config` — captured commands equal the
   expected construction: stream command title == broadcast title, labels
   `1080p`/`30fps` for 1920×1080@30; a dedicated sibling test
   `prepare_records_variable_pairing_for_off_table_video` (a 123×456@24 config variant
   builds both labels `variable`); broadcast command
   `scheduled_start_time` == `scheduled_start_time_from_epoch(fixed_now)`.
3. `operator_sequence_is_insert_insert_bind_then_stream_and_broadcast_lookups` — the
   recorded op label sequence is exactly
   `[create_stream, create_broadcast, bind, lookup_stream, lookup_broadcast]`.
4. `stream_resource_without_id_is_malformed` — insert reply `{}` →
   `MalformedResponse { field: "id" }` (and NO subsequent broadcast/bind calls recorded).
5. `broadcast_resource_without_id_is_malformed` — same for O2.
6. `bind_without_bound_stream_id_is_malformed`.
7. `bind_to_a_different_stream_is_broadcast_state` — boundStreamId mismatch →
   `BroadcastState { detail: "boundStreamId mismatch" }`.
8. The three propagation tests (each own-named, no implementer choice):
   `invalid_metadata_propagates_from_stream_creation` (queued
   `InvalidMetadata { field: "snippet.title" }` on create_stream → the SAME variant+payload
   exits `prepare`, and NO later steps recorded),
   `quota_propagates_from_broadcast_creation` (queued `QuotaLike { reason:
   "userRequestsExceedRateLimit" }` on create_broadcast → same variant out, bind never
   called),
   `insufficient_permissions_propagate_from_bind` (queued
   `InsufficientPermissions` on bind → same variant out).
9. `pre_requested_cancel_executes_inserts_then_waits_cancelled` — cancellation
   pre-requested BEFORE any call: `prepare` STILL records create_stream, create_broadcast
   and bind (inserts are NOT cancellation-gated — only the wait loop checks the flag; the
   50-unit insert requests are made exactly once, per brief §12.2), then the wait's first
   iteration check returns `WaitCancelled` with zero lookups.
10. `prepared_broadcast_debug_reveals_no_ingestion_material` —
    `format!("{:?}", prepared)` contains no `rtmps`, no `/ingest`, nothing of the fixture
    stream name.
11. `missing_rtmps_address_in_confirmed_stream_is_destination_missing` — the confirmed
    lookup stream lacks `rtmpsIngestionAddress` →
    `DestinationMissing { field: "cdn.ingestionInfo.rtmpsIngestionAddress" }`.
12. `missing_stream_name_in_confirmed_stream_is_destination_missing`.

### C — `readiness_wait_tests.rs` (fake transport, shared script fake imported via
`use super::live_ingestion_tests::*` — the credential-refresh-tests import precedent)

1. `both_statuses_ready_returns_the_observed_stream` — stream ready + broadcast ready →
   Ok(stream); exactly one attempt (2 lookups) recorded.
2. `never_ready_times_out_as_readiness_with_stage` — always not-ready, `with_polling(2, 0)`
   → `ReadinessTimeout { stage: "readiness" }`; 4 recorded lookups (2 attempts × 2 ops).
3. `cancel_during_wait_ends_with_wait_cancelled` — the fake flips the canceller on the
   first `lookup_broadcast` via the shared-script hook; outcome `WaitCancelled`; the
   recorded sequence proves the EXTRA poll cycles never ran.
4. `stream_error_terminates_immediately_as_broadcast_state`.
5. `broadcast_revoked_terminates_immediately_as_broadcast_state`.
6. `vanished_stream_lookup_none_is_api_not_found` — `ApiNotFound { resource: "liveStream" }`,
   zero further calls.
7. `vanished_broadcast_lookup_none_is_api_not_found`.
8. `unknown_stream_label_is_malformed_response` — `with field "status.streamStatus"`.
9. `pre_cancelled_wait_makes_zero_api_calls`.
10. `confirmation_wait_returns_when_stream_active` — inactive → active → `Ok(())`; no
    broadcast lookup ever recorded (O4-only profile assertion).
11. `confirmation_timeout_carries_the_confirmation_stage` —
    `ReadinessTimeout { stage: "ingestion-confirmation" }` with `with_polling(3, 0)`.
12. `transient_backend_error_consumes_an_attempt_inside_the_budget` — fake: backend
    failure then ready-ready → Ok, total recorded attempts == 2 (the 5xx tolerance ruling).

### D — `rtmps_ingestion_tests.rs`

1. `assembles_exactly_address_slash_stream_name`.
2. `absent_rtmps_address_never_falls_back_to_plain_ingestion_address` — a stream with
   ONLY `ingestionAddress` → `DestinationMissing`, NOT an assembled rtmp url.
3. `absent_stream_name_is_destination_missing`.
4. `absent_cdn_and_absent_ingestion_info_map_in_layers` — `field: "cdn"` and
   `field: "cdn.ingestionInfo"` cases.
5. `debug_is_exactly_the_redacted_marker` — `format!("{:?}", ingestion)` ==
   `RtmpsIngestion(<redacted>)`.
6. `into_url_consumes_and_returns_the_exact_url` — construction → `into_url()` twice-free
   check: the returned String is exact; (compile-time: `into_url` takes `self`).

### E — `wait_cancellation_tests.rs`

1. `clone_shares_one_flag` — `request_cancel` through the clone flips the original.
2. `fresh_canceller_is_not_cancelled` — starts cleared.

### F — `src/youtube/live_error_tests.rs` / `live_error_display_tests.rs` (appends)

1. (display tests) `wait_cancelled_display_is_stable_and_secret_free` — exact line
   equality + no `rtmps`/`Bearer`/`url` text.
2. (classifier tests) `classifier_never_produces_wait_cancelled` — a spot assertion over
   one representative (status, reason) pair per classifier arm confirming none matches.

Expected new-test delta: A 13 + B 14 (items 1–12, item 8 counts as three separate named
tests) + C 12 + D 6 + E 2 + F 2 = **49 new tests**;
provisional total 282 + 49 = **331** (the OBSERVED gate total is authoritative).

---

## 5. Implementation Steps (4.2; sequential; every cargo/verification command via the
Alpine VM `alpine-vm` MCP, `vm_status` check FIRST, Compose-pattern paths; never chained
into one command)

- S0 Git verification: `git status --porcelain` (empty; gitignore-compliance rule),
  `git branch --show-current` (`feat/phase05-production-live-runtime`),
  `git log --oneline -3` (HEAD `cc6daf6` or later Task-1-closure commits — verify, never
  assume). NO normalization or back-out edits.
- S1 Commit the plan (this file, after it exists on disk):
  `git add .kilo/plans/20261013-phase05-task2-broadcast-ingestion-lifecycle.md`
  + `git commit -m "docs: track the phase 05 task 2 implementation plan"`.
- S2 F-T2-4 (`wait_cancellation.rs`) + its 2 tests + declaration
  `mod wait_cancellation;` in `src/youtube/mod.rs`.

  **Precise instruction (removes D-T2-14 step 1's ambiguity for the NEW files): land every
  NEW task-2 file with its own module-level `#[allow(dead_code)]` staging declaration +
  contract comment ("staged for the Task 3 run wiring; retire the allow there") from its
  first commit — the credential_service form, warning-proof by construction. The
  compiler-arbitrated part of D-T2-14 (steps 2–3) concerns ONLY the five ORIGINAL staged
  files after their allows are deleted in S9.**
- S3 F-T2-7 items 1–2 (`WaitCancelled` variant + Display helper) + the F-roster appends —
  FIRST so its only consumer (the wait loop in S4) never references a missing variant.
  `src/youtube/live_error` REMAINS under its staged allow until S9 (editing a staged module
  does not retire it early). Spot-run:
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c
  "cargo test --locked youtube"` → exit 0.
- S4 F-T2-3 (`readiness_wait.rs`) + C-tests + declaration `mod readiness_wait;`
  (staged allow + comment). Spot-run: same Compose `cargo test --locked youtube` → exit 0.
- S5 F-T2-6 edits (stream_cdn accessors + `pub(crate)` widenings, NO allow deletions yet —
  each allowance dies when its real consumer lands, never before).
- S6 F-T2-5 (`rtmps_ingestion.rs`) + D-tests + `mod rtmps_ingestion;` (staged allow).
- S7 F-T2-1 (`google_live_api_transport.rs` — the concrete backend, staged allow + comment)
  + A-loopback tests + `mod google_live_api_transport;`. The transport consumes the staged
  models/commands/bodies/classifier re-exports, so their `unused_imports` allowances become
  falsified the moment this file lands — the deletion waits for the S9 act so the diff
  boundary stays reviewable.
- S8 F-T2-2 (`live_ingestion.rs`) + B-tests + `mod live_ingestion;` (staged allow).
- S9 THE RETIREMENT ACT — the deletion side (D-T2-14 step 1), executed ONLY after
  every consumer exists (S7 transport + S8 orchestrator):
  1. `src/youtube/mod.rs`: delete the five staged modules' `#[allow(dead_code)]` lines +
     the three-line Task-1 staging COMMENT block; re-sort the declarations with the new
     modules in one alphabetized block; rewrite the module-doc staging paragraph (F-T2-8).
  2. `src/youtube/live_models/mod.rs`: delete the four `#[allow(unused_imports)]`
     attributes + contract comments; update the staging paragraph.
  3. `src/youtube/live_error/mod.rs`: delete the single `#[allow(unused_imports)]`
     attribute + comment.
  4. `api_transport.rs` doc comments + the staged `live_models` files' doc comments:
     replace the planned `google_live_api` spelling with the final
     `google_live_api_transport` (D-T2-2).
- S10 D-T2-14 step-2/3 transcription checkpoint:
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c
  "cargo clippy --locked -- -D warnings"`:
  - clean → nothing re-applied; the five staged allows are fully retired (record).
  - flags → for each flagged staged module, re-apply the one-line module-level allow on the
    `mod` declaration with the UPDATED contract comment; a make-good commit follows.
    (Expected, per the reachability analysis: flags WILL name some of the five.)
- S11 Final verification spot-runs + full standard gate (separate VM invocations, one
  command each, never chained):
  `sh -c "cargo test --locked youtube"` → exit 0 (observed youtube-suite count recorded);
  `sh -c "cargo test --locked readiness_wait"` → exit 0;
  `sh -c "cargo fmt --check"` → exit 0 (fix mechanically via `sh -c "cargo fmt"` if ONLY
  formatting fails — never restructure code during formatting);
  the full gate `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust
  sh scripts/dev-checks.sh` — exit 0 with `ALL CHECKS PASSED`; record the
  `logs/checks/<UTC>.log` path and the OBSERVED test total.
- S12 Doc/dead-code ledger refresh: `.agent/project-structure.md` — add the five new
  F-T2 source files + test siblings to the `youtube/` per-file map; rewrite the
  staging-ledger paragraph (the 15-line inventory minus however many of the five original
  module allows truly retired, plus one staging line per new task-2 module, plus any S10
  transcriptions, with a FRESH grep `git grep -n -F "allow(dead_code)" src/` +
  `git grep -n -F "allow(unused_imports)" src/` output recorded and counted). NO other
  `.agent` doc edits (Task 4 owns docs).
- S13 Hygiene: `git status --porcelain` shows ONLY expected source/test/plan/ledger files;
  `git ls-files dist/` empty; no config.json/tokens/credential files staged
  (gitignore-compliance rule).
- S14 Completion is the §6 commit plan fully applied; steps 4.3–4.6 follow in their own
  sub-agent invocations.

## 6. Commit Plan (implementer; feature-branch commits only — NO push, NO merge, NO version
bump; D-G11)

```text
1. docs: track the phase 05 task 2 implementation plan                        (S1)
2. feat: add the youtube-local wait cancellation seam                         (S2)
3. feat: type the wait cancelled live api failure                             (S3)
4. feat: add bounded cancellable live readiness polling                      (S4)
5. feat: extract the rtmps ingestion destination wrapper                      (S5+S6; includes
   the live_models visibility widenings + stream_cdn accessors it consumes)
6. feat: implement the concrete ureq live api transport                       (S7; loopback tests)
7. feat: orchestrate broadcast creation binding and readiness                 (S8)
8. refactor: consume and retire the staged live api design modules           (S9 deletions +
   doc-comment name corrections)
9. fix: restage modules the compiler still flags as dead                      (S10 transcription;
   SKIPPED and recorded as skipped if clippy was clean)
10. docs: map the live ingestion files in project structure                   (S12)
```

Ordering guarantee: each source commit (2–9) leaves the crate compiling and its tests
green because every new file's staged allow + declaration and its consumers land together
(the commit order matches the step order S2→S3→S4→S5/S6→S7→S8→S9→S10 exactly; the typed
variant lands BEFORE its wait-loop consumers, and the retirement act lands AFTER both the
transport and the orchestrator exist). Exact wording may adapt to the actual diff shape;
the green-at-each-point property is the binding rule.

## 7. Composition Pattern (for the implementer's Compose invocations)

All verification happens on the Alpine VM through the two documented Compose forms — the
spot-test form and the gate form (verbatim, separate invocations, exit code authoritative):

```sh
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked youtube"
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo clippy --locked -- -D warnings"
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```

## 8. Adherence Expectations (4.3–4.5b reviewers)

- `src/app/` diff vs `main` empty; `src/streaming/` diff vs `main` EMPTY for Task 2
  (zero streams files touched — Task 3 owns the FFmpeg extension; D-T1-10's Task-3
  assignment holds);
- `Cargo.toml`/`Cargo.lock` diff empty; no dependency line anywhere.
- No `liveChat` anywhere (fresh `git grep -in "liveChat" src/` NEW-code hits zero), no
  `transition(`/`liveBroadcasts.delete` call sites, no `key=` API-key query parameter.
- Redaction audit (fresh): `git grep -n "secret()" src/youtube/` shows the accessor used
  ONLY at the Bearer-header build site inside `google_live_api_transport.rs` (+ the
  pre-existing token-endpoint/verifier sites); no new `Display` impls on any new type
  holding the destination; `RtmpsIngestion::into_url` has exactly one call site
  (Task-2 test code), its production call site is Task 3's app glue.
- Error-body handling: `git grep -n "into_string" src/youtube/google_live_api_transport.rs`
  → the ONE `respond` helper path; no error variant carries response URLs or raw bodies.
- File/line rules: every new/edited file ≤200 lines (compiler-checked), every
  method/function ≤50-line bodies, ≤2 nesting, ≤2 params (param objects used where
  listed), private members by default (`pub(crate)` only where F-T2 lists it).
- Staging ledger arithmetic: the `allow(unused_imports)` grep is expected to return ZERO
  lines after S6; the `allow(dead_code)` count = observed baseline (updated 15-line
  inventory MINUS however many the five originals truly retire PLUS each new task-2
  module staging line PLUS the S7 transcriptions IF clippy flagged them) computed against
  the fresh grep, no estimate issued as fact (D-T2-14).
- Test-profile behavior: dead-code warnings surfacing only in `cargo test` for
  test-only constructors are recorded and adjudicated per the Phase-04 precedent (never
  "fixed" by making test helpers prod-visible).
- Observed-total rule: the gate log's test count is authoritative over this plan's 331
  estimate.
- Documentation drift: no README/docs claims of implemented `run` behavior anywhere;
  `.agent/project-structure.md` matches the tree.

## 9. TODO Task-2 Bullet — Plan Mapping Self-Check

| TODO Task 2 bullet | Plan coverage |
|---|---|
| Implement the YouTube API operations to create a live stream and a broadcast from the validated config, bind them, inspect status until ready | D-T2-4 (O1–O5 forms), D-T2-8 (`prepare` sequence), F-T2-1/F-T2-2 |
| Only documented fields and legal state transitions | D-T2-7 (documented fields only), D-T2-9 (staged label machines, no transitions) |
| Bounded waits, cancellable waits, typed timeout/readiness error; never spin/wait forever | D-T2-9 (attempt-bounded loops, 180s/5s + 120s constants, cancelled seam), D-T2-10/D-T2-11 |
| Extract the ingestion address/name/key and construct the RTMPS destination without logging it | D-T2-12 (single documented concatenation, redacting wrapper, into_url reveal point) |
| Typed, actionable failures for invalid metadata/privacy, missing API permissions, unsupported/disabled Live Streaming, quota/rate limits, HTTP/provider errors, invalid resource responses, readiness timeouts, with redaction | D-T2-5/D-T2-7 (classify path, MalformedResponse), D-T2-13 (invalid-metadata ruling; eligibility/permission/quota split pre-classified), F-T2-5 (DestinationMissing), D-T2-11 (WaitCancelled), redaction rules in D-T2-5/D-T2-12 + roster A/B tests |
| Tests: success create/bind/readiness, out-of-order/invalid states, API errors, timeouts, malformed responses, secret redaction, fake transport | §4 rosters A–F (49 new tests, all offline) |

## 10. Explicitly Out-of-Task (restated for the implementer)

- No `src/app/` change of any kind (`run` placeholder exit-3 untouched — retired by Task 3).
- No `src/streaming/` change (RTMPS destination/args/output extension is Task 3).
- No Live Chat anything (Phase 06; the `MessageSource` byte-seam stays untouched).
- No `liveBroadcasts.transition`/`liveBroadcasts.delete`/`liveStreams.delete` callers.
- No network egress to Google in tests; loopback only (D-T2-1).
- No doc files beyond `.agent/project-structure.md` + the plan itself (report/manual-plan/
  README work belongs to Task 4).

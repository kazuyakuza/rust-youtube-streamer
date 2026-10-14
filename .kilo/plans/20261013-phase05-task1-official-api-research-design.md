# Phase 05 — Task 1 Implementation Plan — Official API Research and Runtime Design

- Date: 2026-10-13
- TODO authority: `.agent/todos/20261013/20261013-todo-1.md`, Task 1 (`### 1. Official API Research and Runtime Design`) + the global Scope/Boundaries + Explicitly-Out-of-Scope sections
- Global plan (binding): `.kilo/plans/20261013-phase05-production-youtube-live-runtime.md` — D-G1..D-G11 govern; §1.3 ruling: the TODO's Live-Chat acceptance-criteria lines are an error; Live Chat is Phase 06
- Status: APPROVED (per-task plans auto-approved by the user's "Approve Global and Tasks Plans" ruling)
- Branch: `feat/phase05-production-live-runtime` (already checked out; version already bumped to 0.8.0)
- Base tree: 254 tests green; previous fresh gate `logs/checks/20261013T230806Z.log`
- This sub-step: 4.1b (Analysis & Planning). The implementer executes §6 of this plan in 4.2; reviewers use §7 in 4.3–4.5b.

---

## 1. Research Findings (official sources, verified 2026-10-13)

All pages state "Last updated 2026-09-14 UTC" or later — current at planning time.

### 1.1 Selected API operations (the complete runtime call set)

Base URL: `https://www.googleapis.com/youtube/v3/`. Every request carries
`Authorization: Bearer <access_token>` via the Phase-04 verified credentials.
Every operation below is documented on its official reference page:

| # | Operation | HTTP request | parts / params | Runtime purpose |
|---|---|---|---|---|
| O1 | `liveStreams.insert` | `POST /youtube/v3/liveStreams?part=snippet,cdn` | body: `snippet.title`, `cdn.ingestionType=rtmp`, `cdn.resolution`, `cdn.frameRate` (see §2 D-T1-6) | Creates the ingestion stream; response yields `id` + `cdn.ingestionInfo` |
| O2 | `liveBroadcasts.insert` | `POST /youtube/v3/liveBroadcasts?part=snippet,contentDetails,status` | body per §2 D-T1-7 | Creates the broadcast; response yields `id` |
| O3 | `liveBroadcasts.bind` | `POST /youtube/v3/liveBroadcasts/bind?id=<broadcast>&streamId=<stream>` | no request body (official: "Do not provide a request body when calling this method") | Links them; response `liveBroadcast` carries `contentDetails.boundStreamId` for the assertion |
| O4 | `liveStreams.list` | `GET /youtube/v3/liveStreams?part=status,cdn&id=<stream>` | readiness polling (streamStatus) | `status.streamStatus` readiness loop; also re-confirms `cdn.ingestionInfo` |
| O5 | `liveBroadcasts.list` | `GET /youtube/v3/liveBroadcasts?part=status&id=<broadcast>` | readiness polling (lifeCycleStatus) | `status.lifeCycleStatus` readiness loop |

NOT called by the Phase 05 runtime (reserved in the taxonomy + documented only):
`liveBroadcasts.transition` (autoStart/autoStop replaces both transitions — see D-T1-7),
`liveBroadcasts.delete`, `liveStreams.delete` (resource-fate policy — see D-T1-9),
`liveBroadcasts.update`, `cuepoint`, all `liveChat*` operations (Phase 06).
Citations: https://developers.google.com/youtube/v3/live/docs/liveBroadcasts,
https://developers.google.com/youtube/v3/live/docs/liveStreams,
https://developers.google.com/youtube/v3/live/docs/errors,
https://developers.google.com/youtube/v3/live/life-of-a-broadcast.

### 1.2 OAuth scope verification (no scope change needed)

- Phase 04 pinned `REQUIRED_SCOPE = https://www.googleapis.com/auth/youtube.force-ssl`
  (`src/youtube/authorization.rs`). The official per-method reference pages each
  state the request "requires authorization with at least one of the following
  scopes": `https://www.googleapis.com/auth/youtube` or
  `https://www.googleapis.com/auth/youtube.force-ssl` (verified for
  liveBroadcasts list/insert/update/bind/delete and liveStreams list/insert/update/delete).
- **Ruling: `youtube.force-ssl` alone is documented-sufficient for every
  selected operation; `REQUIRED_SCOPE` stays unchanged.** The scope table
  description ("See, edit, and permanently delete your YouTube videos, ratings,
  comments and captions") and the channel-eligibility note are cited from
  https://developers.google.com/youtube/v3/live/authentication ("a channel must
  be approved to use the YouTube Live feature … otherwise the API will return an
  `insufficientPermissions` error").

### 1.3 Exact response fields the implementation relies on

From https://developers.google.com/youtube/v3/live/docs/liveBroadcasts and
.../liveStreams (both last updated 2026-09-14):

Broadcast response:
- `id: string` — broadcast ID (== the corresponding `video` resource ID).
- `snippet.title` / `snippet.description` / `snippet.scheduledStartTime` (RFC 3339).
- `status.lifeCycleStatus`: `complete | created | live | liveStarting | ready | revoked | testStarting | testing`.
- `status.privacyStatus`: `private | public | unlisted`; `status.recordingStatus`: `notRecording | recorded | recording`.
- `contentDetails.boundStreamId: string` — set by `bind`; the implementation asserts it equals our stream ID.
- `contentDetails.enableAutoStart` / `contentDetails.enableAutoStop` / `contentDetails.monitorStream.enableMonitorStream` — insert-time levers per D-T1-7.

Stream response:
- `id: string`.
- `cdn.ingestionType`: `dash | hls | rtmp` ("rtmp (which includes RTMPS)" — officially documented).
- `cdn.ingestionInfo.streamName: string` — the stream name/key YouTube assigns.
- `cdn.ingestionInfo.ingestionAddress`, `backupIngestionAddress` — primary/backup RTMP(S) URLs.
- `cdn.ingestionInfo.rtmpsIngestionAddress: string`, `rtmpsBackupIngestionAddress: string` — the RTMPS-specific primary/backup URLs (source of the FFmpeg destination).
- `status.streamStatus`: `active | created | error | inactive | ready` —
  `ready` = "The stream has valid CDN settings"; `active` = "the user is receiving data via the stream".
- `status.healthStatus.status`: `good | ok | bad | noData` (read-only diagnostics; Phase 05 logs it, never gates on it).

Official RTMPS acquisition rule (https://developers.google.com/youtube/v3/live/guides/rtmps-ingestion):
"Send a LiveStreams insert (POST) request … in the response, the
`cdn.ingestionInfo.rtmpsIngestionAddress` field specifies the RTMPS URL", and the
liveStreams reference says the ingestion URL and stream name may need to be
concatenated as **`STREAM_URL/STREAM_NAME`**. **FFmpeg destination format
locking rule (binding for D-T1-10/D-T1-11): the output URL passed to FFmpeg is
exactly `<rtmpsIngestionAddress>/<streamName>`, with `<rtmpsIngestionAddress>`
carrying the `rtmps://` scheme (port 443 is the RTMPS requirement) — no manual
URL surgery beyond the single documented concatenation.**

Fresh runtime evidence (this planning step, Alpine VM MCP):
`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust ffmpeg -hide_banner -protocols | grep -iE 'rtmps|rtmp'`
→ **exit 0**; `rtmps` (and `rtmp`) appear in BOTH protocol lists of the dev-image
FFmpeg (`5.1.9-0+deb12u1`), so the dev-image FFmpeg natively accepts `rtmps://`
output URLs. FFmpeg's rtmp protocol URL syntax `rtmp(s)://server[:port]/app/playpath`
(`/app` = application name, trailing `[/instance]/playpath` = the key) is exactly
the `STREAM_URL/STREAM_NAME` concatenation (FFmpeg-protocols §rtmp — already cited
in the Phase-03 locked-vector record `.kilo/plans/20261011-phase03-task1-ffmpeg-wrapper.md` §0.2).

### 1.4 Lifecycle state machine (documented legal transitions)

Broadcast (`status.lifeCycleStatus`, transitions performed by
`liveBroadcasts.transition` — EXCEPT with autoStart, see D-T1-7):

```
created ──(settings complete)──> ready ──(video flows / autoStart)──> liveStarting ──> live ──(stop / autoStop)──> complete
                                   │
                                   └─(monitor stream path only — NOT used in Phase 05)──> testStarting ──> testing ──> live
revoked: any state → revoked (admin action only — handled as an error state)
```

- `created` = "incomplete settings, not ready to transition"; `ready` = "settings complete, can transition to live"; `liveStarting`/`testStarting` are PENDING observations while "YouTube completes the actions associated with the transition".
- Stuck `liveStarting`/`testStarting` handling is YouTube's own recommended path
  (`liveBroadcasts.delete` + recreate) — an OPERATOR procedure, documented in the
  recovery docs (Task 4), not automated: Phase 05 never deletes.
- With `enableAutoStart=true`, official wording: "you do not need to call the
  `liveBroadcasts.transition` method". With `enableAutoStop=true`: "you do not
  need to complete this step [transition to complete]" — YouTube "automatically
  ends the broadcast around a minute after you stop sending video".

Stream (`status.streamStatus`): `created` (no valid CDN settings) → `ready`
(valid CDN settings) → `inactive | active` (data flowing or not) → `error`.
We insert with `cdn` present, so a `created` status persisting past the
bounded readiness wait is a typed readiness failure, not a wait-spin.
`cdn.ingestionType`, `cdn.resolution`, `cdn.frameRate` are
**immutable after insert** ("The API does not allow you to change the value of
the `cdn.format`, `cdn.frameRate`, `cdn.ingestionType`, or `cdn.resolution`
fields after the stream is created").

### 1.5 Failure mappings (status-first classification; official errors doc)

Google JSON error shape `{"error": {"code": <http>, "reason": "<reason>"}}`
(the reason is the non-secret classification key; bodies are never embedded —
matches the Phase-04 verifier precedent, `google_authorization_verifier.rs`).
The canonical runtime mapping (implemented as the pure classifier in
§4 file F4):

| HTTP | server reason | → `LiveApiError` variant | Actionability text |
|---|---|---|---|
| 403 | `liveStreamingNotEnabled` | `LiveStreamingNotEnabled` | channel not enabled for live streaming; see youtube.com/features |
| 403 | `livePermissionBlocked` | `ChannelLiveBlocked` | cannot stream live at this time; see youtube.com/features |
| 403 | `insufficientLivePermissions` | `InsufficientPermissions` | authorization lacks live permissions; re-run `auth` |
| 403 | `liveBroadcastBindingNotAllowed` | `BroadcastState` | current broadcast status forbids binding |
| 403 | `errorStreamInactive` | `BroadcastState` | transition not allowed while the bound stream is inactive |
| 403 | `invalidTransition` | `BroadcastState` | illegal lifecycle transition requested |
| 403 | `redundantTransition` | `BroadcastState` | already in/processing the requested status |
| 403 | `concurrentBroadcastsExceedLimit` | `QuotaLike` | another broadcast is already live on the channel |
| 403 | `sharedIngestionBroadcastsExceedLimit` | `QuotaLike` | channel broadcast limit on the shared ingestion source |
| 403 | `userRequestsExceedRateLimit` (incl. `rateLimitExceeded`) | `QuotaLike` | too many requests; retry later |
| 404 | `liveBroadcastNotFound` / `liveStreamNotFound` | `ApiNotFound` | resource vanished — fatal for the one-shot flow |
| 400 | `invalidTitle`(1–100 chars) `invalidDescription`(≤5000) `invalidPrivacyStatus` `invalidScheduledStartTime` `invalidScheduledEndTime` `invalidAutoStart` `invalidAutoStop` `invalidFrameRate` `invalidResolution` `invalidIngestionType` `invalidEmbedSetting` | `InvalidMetadata { field }` | fix `youtube.broadcast.*` config and retry |
| 400 | `titleRequired` `scheduledStartTimeRequired` `privacyStatusRequired` `cdnRequired` `ingestionTypeRequired` `resolutionRequired` `frameRateRequired` (etc. `required (400)`) | `InvalidMetadata { field }` (class: internal validation gap — same actionable class) | same |
| 400 | `limitExceeded` / `userBroadcastsExceedLimit` | `QuotaLike` | too many live/scheduled broadcasts; delete some first |
| 5xx | `backendError`, `internalError`, `errorExecutingTransition`, `serviceUnavailable` | `BackendUnavailable` | transient Google-side; single documented retry policy inside the bounded wait |
| 401 | any (server reason `unauthorized` or missing) | `AuthorizationRejected` | the access token was rejected mid-run — re-run the `auth` command |
| 403 | any other reason | `ApiRejected { reason }` | unexpected — displayed with the reason code only |
| 4xx | any other reason | `ApiRejected { reason }` | same |
| transport failure / unreadable body / unknown method-level shape | n/a | `Network { reason }` / `MalformedResponse { field }` | connectivity or contract problems; `MalformedResponse` for missing required fields AND unknown status labels (lifecycle enum parse) |

Classification stays status-FIRST (Phase-04 precedent: statuses outrank reason
strings); reason extraction tolerates unreadable bodies (fixed-class text).

### 1.6 Live Chat deferral (explicit)

No chat scope, no `liveChatMessages.*` research, no gRPC/prost ruling, no chat
polling/continuation design, no chat failure mappings. The chat-source seam
(`src/app/pipeline/source.rs::MessageSource`) stays byte-unchanged so Phase 06
can attach. The typed error taxonomy has NO chat variants (TODO Task 1's
acceptance line and criteria bullets 88–91 chat parts are the §1.3-ruling
defects; never claimed done).

---

## 2. Binding Design Decisions (D-T1-1 .. D-T1-13)

Refining D-G1..D-G11 under Task 1's authority. Any deviation later requires a
planner adjudication like prior phases.

- **D-T1-1 — Dependency ruling: NO new dependencies.** The runtime uses only
  the pinned `ureq 2` + `serde_json` + `serde` (derive) stack plus `std`.
  Direct HTTPS JSON calls over the existing agent pattern (redirects-disabled
  ureq agent, `GoogleAuthorizationVerifier` precedent). No `google-youtube3`
  (brief §3.3), no Tokio (D-G2), no `chrono`/`time` (RFC 3339 is derived from
  the existing hand-rolled civil algorithm — D-T1-5), no gRPC/prost (chat is
  Phase 06). `Cargo.toml`/`Cargo.lock` are untouched by Task 1. Any future
  addition requires a documented justification per the dependency policy.
- **D-T1-2 — Typed API models only from documented fields.** Response structs
  deserialize the exact fields listed in §1.3 (all `Option<T>` + serde
  defaults — missing parts are tolerated and surfaced as typed
  `MalformedResponse` at the *consumer* boundary) and IGNORE unknown fields —
  the deliberate opposite of the config module's `deny_unknown_fields`
  (Google adds resource fields freely; no version bump hijacks us). Request
  bodies are typed serde `Serialize` structs in the same models file.
- **D-T1-3 — Mockable transport seam.** One trait `LiveApiTransport`
  (§4 file F5) with exactly the five selected operations O1–O5.
  The concrete ureq-backed implementation is Task 2
  (`src/youtube/google_live_api.rs` planned there). Offline fakes implement the
  trait in Task 2 tests; no `cfg` split for tests.
- **D-T1-4 — Error taxonomy.** One enum `LiveApiError` (§4 file F4) covering
  the §1.5 table plus `ReadinessTimeout` (bounded-wait exhaustion) and
  `InvalidMetadata { field }` (locally produced before any call: blank
  title/description/privacy — matches the frozen config validation). The
  stream key never flows through this enum: key handling uses the secret
  wrapper type of D-T1-11. Every variant
  has a hand-written `Display` (stable, end-user-reportable, never a panic),
  `Debug` derives fine (the enum carries no secrets — the RTMPS destination
  string never enters this enum), and `std::error::Error::source`.
- **D-T1-5 — Scheduled start time: `now + 1 minute`, RFC 3339, no clock crate.**
  `liveBroadcasts.insert` REQUIRES `snippet.scheduledStartTime` ("must be in the
  future"); with `enableAutoStart=true` the broadcast goes live when video
  flows regardless of the scheduled time (official stage-3 note "broadcast
  starts whenever the channel owner starts streaming"). Constant
  `SCHEDULED_TIME_OFFSET_SECONDS: u64 = 60`; pure helper
  `scheduled_start_time_from_epoch(epoch_secs: u64) -> String` formats
  `YYYY-MM-DDThh:mm:ssZ` reusing the existing civil-from-days math
  (`src/streaming/utc_timestamp.rs` holds the private variant; the new file is
  `src/youtube/scheduled_start_time.rs`, a small pure module duplicating the
  ~40-line formatter because streaming is sealed to youtube boundaries —
  duplication is cheaper than a cross-module contract).
- **D-T1-6 — Stream CDN mapping from validated config.** `cdn.ingestionType`
  is fixed `"rtmp"` (officially the value that ALSO covers RTMPS: valid values
  are `dash|hls|rtmp`, "rtmp (which includes RTMPS)"). `cdn.resolution`/`cdn.frameRate`
  map from validated `config.video`:
  exact `(width,height)` → one of `240p|360p|480p|720p|1080p|1440p|2160p`
  (1920×1080 → `1080p` by construction); exact `fps == 30` → `"30fps"`,
  `fps == 60` → `"60fps"`, anything else → BOTH resolution and frame rate
  become `variable` (official rule: variable resolution requires variable frame rate).
  impl `StreamCreationCdn::from_video_config(...)` is pure + tested. Stream
  `snippet.title` reuses the validated `youtube.broadcast.title` (≤100 chars;
  the stream title rule is 1..128 — always satisfied).
- **D-T1-7 — Broadcast shape: autoStart + autoStop, no monitor stream, no
  transition calls.** The insert body sets
  `contentDetails.enableAutoStart = true`, `contentDetails.enableAutoStop = true`,
  `contentDetails.monitorStream.enableMonitorStream = false`, plus the
  required `snippet.{title, description, scheduledStartTime}` and
  `status.privacyStatus`. WHY (official Life-of-a-Broadcast stage-3 note):
  "This stage is optional. If you don't plan to have a testing stage, consider
  setting enableAutoStart and enableAutoStop to true — YouTube automatically
  starts the broadcast when you start streaming … and automatically ends it
  about a minute after you stop transmitting." This (i) removes the manual
  `testing` stage, (ii) removes BOTH `transition` calls from the runtime path,
  (iii) satisfies "Do not mark startup successful until the broadcast is in a
  streamable state" via the streamStatus polling of §2 D-T1-13, and (iv) makes
  normal shutdown leave the broadcast to auto-complete without our owning a
  `transition(complete)` call. `invalidAutoStart`/`invalidAutoStop` are typed
  `InvalidMetadata` failures (not all channels/broadcasts support the setting —
  official phrasing "Not all broadcasts support this setting") — actionable,
  not retried.
- **D-T1-8 — Orchestration sequence for the production `run`** (Task 3
  implements; Task 1 documents; the brief §7.2 order holds with these pinned
  refinements justified by official docs):
  1. Load config (existing funnel).
  2. Credentials/authorization: `CredentialService::establish_authorization()`
     (load → offline scope gate → verify-first probe → refresh-once → persist →
     re-verify) — the staged Phase-04 service consumed verbatim (D-G4).
  3. Stream resource (O1 — stream first: its failure surfaces
     `liveStreamingNotEnabled`/`livePermissionBlocked` eligibility before the
     broadcast insert; either order is legal).
  4. Broadcast resource (O2, D-T1-7 shape).
  5. Bind (O3); assert response `contentDetails.boundStreamId == stream_id`,
     else `BroadcastState`.
  6. Bounded, cancellable readiness wait (STOP-CHECKED; D-T1-13 mechanics):
     poll O4/O5 alternately until `streamStatus == ready` AND
     `lifeCycleStatus ∈ {ready}` observed at least once each; typical total is
     seconds; budget 180 s, interval 5 s. Extract on success:
     `rtmpsIngestionAddress` + `streamName` → `RtmpsDestination` (D-T1-11).
  7. FFmpeg start targeting the RTMPS destination (arg vector per D-T1-10).
  8. Post-start bounded confirmation wait: poll O4 until `streamStatus == active`
     (budget 120 s, interval 5 s, cancellable — official: transition
     "typically 5–10 seconds … might take up to a minute"). Startup success =
     FFmpeg running AND `streamStatus == active`. A `lifeCycleStatus == live`
     observation is logged once as an info event (never a gate: autoStart fires
     concurrently and out of phase with our poll window).
  9. Continuous chat-free frame loop (existing pipeline; `MessageSource` seam
     untouched; no chat source constructed in Phase 05).
- **D-T1-9 — Runtime shutdown + broadcast/stream resource fate (policy):**
  - Normal shutdown (Ctrl+C or fatal failure after readiness): (1) request the
    frame-loop stop via the shared `StopFlag` (existing seam), (2) request any
    in-progress API readiness wait to cancel (poll loop checks the same
    `StopFlag` pattern at every iteration; the bounded deadlines remain), (3)
    close FFmpeg stdin (EOF) and let `FfmpegProcess::finish`/`Drop` semantics
    drain + wait + kill (existing, unchanged; the known unbounded-wait
    limitation carries), (4) cleanup failures are REPORTED without masking the
    primary error (existing exit-funnel contract).
  - Resource fate: **never delete, never transition on shutdown.** With
    autoStop=true the broadcast auto-completes ~1 minute after streaming stops
    (official). Deleting on a failed/ambiguous path risks the exact
    duplicate-resource and ambiguous-request problems brief §12.2 forbids;
    our stream keeps the default `isReusable=true`, so it remains addressable
    later for manual cleanup via `liveStreams.list(id=...)`. Startup failures
    AFTER O1/O2 succeeded (bind/readiness/FFmpeg/start-failure) leave the
    created/`ready` broadcast and stream in place too; nothing ever went live
    (no video flowed), so nothing is left `live`.
  - Repeat-run accumulation is a REAL documented limitation
    (`userBroadcastsExceedLimit` is the typed outcome) — operator guidance:
    delete stale broadcasts/streams in YouTube Studio (automated cleanup is
    out-of-scope per the brief's non-goals and TODO's Explicitly-Out-of-Scope).
  - Crash: no own cleanup (brief §12.4; manual recovery documented Task 4).
- **D-T1-10 — Minimal streaming-output extension (decided here; implemented in
  Task 3, EXCEPT the `Session` deletion which Task 1 performs — see F6).**
  - `OutputKind::Session` (reserved `session-` variant): REPLACED by nothing —
    deleted in Task 1 (§4 file F6). Rationale: production streaming never
    generates a local filename; `OutputKind` remains a local-recording-only
    discriminator (`OutputKind::Demo` single-variant stays for demo);
    the item-level `#[allow(dead_code)]` retires by deletion exactly as D-G7
    anticipated. Its test assertion (§5 test T-streaming-1) is retired with it.
  - Task 3 adds to `src/streaming/`: `rtmps_destination.rs` — the secret
    wrapper `RtmpsDestination` (see D-T1-11) and `args::build_stream_arguments(
    config, &destination)` = the LOCKED local vector with exactly three
    output-side changes and nothing reordered:
    (i) the output URL replaces the MP4 path as the final argument;
    (ii) `-movflags +faststart` is REMOVED (an MP4-only post-encode container
    rewrite; meaningless for the FLV muxer RTMP/RTMPS uses);
    (iii) `-f flv` is ADDED to the output option group (the other output-side
    options; after `-i pipe:0`) — explicit muxer selection beats
    output-extension detection (no extension exists);
    (iv) `-g <2 × fps>` is ADDED — the official encoder-settings requirement
    (keyframe interval recommended ≤ 4 s, targeting 2 s; derive from configured
    FPS; https://support.google.com/youtube/answer/2853702, the same source the
    Phase-02 plan recorded). `-nostdin -n` and the entire input segment stay
    byte-identical; the local-MP4 vector stays byte-identical for `demo`.
  - `FfmpegProcess` gains no struct fields for the target beyond what
    `start(config, output)` already receives (Task 3 refines the spawn seam
    into `output: LocalRecording` vs `RtmpsDestination` inputs and makes the
    `finish()` non-empty-file verification target-aware); `FrameSink`,
    drain threads, no-shell spawn, drop-kill semantics unchanged.
- **D-T1-11 — Secret wrapper for the RTMPS destination.** New streaming type
  `RtmpsDestination { url: String }` (Task 3 file): `Debug` prints
  `RtmpsDestination(<redacted>)` (the `AccessToken` precedent,
  `src/youtube/token.rs`), NO `Display`, an unformattable internal body —
  the assembled URL (address + `/` + streamName) exists ONLY inside this
  wrapper and inside the argv vector passed to `std::process::Command`.
  **No** error payload, tracing/event field, panic message, debug print, or
  exit report may ever receive it. FFmpeg's own stderr ECHOES the output URL
  (typically `Output #0, flv, to 'rtmps://…/<key>'`) — so Task 3's
  diagnostics-capture path (the 4096-byte tail) runs a redaction pass
  substituting the destination secret substring with `[redacted]` BEFORE any
  diagnostic text is placed in `StreamingError::ExitFailure::diagnostics`;
  the redaction helper is pure + `-f`-tested with a fake destination, and a
  spawn/exit-failure redaction test is mandatory (TODO Task 3 bullet + D-G5).
- **D-T1-12 — Bounded, cancellable waits (mechanics).** One shared
  iteration-contract: `loop { check StopFlag → GET → sleep interval }` with a
  monotonic deadline (the Phase-03 monotonic-clock seam pattern). Constants
  (Task-2 owned): readiness budget `180 s`, post-start confirmation budget
  `120 s`, poll interval `5 s`, max ~36 list calls at 1 quota unit each per
  wait — well inside the $0/10,000-units-per-day model (insert/bind = 50 units
  each; costs per `docs/google-cloud-prerequisites.md` and the official quota
  documentation). No retry loop of its own outside that deadline cycle;
  `BackendUnavailable` (5xx) occurrences WITHIN a wait poll are
  deadline-tolerated (polling continues while budget remains) — outside waits
  (create/bind calls) a 5xx is immediately typed (single call, no request
  duplication, therefore no duplicate-broadcast risk; brief §12.2 honored).
- **D-T1-13 — Success/failure predicates.** Attempt-level: `streamStatus ==
  ready` AND `lifeCycleStatus == ready` (settings-complete, pre-data states);
  Phase-05 startup success: `streamStatus == active` while FFmpeg is running;
  non-ready-broadcast persistence within the budget is `BroadcastState`
  (bounded, actionable); deadline exhaustion is ALWAYS
  `ReadinessTimeout { stage }` where stage names the machine stage (readiness /
  ingestion-confirmation) — never a bare unwrap of elapsed-vs-budget math.

---

## 3. Task-Boundary Ruling (what Task 1's implementation (4.2) does)

Task 1 is design-first and implementation-light. It lands ONLY the pure,
offline-testable design artifacts the TODO bullet explicitly requires
("Define typed API models, transport interfaces, errors, and lifecycle state
transitions") — as staged, compilable `src/youtube/` types guarded by the
established module-level `#[allow(dead_code)]` staging contract (Phase-02/04
precedent), plus ONE small streaming-module retirement (D-T1-10 first bullet).

Task 1 (4.2) does NOT implement:
- the concrete ureq-backed API backend, binding/polling functions, the
  readiness wait loop, or the RTMPS destination assembly (Task 2);
- any `run`/modes/orchestration/OutputTarget extension or FFmpeg change
  (Task 3 — beyond the Session deletion here);
- any README/docs/config-example change (Task 4 owns them; module docs in the
  new files carry the design contract in-task; project-structure refresh is
  part of 4.4 remediation if reviewers flag drift);
- any chat research/code (permanently out of Phase 05);
- any Cargo.toml/Cargo.lock change.

This boundary avoids duplicating Task 2/3 work (the global plan Approach:
Task 2 = "concrete YouTube API layer … reusing transport.rs patterns";
Task 3 = orchestration).

---

## 4. File Plan (exact; all new files must respect ≤200 lines, ≤50-line method
bodies, ≤2 nesting, ≤2 params/method, private-by-default members)

### F1 — `src/youtube/live_models.rs` (NEW, ~170 lines)

Serde models for the documented fields (D-T1-2). `#[derive(Deserialize)]` on
response types with per-field `#[serde(default, rename = "...")]`; request
types `#[derive(Serialize)]` with camelCase renames. NO `deny_unknown_fields`.

```rust
// Response side (deserialize; unknown fields tolerated):
pub(crate) struct LiveBroadcastList { #[serde(default)] items: Vec<LiveBroadcast> }
pub(crate) struct LiveBroadcast {
    id: Option<String>,                            // YouTube-assigned broadcast ID
    snippet: Option<BroadcastSnippet>,
    status: Option<BroadcastStatus>,
    content_details: Option<BroadcastContentDetails>,
}
pub(crate) struct BroadcastSnippet {
    #[serde(default, rename = "scheduledStartTime")] scheduled_start_time: Option<String>,
    #[serde(default)] title: Option<String>,
    #[serde(default)] description: Option<String>,
}
pub(crate) struct BroadcastStatus {
    #[serde(default, rename = "lifeCycleStatus")] life_cycle_status: Option<String>,
    #[serde(default, rename = "recordingStatus")] recording_status: Option<String>,
}
pub(crate) struct BroadcastContentDetails {
    #[serde(default, rename = "boundStreamId")] bound_stream_id: Option<String>,
}
pub(crate) struct LiveStreamList { #[serde(default)] items: Vec<LiveStream> }
pub(crate) struct LiveStream { id: Option<String>, status: Option<StreamStatus>, cdn: Option<StreamCdn> }
pub(crate) struct StreamCdn {
    #[serde(default, rename = "ingestionType")] ingestion_type: Option<String>,
    #[serde(default, rename = "ingestionInfo")] ingestion_info: Option<IngestionInfo>,
}
pub(crate) struct IngestionInfo {
    #[serde(default, rename = "streamName")] stream_name: Option<String>,
    #[serde(default, rename = "ingestionAddress")] ingestion_address: Option<String>,
    #[serde(default, rename = "backupIngestionAddress")] backup_ingestion_address: Option<String>,
    #[serde(default, rename = "rtmpsIngestionAddress")] rtmps_ingestion_address: Option<String>,
    #[serde(default, rename = "rtmpsBackupIngestionAddress")] rtmps_backup_ingestion_address: Option<String>,
}
pub(crate) struct StreamStatus {
    #[serde(default, rename = "streamStatus")] stream_status: Option<String>,
    #[serde(default, rename = "healthStatus")] health_status: Option<StreamHealth>,
}
pub(crate) struct StreamHealth { #[serde(default, rename = "status")] overall: Option<String> }

// Request side (serialize; camelCase renames — transport-agnostic command types):
pub(crate) struct CreateStreamCommand { pub(crate) title: String, pub(crate) resolution: String, pub(crate) frame_rate: String }
pub(crate) struct CreateBroadcastCommand {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) privacy_status: String,
    pub(crate) scheduled_start_time: String,
}
pub(crate) struct BindCommand { pub(crate) broadcast_id: String, pub(crate) stream_id: String }

/// Pure config→CDN mapping (D-T1-6). Takes the typed `VideoConfig` (one
/// argument — max-2-params rule); nonzero risk notes live in the doc comment.
impl StreamCreationCdn {
    pub(crate) const RTMP_INGESTION_TYPE: &'static str = "rtmp";

    pub(crate) fn from_video_config(video: &VideoConfig) -> Self;
    // Private helpers, each ≤2 params:
    //   fn resolution_label(width: u32, height: u32) -> String
    //   fn frame_rate_label(fps: u32) -> String   // 30→"30fps", 60→"60fps", else "variable"
}

// Typed request bodies (Serialize; the exact JSON the API documents) so the
// JSON shape is locked HERE, testable offline (T1 test 8), before Task 2's
// transport exists. Builders take exactly one argument each:
pub(crate) struct BroadcastRequestBody {
    snippet: BroadcastCreationSnippet,          // { title, description, scheduledStartTime }
    content_details: BroadcastCreationDetails,  // { enableAutoStart: true, enableAutoStop: true, monitorStream: { enableMonitorStream: false } }
    status: BroadcastCreationStatus,            // { privacyStatus }
}
impl BroadcastRequestBody { pub(crate) fn from_command(command: &CreateBroadcastCommand) -> Self; }
pub(crate) struct StreamRequestBody {
    snippet: StreamCreationSnippet,             // { title }
    cdn: StreamCreationCdn,                     // { ingestionType: "rtmp", resolution, frameRate }
}
impl StreamRequestBody { pub(crate) fn from_command(command: &CreateStreamCommand) -> Self; }
```

**Locked resolution mapping (planner ruling — the official reference documents
ONLY the label values `240p|360p|480p|720p|1080p|1440p|2160p|variable`, not
pixel tuples; these pairs are the industry-standard tuples behind each label,
and any tuple outside the table maps to the documented `variable` fallback,
which per the official rule also forces the frame rate label to `variable`):**

`(426,240)→"240p"`, `(640,360)→"360p"`, `(640,480)→"480p"`,
`(1280,720)→"720p"`, `(1920,1080)→"1080p"`, `(2560,1440)→"1440p"`,
`(3840,2160)→"2160p"`; anything else → `("variable", "variable")`.
The example/default config 1920×1080@30 → `resolution "1080p"` +
`frame_rate "30fps"` (tested). `use crate::config::model::VideoConfig;` —
allowed: the youtube-boundary rule forbids chat/renderer/streaming deps, not config.

### F2 — `src/youtube/live_lifecycle.rs` (NEW, ~110 lines)

```rust
pub(crate) enum BroadcastLifecycle { Created, Ready, LiveStarting, Live, Complete, Revoked, TestStarting, Testing }
impl BroadcastLifecycle {
    pub(crate) fn from_api_label(label: &str) -> Result<Self, LiveApiError>; // unknown -> MalformedResponse
    pub(crate) fn is_transition_pending(self) -> bool; // LiveStarting | TestStarting
    pub(crate) fn is_streamable_setup(self) -> bool;   // Ready (the Phase-05 readiness gate stage)
}
pub(crate) enum StreamStatusKind { Active, Created, Error, Inactive, Ready }
impl StreamStatusKind {
    pub(crate) fn from_api_label(label: &str) -> Result<Self, LiveApiError>; // unknown -> MalformedResponse
}
// No chat variants exist; a derived `Debug` suffices (the enum holds nothing
// sensitive) and the readiness-poll code consumes booleans and labels only.
```

### F3 — `src/youtube/scheduled_start_time.rs` (NEW, ~80 lines)

```rust
pub(crate) const SCHEDULED_TIME_OFFSET_SECONDS: u64 = 60;

/// RFC 3339 UTC timestamp for `liveBroadcasts.insert`'s required
/// `snippet.scheduledStartTime`: the current clock moment plus the fixed
/// one-minute offset (epoch math saturates at `u64::MAX` — no panic).
pub(crate) fn scheduled_start_time_from_epoch(now_epoch_secs: u64) -> String;

/// Pure `u64`-epoch → `"YYYY-MM-DDThh:mm:ssZ"` formatter (civil-from-days + era
/// math, D-T1-5; the algorithm duplicates `src/streaming/utc_timestamp.rs`'s
/// private implementation on purpose — streaming is sealed against youtube).
private fn format_rfc3339(epoch_secs: u64) -> String;
```

### F4 — `src/youtube/live_error.rs` (NEW, ~150 lines)

```rust
pub(crate) enum LiveApiError {
    InvalidMetadata { field: String },
    LiveStreamingNotEnabled,
    ChannelLiveBlocked,
    InsufficientPermissions,
    AuthorizationRejected,
    QuotaLike { reason: String },
    BroadcastState { detail: String },
    ApiNotFound { resource: String },
    ApiRejected { status: u16, reason: String },
    BackendUnavailable { reason: String },
    Network { reason: String },
    MalformedResponse { field: String },
    ReadinessTimeout { stage: String },
    DestinationMissing { field: String },  // rtmpsIngestionAddress/streamName absent — MalformedResponse alone cannot name the ingestion gap
    // NO secret-bearing field exists in any variant: the RTMPS destination
    // (D-T1-11) never enters this enum.
}
/// Pure canonical §1.5 mapping; exactly TWO arguments (status, reason: Option<&str>).
pub(crate) fn classify_api_error(status: u16, reason: Option<&str>) -> LiveApiError;
/// Display is written per-variant-group helper fns (each body ≤50 lines — the
/// Phase-03 `StreamingError` split precedent). `field` values follow the API
/// resource-path spelling (e.g. invalidTitle → "snippet.title"); locally
/// produced invalid metadata (Task 3's pre-call guards) use config paths
/// ("youtube.broadcast.title").
```

### F5 — `src/youtube/api_transport.rs` (NEW, ~85 lines)

```rust
pub(crate) trait LiveApiTransport {
    fn create_stream(&self, command: &CreateStreamCommand) -> Result<LiveStream, LiveApiError>;
    fn create_broadcast(&self, command: &CreateBroadcastCommand) -> Result<LiveBroadcast, LiveApiError>;
    fn bind(&self, command: &BindCommand) -> Result<LiveBroadcast, LiveApiError>;
    fn lookup_stream(&self, id: &str) -> Result<Option<LiveStream>, LiveApiError>;
    fn lookup_broadcast(&self, id: &str) -> Result<Option<LiveBroadcast>, LiveApiError>;
}
// `lookup_*` return Ok(None) only when the list answer is verifiably empty for
// the requested id — a MalformedResponse body is an error, never a None.
```

### F6 — `src/streaming/video_output.rs` (EDIT, existing)

- Delete the `// `session-` … reserved` variant `Session` (lines ~21–29) and the
  `#[allow(dead_code)]` attribute line; `filename_prefix` collapses to the
  single `OutputKind::Demo => "demo"` arm.
- Doc sentence updated: production streaming uses the RTMPS destination
  (D-T1-10) and generates no local file.

### F7 — `src/youtube/mod.rs` (EDIT, existing)

- Add the NEW staged module declarations with the retirement contract
  (deliberate staging precedent, `credential_service` form):

```rust
// Staged design artifacts for the Phase-05 runtime (TODO Task 1): typed
// models, the lifecycle state machine, typed failures, scheduled-start
// helper and the mockable LiveApiTransport seam. The staging allows retire
// when Task 2 (concrete backend + polling) and Task 3 (run wiring) consume
// them; until then they must not be wired anywhere.
#[allow(dead_code)]
mod live_models;
#[allow(dead_code)]
mod live_lifecycle;
#[allow(dead_code)]
mod live_error;
#[allow(dead_code)]
mod scheduled_start_time;
#[allow(dead_code)]
mod api_transport;
```

- Module-doc sentence added listing the five staged files + the Task-2/3
  consumption plan and the no-network standing rule for their tests.

---

## 5. Test Roster (all offline; no network, no real Google call)

### T1 — `src/youtube/live_models_tests.rs` (NEW, 8 tests)
1. `broadcast_page_item_deserializes_documented_fields` — realistic JSON
   fixture for O3's response: `id`, `status.lifeCycleStatus == "ready"`,
   `status.privacyStatus`, `contentDetails.boundStreamId` all read.
2. `broadcast_page_ignores_unknown_fields` — extra unknown top-level +
   nested fields do not fail deserialization (`etag`, `monetizationDetails`).
3. `stream_page_item_deserializes_ingestion_info_all_five_fields` —
   `streamName` + all four `*IngestionAddress` family members parse (fixture
   values are fake-shaped, never real keys).
4. `empty_items_fall_back_to_default` — `{}` body → empty item vecs.
5. `missing_parts_yield_none_fields` — a response missing `status` parses
   with `None`; consumers (Task 2) convert missing to `MalformedResponse`.
6. `stream_cdn_maps_known_resolution_and_frame_rate` —
   `StreamCreationCdn::from_video_config(&VideoConfig{width:1920, height:1080, fps:30, …})`
   → `resolution = "1080p"`, `frame_rate = "30fps"`, ingestion type constant `"rtmp"`.
7. `stream_cdn_maps_unknown_size_to_variable_pair` — a `VideoConfig` with a
   non-table size (e.g. 123×456) and `fps = 24` → both labels `"variable"`
   (documented pairing rule).
8. `create_commands_serialize_documented_shape` — serialize a broadcast
   command; assert the exact JSON field mapping:
   `{"snippet":{title,description,scheduledStartTime},"contentDetails":{"enableAutoStart":true,"enableAutoStop":true,"monitorStream":{"enableMonitorStream":false}},"status":{"privacyStatus":"unlisted"}}`.

### T2 — `src/youtube/live_lifecycle_tests.rs` (NEW, 7 tests)
1. `every_documented_broadcast_label_parses` — 8 labels → 8 variants.
2. `every_documented_stream_label_parses` — 5 labels → 5 variants.
3. `unknown_broadcast_label_is_malformed_response`.
4. `unknown_stream_label_is_malformed_response`.
5. `only_starting_states_are_transition_pending` — `LiveStarting`/`TestStarting` true; others false.
6. `only_ready_is_streamable_setup` — `Ready` true; `Created`/`LiveStarting`/`Live`/`Complete`/`Revoked`/`TestStarting`/`Testing` false.
7. `labels_parse_case_sensitively` — `"READY"` / `"ready "` fail (exact-match contract per the API).

### T3 — `src/youtube/live_error_tests.rs` (NEW, 9 tests)
1. `live_streaming_not_enabled_maps_from_forbidden_reason` — (403, `liveStreamingNotEnabled`).
2. `channel_live_blocked_maps_from_live_permission_blocked` — (403, `livePermissionBlocked`).
3. `insufficient_permissions_map` — (403, `insufficientLivePermissions`).
4. `broadcast_state_reasons_map` — the four `forbidden` reasons of §1.5 → `BroadcastState`.
5. `invalid_metadata_reasons_map_by_field` — sample 400 invalid-value reasons
   (`invalidTitle`, `invalidPrivacyStatus`, `invalidScheduledStartTime`,
   `invalidAutoStart`, `invalidFrameRate`) → `InvalidMetadata` whose field
   carries the API resource-path spelling from F4's mapping column
   (`snippet.title`, `status.privacyStatus`, `snippet.scheduledStartTime`,
   `contentDetails.enableAutoStart`, `cdn.frameRate`).
6. `not_found_reasons_map` — (404, both reasons).
7. `quota_reasons_map` — rate-limit, concurrent, shared-ingestion, user-broadcasts-limit.
8. `backend_and_unknown_map_status_first` — 5xx reasons → `BackendUnavailable`;
   unknown reasons → `ApiRejected { status, reason }`; 401 → `AuthorizationRejected`.
9. `every_variant_display_stable_and_secret_free` — one asserted exact
   `Display` line per variant (the stable-display precedent,
   `src/youtube/error_tests.rs`) and no variant output contains `rtmps://`,
   `stream key`, or any URL text (regression pair).

### T4 — `src/youtube/scheduled_start_time_tests.rs` (NEW, 4 tests)
1. `epoch_zero_plus_offset_formats_exact` — `format_rfc3339(60)` ==
   `"1970-01-01T00:01:00Z"`.
2. `known_epoch_matches_rfc3339` — one hand-computed 2026 date string.
3. `leap_day_included_correctly` — 2024-02-29 boundary date formats exactly.
4. `offset_is_exactly_one_minute` — `scheduled_start_time_from_epoch(0)` ==
   `format_rfc3339(60)`.

### T-streaming — `src/streaming/video_output_tests.rs` (EDIT)
- T-streaming-1: inside `filename_prefixes_match_outputs`, delete ONLY the
  `OutputKind::Session.filename_prefix()` assertion line (the `Demo`
  assertion stays). The test and the file's suite count stay at 10 tests.

Expected Task-1 delta: **28 new tests → 254 + 28 = 282 total**
(T1 8 + T2 7 + T3 9 + T4 4 = 28; the observed gate total remains authoritative).

---

## 6. Implementation Steps (4.2; tiny + sequential; every run via the Alpine VM `alpine-vm` MCP, `vm_status` first)

- S0 `git status --porcelain` + `git branch --show-current` — verify clean tree
  on `feat/phase05-production-live-runtime` and no `.gitignore`-matching file
  is staged (gitignore-compliance rule). NO normalization/back-out changes.
- S1 Commit the plan (never BEFORE the plan file exists on disk — this file):
  `git add .kilo/plans/20261013-phase05-task1-official-api-research-design.md`
  + `git commit -m "docs: track task 1 phase 05 implementation plan"`.
- S2 Create F1 (`live_models.rs`) + T1 tests, AND append its staged declaration
  to F7 (`src/youtube/mod.rs`) immediately:
  `#[allow(dead_code)] mod live_models;` (the 4-line staging-contract comment
  block is introduced with this first declaration).
- S3 Create F2 (`live_lifecycle.rs`) + T2 + `#[allow(dead_code)] mod live_lifecycle;`.
- S4 Create F3 (`scheduled_start_time.rs`) + T4 + `#[allow(dead_code)] mod scheduled_start_time;`.
- S5 Create F4 (`live_error.rs`) + T3 + `#[allow(dead_code)] mod live_error;` —
  Display split into helper fns so no fn body exceeds 50 lines (the Phase-03
  `StreamingError` split precedent).
- S6 Create F5 (`api_transport.rs`) + `#[allow(dead_code)] mod api_transport;`.
  Each staged file compiles at every commit point from here on (whole-module
  allow covers its items; no item-level spreads).
- S7 Edit F7 (`src/youtube/mod.rs`) module docs: add the design-artifact
  inventory sentence (the five staged files), the retirement contract
  (retire when Task 2's concrete backend and Task 3's run wiring consume
  them), and the standing no-network test rule. One commit covers S7 += S2's
  block extensions.
- S8 Edit F6 (`video_output.rs`): delete the reserved Session variant +
  `#[allow(dead_code)]`, collapse the match, adjust the doc, drop the dead
  test assertion line in `video_output_tests.rs`.
- S9 Spot-runs (separate invocations, never chained):
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked youtube"`
  → exit 0; expected total = 72 existing youtube tests + 28 new = **100 passed**.
  `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh -c "cargo test --locked streaming::video_output"`
  → exit 0; **10 passed** (suite unchanged, one assertion line removed).
- S10 Full gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` —
  exit 0 with `ALL CHECKS PASSED` and `running 282 tests` expected; record the
  `logs/checks/<UTC>.log` path. The OBSERVED total is the authoritative figure.
- S11 Hygiene: `git status --porcelain` shows ONLY expected source/test/plan
  files; `git ls-files dist/` empty; no `config.json`, no tokens file, no
  credential files staged.
- S12 Update `.agent/project-structure.md`: the `src/youtube/` map gains the
  five staged files; the dead-code staging inventory swaps the retired
  `OutputKind::Session` line for the five new module-level staging entries.
  NO other `.agent` doc edits (Task 4/4.4 own those).
- S13 Commit sequence (§7). Task 1 implementation ends here — steps 4.3–4.6
  follow in their own sub-agent invocations.

## 7. Commit Plan (implementer; feature-branch commits only — no push, no merge, no version bump in this task; D-G11)

```text
1. docs: track the phase 05 task 1 implementation plan        (S1)
2. feat: parse youtube live api resource models               (S2 + T1)
3. feat: add broadcast lifecycle state types                  (S3 + T2)
4. feat: derive the scheduled broadcast start time            (S4 + T4)
5. feat: add typed live api failures with status mapping      (S5 + T3)
6. feat: add the mockable live api transport seam             (S6)
7. docs: document the staged live api design modules          (S7)
8. refactor: retire the reserved session output variant       (S8)
9. docs: map the live api design files in project structure   (S12)
```

Each source commit (2–6) leaves the crate compiling and its own tests green
because the `#[allow(dead_code)]` staging declaration lands with each file.
Exact wording may be adjusted to the actual diff shape; the green-at-each-point
property must hold. NO push, NO merge, NO version bump (D-G11).

## 8. Adherence Expectations (4.3–4.5b reviewers)

- Cargo: NO `Cargo.toml`/`Cargo.lock` diff; NO new line in any `config/` file.
- `src/app/` untouched: `git diff main..HEAD -- src/app` is empty.
- `src/streaming/` diff == exactly `video_output.rs` + `video_output_tests.rs`.
- Hygiene: no fixture secret may imitate real credential material in
  non-test files; test files keep the established fixture-secret confinement.
- New files: no commented-out code, no commented-code blocks, module docs
  serve the design contract; the `#[allow(dead_code)]` inventory arithmetic
  is: baseline 11 items (which includes the `Session` variant) − 1 retired
  (`Session`) + 5 new module-level staging entries = **15 expected** — verified
  fresh at review via `git grep -n -F "allow(dead_code)" src/`.
- Observed-total rule: the authoritative test count is the gate log, not this
  plan's 282 estimate.
- Live-Chat deferral honored fresh: new `git grep -in "liveChat" src/` hits
  must be zero (the phrase appears only in the pre-existing module-doc
  citation lines).
- Documentation drift: `.agent/project-structure.md` matches the tree; no
  README/docs claims of implemented `run` behavior anywhere.

## 9. TODO Bullet — Plan Mapping Self-Check

| TODO Task 1 bullet | Where this plan satisfies it |
|---|---|
| Read brief/reports/staged interfaces before designing | Done in 4.1b (this planning step; §1 evidence + Phase 03/04 reports) |
| Research scopes for broadcast/stream management | §1.2 (`youtube.force-ssl` documented-sufficient; REQUIRED_SCOPE unchanged) |
| Research create/bind/readiness/fields/legal transitions | §1.1, §1.3, §1.4 (models D-T1-2; lifecycle D-T1-7/D-T1-13) |
| Research RTMPS ingestion fields + FFmpeg destination format | §1.3 (`rtmpsIngestionAddress`, `STREAM_URL/STREAM_NAME`, dev-image `rtmps` protocol fresh evidence) + D-T1-10 |
| Research failure cases incl. transition conflicts + map them | §1.5 + F4 classifier (notable: transition-conflict reasons mapped even though the runtime no longer calls `transition`) |
| Defer Live Chat scope/API/research to Phase 06 | §1.6, §2 D-G3 restated; no chat artifacts in the file plan |
| Select smallest viable maintained dependencies + document | §2 D-T1-1 (none new; justification + rejection record) |
| Define typed API models/transport/errors/lifecycle (mockable, offline) | §3 ruling + F1–F5 + §5 rosters (all offline/fixture-only) |
| Document orchestration + shutdown before implementation | §2 D-T1-8 (ordered startup), D-T1-9 (shutdown ordering + resource fate policy) |
| Preserve renderer/streaming boundaries | F6's Session retirement is the only streaming change; §8 app-untouched check; D-T1-10 keeps chat/renderer untouched |

---

## 10. Non-Goals / Phase-06 Handoff Pointers (recorded for the back-log)

- Live Chat: the ONLY Phase-05 seam surface is `src/app/pipeline/source.rs::MessageSource`.
  Phase 06 owns: chat scope analysis (likely same `youtube.force-ssl` — to be
  RESEARCHED then, not assumed), `liveChatMessages.list|streamList`,
  ingestion of `snippet.liveChatId` (the broadcast snippet field already kept
  OUT of the Phase-05 models — deliberately omitted field list of
  `BroadcastSnippet`), historical-message baseline, the gRPC/prost evaluation,
  and chat ingestion tests.
- `liveBroadcasts.transition` type support remains documented in the §1.5 table
  for future phases; the runtime private code must NOT grow a transition caller
  in Phase 05 (would contradict autoStart/autoStop).
- Manual verification procedure, README/config docs, completion report: Task 4.

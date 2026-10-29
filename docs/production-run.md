# Production Run

Operator guide for the production `run` command: what one invocation does, what it needs, what happens at startup, what the logs show (without secrets), how shutdown and broadcast fate behave, and the manual verification procedure. This procedure is operator-run; it was NOT executed by the agent.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [What Happens at Startup, in Order](#what-happens-at-startup-in-order)
- [What the Logs Look Like (Without Secrets)](#what-the-logs-look-like-without-secrets)
- [Shutdown Behavior](#shutdown-behavior)
- [Broadcast/Stream Fate (Policy and Honest Limitations)](#broadcaststream-fate-policy-and-honest-limitations)
- [Live Chat (Behavior, Diagnostics, Recovery, Privacy, Quota)](#live-chat-behavior-diagnostics-recovery-privacy-quota)
- [Ingest Probe Procedure (Operator-Run)](#ingest-probe-procedure-operator-run)
- [Known Stream-Health Limitation (Pure-Black MVP Content)](#known-stream-health-limitation-pure-black-mvp-content)
- [Manual Verification Procedure (Operator-Run)](#manual-verification-procedure-operator-run)
- [Out of Scope](#out-of-scope)
- [See Also](#see-also)

## Overview

One `run` invocation:

- creates ONE new broadcast and ONE new live-stream resource from the validated `youtube.broadcast.*` configuration (title, description, privacy status) — there is no broadcast reuse and no multi-broadcast support;
- binds the two resources together, waits for the stream to become ready, and acquires the RTMPS ingestion destination from the API response;
- streams continuous rendered frames to YouTube Live over RTMPS through the configured external FFmpeg executable (`ffmpeg.executable`);
- writes no local recording in stream mode (the local MP4 behavior belongs to the `demo` command only); accepted incoming chat messages are appended to the configured chat log (`chat.log_file`).

The frame loop is continuous and chat-backed: the `run` command reads the created broadcast's Live Chat over a server-streamed connection and renders newly received ordinary messages (author and text) onto the frames; between messages — or while chat is idle or slow — the frames stay black. Detailed Live Chat behavior, diagnostics, failure recovery, privacy, and quota documentation lives in the "Live Chat (Behavior, Diagnostics, Recovery, Privacy, Quota)" section below.

## Prerequisites

1. The Google Cloud setup of [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md): a Google Cloud project with the YouTube Data API v3 enabled, and a Desktop-app OAuth client at `oauth.client_config_path`.
2. Completed authorization via `auth`: issued credentials at `oauth.credentials_path`; re-run `auth` per [`docs/authentication.md`](authentication.md) when the stored credentials are revoked or unrefreshable.
3. FFmpeg installed separately (never bundled with this project); the configured path `ffmpeg.executable` must be correct and runnable.
4. A filled-in `config/config.json` (all six sections). Recommended `privacy_status: "unlisted"` for the first verification.
5. A parseable font at `renderer.font` (the renderer validates it at construction; the OFL-licensed `fonts/Exo2-Light.otf` ships in the repository).
6. Live-streaming access on the target channel: the channel itself must have live streaming enabled — channel phone verification plus a first-time activation window (official guidance: it may take up to 24 hours). The Google Cloud setup in item 1 does NOT enable live access on the channel; the requirements and where to check are documented in [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md) (section "Channel Live-Streaming Access").

## What Happens at Startup, in Order

The ten-stage startup (in operator-facing wording; stop flags are armed before any cloud call):

1. Configuration loads and validates (`config/config.json` or `--config <path>`); on failure the process exits 1 with one `error:` line.
2. The single Ctrl+C handler pair is installed — both stop flags (the frame-loop stop flag and the wait canceller) are armed before any cloud call, so a Ctrl+C during startup is honored.
3. Credentials load from `oauth.credentials_path`; the credential service runs its verify-first probe against the minimal read-only YouTube probe; only if that probe rejects the access token does it refresh exactly once via the stored refresh token, persist the refreshed credentials atomically, and re-verify.
4. The required-scope gate (`youtube.force-ssl`) runs BEFORE any resource creation — an insufficient-scope grant fails fast, before a broadcast or stream exists.
5. The live-stream resource is inserted (`liveStreams.insert`).
6. The broadcast resource is inserted (`liveBroadcasts.insert`, with `enableAutoStart=true`, `enableAutoStop=true`, and no monitor stream). The optional `made_for_kids` config value rides on this same insert as `status.selfDeclaredMadeForKids`; leaving the config field unset keeps YouTube's channel-level made-for-kids default (see `docs/configuration.md`).
7. The broadcast is bound to the stream (`liveBroadcasts.bind`); the `boundStreamId` in the response is asserted.
8. A bounded readiness wait polls until the stream is ready — up to 180 seconds at a 5-second interval; the wait is cancellable by Ctrl+C.
9. The RTMPS ingestion destination is acquired from the API response; it is kept secret and never printed.
10. FFmpeg starts with the RTMPS-target argument set (FLV muxer, 2-second-target GOP derived from the configured FPS, no local output file). After FFmpeg has started, a post-start confirmation wait polls until YouTube reports `streamStatus == active` — up to 120 seconds at a 5-second interval, on a thread parallel with the frame loop (documented refinement of the startup sequence), then the continuous chat-backed frame loop runs (stage 7.5: the chat worker thread starts ONLY after the gate confirms `streamStatus == active`, first resolves the broadcast's live chat id with a bounded 30-second budget, then establishes the server-streamed Live Chat connection (pushed message batches are mapped into the bounded screen channel; the link is re-established only after a drop, carrying the accumulated `nextPageToken` — it is NEVER re-called on a fixed schedule); on Ctrl+C the worker stops FIRST, before FFmpeg's stdin is closed, and the join is bounded — ≤ ~1 s wake, ≤ ~10 s inside a stream-connect window, ≤ ~30 s during the one-shot chat-id resolution window).

Nothing is marked live until the API reports the stream active while FFmpeg is running: the broadcast is not started (`enableAutoStart`) until ingestion has begun, and the application does not declare success before the confirmation gate observes `streamStatus == active`.

## What the Logs Look Like (Without Secrets)

The human-visible lines/events appear in this order (via `tracing` on stderr; stdout carries only the final stop line):

| Order | Line / event | Notes |
| --- | --- | --- |
| 1 | `configuration loaded and validated` | startup, before any cloud call |
| 2 | `broadcast prepared for streaming` | carries the non-secret `broadcast_id` only |
| 3 | `FFmpeg started toward the configured streaming destination` | after the spawn succeeded |
| 4 | `stream confirmed active on YouTube` | after the post-start confirmation gate observes `streamStatus == active` |
| 5 | `public watch link for the live stream` | carries the non-secret `watch_url` field — the watch URL built from the broadcast id, `https://www.youtube.com/watch?v=<broadcast_id>` (an unlisted broadcast is watchable by anyone holding the link; unlisted is not private) |
| 6 | `live chat id resolved` | info; fires after the confirmation gate confirmed `streamStatus == active`; carries the non-secret `live_chat_id` field (an operator diagnostic of the same class as `broadcast_id`) |
| 7 | `live chat baseline established; the history page was discarded` | info; fires once when the worker's first baseline (history-bearing) stream batch is discarded; carries `discarded_messages` (the count of history items dropped — messages sent before the stream went live never appear) |
| 8 | `live chat ended; the stream continues without new chat messages` | info; the graceful end-of-chat signal; the video keeps flowing until Ctrl+C |
| 9 | (gate-timeout only) `confirmation gate timed out; last observed streamStatus` | info; the `stream_status` field is the raw documented `status.streamStatus` label from the ONE tolerated final lookup at failure time (not a per-attempt history) — appears instead of rows 4–5 when the post-start confirmation gate exhausts its budget (the chat rows 6–8 never fire on this path — the worker is never spawned without a confirmed stream) |
| 9 | (gate-timeout, status unobservable) `confirmation gate timed out; the stream status could not be observed` | info; the tolerated final lookup yielded no label (the lookup itself failed, no stream was found, or the resource carries no label) |
| 10 | (end of run, clean exit) `FFmpeg streaming process exited cleanly (exit status 0)` | info; the `exit_code` field; the same event carries `captured diagnostics follow: <redacted stderr tail>`, or `captured diagnostics follow: none captured` when the tail is empty |
| 10 | (end of run, exit failure) `FFmpeg streaming process exited with a failure status` | **warn**; the `exit_code` field (`None` = signal death); the redacted tail follows in the same event — NEVER a second `error:` line (the single-`error:`-line contract holds) |

Ordering: the row-9 events fire only on the confirmation-gate timeout shutdown path, where the gate thread times out BEFORE the frame loop ends — the stop it requests ends the loop, so the row-10 exit evidence follows and both precede the single final `error:` line. Row 10 fires at the end of every run's frame loop: a normal Ctrl+C stop shows the clean-exit info line before the stdout stop line. The chat rows 6–8 appear only on the confirmed-stream path (row 6 after row 5; row 7 at the worker's first baseline batch; row 8 only if/when chat ends gracefully before Ctrl+C).

Standing secret guarantees: OAuth tokens, refresh tokens, client secrets, stream keys, full RTMPS URLs, and authorization codes NEVER appear in logs, error lines, or diagnostics. FFmpeg's own stderr echo of the output URL is replaced with `[redacted]` in captured diagnostics, and the stream key/RTMPS URL is never embedded in command logs, error text, or debug output.

Inspecting the logs: the `run` command writes no operational log file — every operational line is a `tracing` event on stderr. (The one file it appends is the chat log at `chat.log_file`: each accepted Live Chat message is recorded there — see [`docs/configuration.md`](configuration.md); chat-log privacy and handling are covered by the "Privacy" subsection of the Live Chat section below.) Run the executable in a terminal to read the lines live, or redirect stderr to a file for later inspection (the secret guarantees above keep captured lines safe to store). Verbosity follows the exact, case-sensitive `RUST_LOG` environment variable (default `info`); e.g. `RUST_LOG=debug rust-youtube-streamer-service run`. Full filter semantics: [README's Logging (RUST_LOG) guide](../README.md#logging-rust_log).

Identifying FFmpeg failures: every FFmpeg-origin failure ends the run with exit 1 and one `error:` line naming the stage:

- `unable to start the FFmpeg executable '<path>': ...` — `ffmpeg.executable` points at a missing or non-runnable binary; verify the installed path (see [`docs/configuration.md`](configuration.md)).
- `writing a frame to the FFmpeg process failed: broken pipe; the FFmpeg process exited while receiving frames` — FFmpeg died while streaming; the underlying cause appears in its captured stderr.
- `FFmpeg exited with failure status <N>; captured diagnostics follow: ...` (or `FFmpeg terminated by a signal; captured diagnostics follow: ...`) — the tail of FFmpeg's own stderr is appended, with the RTMPS output URL replaced by `[redacted]`.

## Shutdown Behavior

- **Ctrl+C during streaming**: the frame loop stops within at most one frame interval; FFmpeg's stdin is closed (EOF) and the process is waited on; any in-progress API wait is cancelled. Success prints on stdout, verbatim:

  ```
  streaming stopped; the broadcast auto-completes on YouTube after ingestion ends
  ```

  and the process exits 0.
- **Ctrl+C before streaming started**: prints `run stopped by the stop request before streaming started` on stdout and exits 0. No auto-complete claim is made in that case — the broadcast never went live.
- **Any fatal failure**: one redacted `error:` line on stderr and exit 1.

Exit codes:

| Code | Meaning |
| --- | --- |
| 0 | clean stop (Ctrl+C during streaming) or pre-streaming stop |
| 1 | startup or runtime fatal failure |
| 2 | usage error |
| — | exit code 3 no longer exists in the CLI (the `run` placeholder was retired) |

Documented limitations (inherited Phase-03 Ctrl+C list): repeated Ctrl+C is inert — the installed handler replaces default termination, so a process stuck in a long blocking operation must be terminated with OS tools; and the FFmpeg exit wait is unbounded (no timeout configuration field exists).

## Broadcast/Stream Fate (Policy and Honest Limitations)

- The application NEVER deletes and NEVER manually transitions broadcast or stream resources — no `transition`/`delete` calls exist in the binary.
- The broadcast is created with `enableAutoStop=true` (and `enableAutoStart=true`, no monitor stream), so YouTube automatically ends the broadcast about a minute after ingestion stops (official Life-of-a-Broadcast behavior). After a successful clean stop, the broadcast is expected in YouTube Studio as `complete`/ended — confirm there.
- Startup failures after the stream/broadcast inserts leave the `ready` resources in place: nothing went live and no video flowed, but repeated failing runs ACCUMULATE orphaned broadcasts/streams (a typed `userBroadcastsExceedLimit` outcome surfaces when the limit is hit). Operator guidance: delete stale broadcasts/streams in YouTube Studio — automated cleanup is out of scope.
- On an abrupt crash there is no application cleanup: inspect and end any still-live broadcast in YouTube Studio manually.
- Failure recovery pointers: OAuth problems → [`docs/authentication.md`](authentication.md) (recovery table); Google Cloud/API-quota topics → [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md); any still-live broadcast after a crash → this document's "Broadcast/Stream Fate" section above (there is no separate streaming-recovery document — the recovery content lives here).

Common ingestion/API failures (each surfaces as the single redacted `error:` line with exit 1; the run performs no automatic retries):

- Live-streaming eligibility — `live streaming is not enabled; see youtube.com/features` or `this channel cannot stream live now; see youtube.com/features`: enable/verify live streaming on the channel, then delete the leftover `ready` resources in YouTube Studio before re-running; the channel-side requirements and the first-time activation window (up to 24 hours) are documented in [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md) — Channel Live-Streaming Access.
- Authorization/scope — `access token rejected mid-run; run auth again`, `authorization lacks live permissions; run auth again`, or the scope-gate failure naming the required scope: re-authorize with `auth` ([`docs/authentication.md`](authentication.md) recovery table).
- Length Required (HTTP 411) — `YouTube rejected request (HTTP 411, reason 'unspecified')`: surfaced before this fix when the bind request lacked a Content-Length header; FIXED in v0.9.1 (the transport sends `Content-Length: 0` on the bodyless bind POST; pinned by loopback tests). If it re-appears identically, capture the log line and report; delete the leftover `ready` broadcast/stream pair in YouTube Studio (the failed run leaves both — see the Broadcast/Stream Fate section above) before re-running.
- Quota and rate limits — `YouTube quota or rate limit (reason '...'); retry later`, including `userBroadcastsExceedLimit` once accumulated stale resources hit the limit: delete stale broadcasts/streams in YouTube Studio and retry later; quota model: [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md).
- Readiness timeouts — `YouTube readiness wait timed out (stage 'readiness')`, or the post-start confirmation timing out (stage `ingestion-confirmation`): both waits are bounded (180 s / 120 s at 5 s intervals); verify channel eligibility and API enablement, clean up leftover `ready` resources in YouTube Studio, then re-run. On an `ingestion-confirmation` timeout the run also logs the diagnostics evidence lines (table row 9 above); operator diagnosis procedure:

  1. `confirmation gate timed out; last observed streamStatus stream_status="ready"` — YouTube never registered the ingest: the encode side is healthy (see item 4), so investigate the YouTube side — check Studio (made-for-kids channel setting — a per-stream declaration is available: `youtube.broadcast.made_for_kids` in `config/config.json` (docs/configuration.md); stream routing, channel live access, `docs/google-cloud-prerequisites.md` eligibility) and delete the orphaned `ready` broadcast + liveStream pair (the "Broadcast/Stream Fate" section above) before re-running.
  2. `…last observed streamStatus` naming another label (`inactive`, `error`, `created`) — treat it per the documented `liveStreams` status semantics; the `error` label points at the stream's own diagnostics; for the publish-succeeded-but-inactive/`No data` shape (FFmpeg exited cleanly with the full frame budget), run the Ingest Probe Procedure below before re-running.
  3. `…the stream status could not be observed` — the tolerated final lookup itself failed (network/API): resolve connectivity, delete stale Studio resources, then re-run.
  4. The FFmpeg exit lines (table row 10 above) close the encode-side question: a clean exit (exit status 0) with a progress-line stderr tail means FFmpeg encoded and streamed frames for the full budget; a `warn` failure-status line names a broken-piped/encode process — its tail's last lines name the underlying cause, with the RTMPS URL pre-redacted.
  5. Studio cleanup reminder: each failed run leaves an orphaned `ready` pair per the fate policy — delete before re-running, or the account eventually hits `userBroadcastsExceedLimit`.
  6. Honest limits: the mid-loop broken-pipe shape (`writing a frame to the FFmpeg process failed: broken pipe …`) still cannot print the stderr tail (the drain threads are never joined on that path) — the single `error:` line remains the evidence surface there; and the logged streamStatus is the FINAL tolerated lookup's label at failure time, not a per-attempt history.

- Broadcast-state conflicts — `broadcast state forbids this operation (reason '...')`: typically stale resources from earlier failing runs; clean them up in YouTube Studio and re-run.
- Transient backend/network — `YouTube backend temporarily unavailable (reason '...')` or `unable to reach the YouTube API: ...`: resolve connectivity, then re-run.

## Live Chat (Behavior, Diagnostics, Recovery, Privacy, Quota)

### Prerequisites

- Scopes: the already-granted `https://www.googleapis.com/auth/youtube.force-ssl` is documented-sufficient for BOTH chat calls the worker makes — `liveChatMessages.streamList` (the server-streamed retrieval) and the `liveBroadcasts.list` chat-id lookup (official scope bindings re-verified 2026-10-29; the Phase 07 record §1.5, `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md`). Re-running `auth` is NOT required for chat: there has been no scope change since Phase 04, so the existing credentials keep working.
- The standing Google Cloud and channel prerequisites apply unchanged: the Google Cloud project with the YouTube Data API v3 enabled, a Desktop-app OAuth client, and live-streaming access on the channel — see [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md).
- Live chat is enabled by default on live broadcasts and stays available while the event is active (official `liveChatMessage`/`video` resource semantics; Task-1 record §1.1).
- There is NO new configuration field for chat: message pacing is SERVER-PUSHED (messages arrive on the established stream; no poll interval exists at all) — nothing to configure, and no operator override exists. The re-establishment bounds (connect timeout, backoff cap, consecutive-failure budget) are internal constants, not settings.

### Expected Behavior

- Retrieval method: the worker uses the official `liveChatMessages.streamList` server-streaming method — a gRPC over HTTP/2+TLS client built on the published proto, authenticated Bearer-only (never an API key). It establishes ONE connection for the session and receives message batches as they are pushed. If the connection drops, the worker re-establishes with the last `nextPageToken` and the server resumes from where the client left off; the retrieval is NEVER re-called on a fixed/periodic schedule. The Phase 06 `liveChatMessages.list` polling surface was RETIRED in the Phase 07 migration (user ruling, 2026-10-29; the Phase 07 record §12, `.kilo/plans/20261029-phase07-task1-streamlist-research-design.md`).
- Worker timeline (operator terms): the worker thread starts ONLY after the confirmation gate observes `streamStatus == active`. Its FIRST act resolves the broadcast's `snippet.liveChatId` via `liveBroadcasts.list(part=snippet)` with a bounded budget (6 attempts × 5 s = 30 s); absence after the budget is a fatal typed error. The worker then establishes the streamed connection. The FIRST-established session is the BASELINE session: ALL items it delivers are discarded (they are history the connect pushes — never queued, never logged), so messages sent before this moment do not appear, and — because the API documents no boundary between pushed history and pushed live traffic — messages posted DURING the first session are lost too (documented limitation; the window's length is the session's own lifetime). The session's final token is retained. Fresh sessions after a reconnect (they always carry the token) deliver live: ordinary text messages (author + text) are accepted at most one per frame, appended to the chat log, and rendered. Replayed message IDs are dropped (bounded dedup window that survives reconnects). While chat is idle the connection stays silent but stays open (no reconnect churn); the frames continue (black when no visible text).
- Latency: a pushed message is mapped and queued the moment its batch arrives and appears on the watch-page video at the NEXT frame iteration (at the configured 30 FPS: sub-second) AFTER YouTube-side delivery latency, which the application does not control and which is the largest contributor when chat is quiet. The official documentation characterizes the method as low-latency push ("as soon as they are available"); no operator tuning exists and none is needed. The offline pin for the no-added-interval property is `worker_delivery_after_a_stalled_read_adds_no_poll_interval` (arrival → channel within a sub-second wall bound).
- Message/event policy: ONLY `textMessageEvent` messages render. Super Chats, Super Stickers, membership/sponsor/gifting events, milestone chats, polls, bans, tombstones, and any unrecognized future event type are SKIPPED — never displayed, never logged.

### Diagnostics

The three chat events in the "What the Logs Look Like" table above (rows 6–8) are the success-path signals. Failures surface as typed errors through the single redacted `error:` line with exit 1:

- `live chat is unavailable: <detail>` — chat disabled / not found / insufficient permissions, or the chat-id resolution budget exhausted.
- `live chat retrieval stopped by YouTube quota: <detail>` — quota exhaustion.
- `live chat transport failed after repeated attempts: <detail>` — the bounded transient retry budget exhausted.

The `detail` is the redacted Display of the underlying typed YouTube error (e.g. the `liveChatDisabled`/`liveChatNotFound` reasons) — never a raw response body.

On a connection drop the operator CAN observe: nothing while reconnect churn stays inside the budget — a silent re-establishment logs no event and prints no line; the only externally visible sign is a pause in new messages while idle traffic resumes. The operator CANNOT observe: whether the link is currently established, individual reconnect attempts, or the handshake itself. When the bounded persistence budget exhausts (10 consecutive transient failures, backoff capped at 60 s), the typed `live chat transport failed after repeated attempts:` error line surfaces and the run exits 1. Honest limitation: the real TLS/gRPC handshake against Google is validated ONLY by the operator E2E below — the offline tests pin everything around it (proto wire bytes, Bearer metadata, status classification, the resumable loop) with scripted offline doubles.

### Failure Recovery

| Situation | Documented symptom | Outcome | Recovery |
| --- | --- | --- | --- |
| Broadcast ended / chat closed | `chatEndedEvent`, `offlineAt`, or the `FAILED_PRECONDITION` chat-reason AFTER live deliveries (`liveChatEnded`) | GRACEFUL: one info line, the worker stops, video keeps flowing until Ctrl+C | none needed |
| Chat disabled at establishment | the `FAILED_PRECONDITION` chat-reason (`LIVE_CHAT_DISABLED`/`LIVE_CHAT_ENDED` — ambiguous by gRPC code per the official footnote) AT ZERO delivered batches | fatal (exit 1) | confirm chat is enabled on the broadcast, then re-run |
| Wrong/expired chat id | `NOT_FOUND` / the `liveChatNotFound` class | fatal (exit 1) | re-run `run` (each run resolves its own broadcast's chat id) |
| Permissions revoked mid-run | `PERMISSION_DENIED` / 401-403 class | fatal (exit 1) | re-run `auth` ([`docs/authentication.md`](authentication.md) recovery table) |
| Quota exhausted | the typed Quota terminal (the official gRPC-path exhaustion surface is NOT documented — recorded unknown; see Quota below) | fatal (exit 1) | wait for the midnight PT reset, or apply for Google's quota-extension form |
| Dropped/stalled stream link | silent push pause; bounded re-establishment with the retained token (backoff 1 s × 2^(attempt−1) capped 60 s; ≤ 10 consecutive transient failures) | recoverable — invisible while inside the budget | resolve connectivity; if the budget exhausts, the Transport error ends the run — re-run |
| Malformed/partial batch | a batch that fails typed decoding | transient — the whole batch is discarded, never ordinary chat | none (auto re-establishment with the retained token) |
| Chat-id resolution budget | no `liveChatId` after 6 × 5 s | fatal (exit 1) | re-run after the broadcast is confirmed live |

Honest boundedness facts: a Ctrl+C stops the chat worker FIRST (before FFmpeg's stdin EOF); the join is bounded (≤ ~1 s wake; ≤ ~10 s worst case inside a connect window; ≤ ~30 s worst case during the one-shot chat-id resolution window, where the worker sleeps in raw 5 s slices); every re-establishment wait is stop-pollable within one 1 s slice except the one-shot ≤ 10 s connect attempt; and up to 64 queued-but-undelivered messages can be lost at shutdown — they are never logged, so the chat log and the video never diverge.

### Privacy

- NEVER logged anywhere (stderr events, error lines, or diagnostics): message text, author names/display names, individual users' message IDs, access tokens, authorization headers, raw response bodies, stream keys, RTMPS URLs.
- The ONLY place message content (author + text) is recorded is the chat log file itself (`chat.log_file`, default `logs/chat.log`, gitignored under the `logs/*` rule) — that is its documented purpose.
- The `live_chat_id` tracing field on the resolution line is a NON-secret operator diagnostic (same class as `broadcast_id`).

### Quota (What Chat Streaming Costs)

Chat streaming costs **5 units per connection call**: the initial establishment and every re-establishment after a drop are new calls. The one-time chat-id resolution lookup costs 1 unit. The default project quota is 10,000 units/day, reset at midnight Pacific Time.

The accountable worst case under the worker's bounds: a sustained outage re-establishes at most 10 times (consecutive-failure budget) plus the initial attempt — ≤ 11 connection calls ≤ 55 units — before the typed transport error ends the run.

Officially NOT documented (recorded as unknown — never invented and never promised): whether pushes on an ESTABLISHED connection bill per item/per batch; whether connection lifetime itself bills continuously; any server keepalive/idle-drop timing; the appearance of `quotaExceeded`/`dailyLimitExceeded` on the gRPC path. The worker therefore classifies mid-stream quota-shaped failures into its existing bounded rate-limit/transient arm and documents the unknown honestly.

Operator guidance: the Phase 06 per-interval poll-burn profile is GONE (no periodic calls exist — the polling-burn table of the previous build is superseded by this subsection); a session's accountable documented costs are the establishment calls (5 units each) and the 1-unit resolution. The docs make NO quota-neutral promise beyond these documented call costs; a quota extension (free, manually reviewed) remains the only path above the default ceiling.

## Ingest Probe Procedure (Operator-Run)

Why this procedure exists: a run can finish with FFmpeg exiting cleanly at status 0 after streaming the full frame budget (the RTMPS publish succeeded at the protocol level) while the stream still sits inactive and YouTube Studio shows "No data". This operator-run probe discriminates the candidate causes behind that shape: C1 — the ingest expects an audio track; C2 — the streamed content itself fails the ingest signal lock (for example a near-zero-bitrate all-black source); environment levels (FFmpeg build capability, local network/firewall); and the connection/URL level. Like the manual verification procedure below, it is OPERATOR-RUN: the agent did not execute it, never will, and probe results are recorded only by the operator — never as build evidence. The probe results feed the implement-or-skip decision of the ingest-fix task's research gate (recorded in this work's TODO; an operator report back is the expected next step). Background fact (official): YouTube's Live Streaming API documents the configuration issue `noAudioStream` — "The ingestion stream does not contain any audio streams, but it must contain exactly one audio stream." (https://developers.google.com/youtube/v3/live/docs/liveStreams) — and the encoder-settings page instructs that pre-stream tests "should include audio and movement in the video" (https://support.google.com/youtube/answer/2853702).

**Secret-safety rules (read before touching the shell):**

- The stream key is pasted ONLY into the local command line of a terminal you control. It must NEVER appear in any log file, documentation file, issue, screenshot, or commit. The placeholders below (`<RTMPS_INGEST_URL>` and `<STREAM_KEY>`) are replaced by you at run time; the command with the real key is never copied into any persistent artifact.
- Prefer a plain `cmd.exe` window (no persistent history); if your shell persists history (e.g. PowerShell's PSReadLine), clear or start a fresh session afterwards.
- Each probe ends BY ITSELF after 60 s (`-t 60`): no manual kill needed, no orphan process.

**Get a probe destination (your secret):**

- In YouTube Studio (CREATE → Go live), open the Stream tab, create fresh stream settings (or reuse an existing one you are willing to delete afterwards), and copy BOTH the stream URL (an `rtmps://…` server URL) and the stream key. The FFmpeg destination is the concatenation `stream URL + "/" + stream key` — the same shape the application assembles from the API's `rtmpsIngestionAddress` and `streamName` fields, per the official form "STREAM_URL/STREAM_NAME" (https://developers.google.com/youtube/v3/live/docs/liveStreams, `cdn.ingestionInfo.rtmpsIngestionAddress`).
- Keep Studio's stream tab OPEN on a visible surface: the probe outcomes are read there.

**Preflight (local, no network, no secrets):**

- `ffmpeg -devices` — confirm the device list contains `lavfi` (the lavfi input device is an internal FFmpeg component present in standard Windows builds; gyan.dev's builds include "all internal components available for Windows" per https://www.gyan.dev/ffmpeg/builds/). If `lavfi` is absent, stop: your FFmpeg build cannot run this procedure; record that fact (environment-level evidence) instead of improvising.
- `ffmpeg -f lavfi -i "testsrc2=size=160x120:rate=30:duration=3" -f lavfi -i "anullsrc=channel_layout=stereo:sample_rate=48000" -c:a aac -shortest -f null -` — expect a clean exit 0 with no output file. This validates the exact filter/lavfi/`-shortest` syntax shape below against YOUR installed FFmpeg build before any network push.

**Probe i — rich source, video-only (`testsrc2` at the app's locked streaming-vector parameters):**

```bash
ffmpeg -re -f lavfi -i "testsrc2=size=1920x1080:rate=30" -c:v libx264 -preset veryfast -b:v 6000k -g 60 -pix_fmt yuv420p -t 60 -f flv "<RTMPS_INGEST_URL>/<STREAM_KEY>"
```

Options match the application's streaming-vector encoder core (libx264, veryfast, 6000k, GOP 60 frames = the 2-second-target keyframe at 30 fps, yuv420p, FLV muxer) at 1080p30; this probe predates the v0.11.1 derived CBR block (`-maxrate`/`-minrate`/`-bufsize`/`-x264-params`) and omits it; `-re` paces the generated video at native rate; `-t 60` bounds the run. The video content moves continuously.

**Probe ii — black source + silent AAC audio (the v0.11.0 candidate remedy's wire shape):**

```bash
ffmpeg -re -f lavfi -i "color=c=black:size=1920x1080:rate=30" -f lavfi -i "anullsrc=channel_layout=stereo:sample_rate=48000" -c:v libx264 -preset veryfast -b:v 6000k -g 60 -pix_fmt yuv420p -c:a aac -b:a 128k -shortest -t 60 -f flv "<RTMPS_INGEST_URL>/<STREAM_KEY>"
```

Same video vector (all-black content = the application's frame content); adds a second lavfi input — infinite silence at 48 kHz stereo (`anullsrc`, long option names `channel_layout`/`sample_rate` per FFmpeg's `asrc_anullsrc` source) — encoded AAC at 128 kbit/s (the officially recommended stereo audio bitrate/codec), cut by `-shortest` when the bounded video input ends (FFmpeg: "Finish encoding when the shortest output stream ends.").

**Probe iii — rich source + silent AAC audio (control: separates environment/connection from signal):**

```bash
ffmpeg -re -f lavfi -i "testsrc2=size=1920x1080:rate=30" -f lavfi -i "anullsrc=channel_layout=stereo:sample_rate=48000" -c:v libx264 -preset veryfast -b:v 6000k -g 60 -pix_fmt yuv420p -c:a aac -b:a 128k -shortest -t 60 -f flv "<RTMPS_INGEST_URL>/<STREAM_KEY>"
```

Same shape as probe ii with the moving `testsrc2` content — a positive signal in both halves.

**How to observe (during each 60 s run):** watch the Studio stream page: whether the ingest indicator/preview shows incoming data (the exact wording YouTube shows varies — record what you see, with the timestamp). A probe is ACTIVE when the preview/health indicator shows the stream receiving data; a probe is INACTIVE when Studio keeps showing "No data".

**Outcome → cause mapping (decisive outcomes):**

| Outcome | Meaning | Next step |
| --- | --- | --- |
| Probe (iii) errors locally (protocol/encoder missing) | The local encoder build lacks lavfi/rtmps support | Fix the encoder build first (gyan.dev full build), then re-run |
| Probe (i) goes active | Video-only IS accepted — audio is not the blocker; the black video-only signal is the problem (content-level) | Record the results and report them to the planner; a product-level content decision is required (the silent-audio remedy alone does not apply) |
| Probe (i) stays inactive, probe (ii) goes active | The audio track is the discriminator — YouTube's ingest requires an audio stream for the signal lock | The silent-audio remedy applies; proceed on that path |
| Probes (i) and (ii) stay inactive, probe (iii) goes active | Audio is necessary but not sufficient — the black-only low-bitrate signal also fails the lock | Record the results and report them to the planner; a product-level content decision is required beyond the audio remedy |
| All three probes stay inactive (no local errors) | Environment/URL-level: network, firewall, the encoder→YouTube path, or the key/URL shape — not a signal-content issue | Check network/firewall and the URL/key handling; no application change is warranted; record the findings |

Record each outcome with the timestamp and the exact Studio wording. If ALL probes are inactive, also re-run probe i once with a freshly copied URL+key before concluding connection/URL-level.

**Cleanup after the probes:**

- Delete the probe's stream/broadcast pair in YouTube Studio (the same hygiene the production fate policy requires of failed `run` attempts) — do not let probe resources accumulate.
- If any probe accidentally reached the live state, let it end/stop it, and verify the broadcast completed in Studio.
- Remember: the key was only ever on your local command line — nothing to purge from this repository, and nothing to paste anywhere.

**Product-level status (v0.11.1):** the streaming vector now enforces true CBR
video (`-maxrate`/`-minrate` mirroring the configured bitrate, `-bufsize` at 2×
that bitrate, plus the x264 `nal-hrd=cbr` filler) and encodes the silent track
as MP3-CBR at 128 kbit/s (`-c:a libmp3lame -b:a 128k`) — addressing the
measured-bitrate shape behind both Studio health warnings ("not receiving
enough video to maintain smooth streaming", characterized at ~18.5 kbit/s
against the 6000k stance, and "audio bitrate (0) lower than the recommended
128 Kbps", measured at ~2 kbit/s on digital silence). What the remedy does NOT
change: the video content stays black/filler-padded, the `anullsrc` audio is
lossless silence, and any further Studio advisory seeded on content movement
(Google's tests "should include audio and movement") may remain cosmetic. The
operator's FFmpeg build must include libmp3lame (the standard full builds do);
a build lacking it fails at stream start with the clear spawn-error path — see
[`docs/configuration.md`](configuration.md). Verification honesty: the
remedy's mechanism is pinned by a local-FLV ffprobe floor gate (measured
container average ≈ the encoder-declared sum, with mux overhead and startup
skew inside the recorded tolerance); the real remedy evidence remains the
OPERATOR's next run against real Studio — this document claims no agent-run
Studio observation. The `ffmpeg.audio.enabled: true` gate (the example config
enables it) still selects whether the vector carries the silent track, while
absent/`false` keeps the stream without any audio track (the derived CBR video
parameters stay). Once a run ships with audio enabled, the remedy path SUPERSEDES this
procedure as the deployed fix; the probes remain the operator's verification
and diagnosis path: if an audio-enabled run still ends with the
publish-succeeded-but-inactive/"No data" shape, the outcome table above maps
the cause PAST C1 (content-level or environment — report the recorded results
to the planner rather than re-judging in the field).

## Known Stream-Health Limitation (Pure-Black MVP Content)

Operator verdict (2026-10-25): the fifth native Windows `run` (v0.11.1) is healthy on the wire,
while YouTube Studio's **Stream Health** tab still flags "YouTube is not receiving enough video to
maintain smooth streaming". The verdict is ACCEPT and DOCUMENT: for this MVP's pure-black content
the flag is expected, content-rooted, and spurious for playback — it is recorded here so operators
do not chase encoder/network causes for it.

- Root cause (documented technical assessment): a 1080p static black frame carries ~zero real
  entropy. The v0.11.1 true-CBR video block (`nal-hrd=cbr`) keeps the WIRE at the target bitrate
  (verified ~6.1 Mbit/s below), but the filler is padding, not content — YouTube's Stream-Health
  measurement still registers the content-level signal poverty.
- Honest attribution: the wire facts are AGENT-VERIFIED from the operator-supplied v0.11.1 log
  (`logs/run/20261025-200332.log`: x264 summary `kb/s:5993.65`, progress `bitrate=6132.3kbits/s`,
  MP3 silent track ~124 kbit/s, `exit_code=Some(0)`, run exit 0). The Stream-Health flag is
  OPERATOR-OBSERVED (Studio Stream Health tab; no bitrate graph shown in that view). The
  zero-entropy root cause is a DOCUMENTED TECHNICAL ASSESSMENT — it is NEVER claimed as
  agent-observed Studio behavior (the agent did not run the manual procedure and never will).
- Operator guidance while the MVP streams a static black screen:
  - EXPECT this warning for as long as the content is pure black; do not treat it as an encoder,
    network, or configuration fault.
  - It is spurious for playback: static content cannot visibly stutter. Verify smoothness on the
    watch page (`run` prints the non-secret watch URL) instead of chasing this specific flag.
  - The separate audio-bitrate warning ("audio bitrate (0) lower than the recommended 128 Kbps")
    was CLEARED by the v0.11.1 MP3-CBR remedy above.
  - This video-content warning is expected to persist while frames stay black (between
    messages); the Phase 06 chat text now wired into `run` adds real — though still modest —
    content bits when messages arrive (operator Stream-Health confirmation with chat content
    is pending).
- The v0.11.1 CBR/MP3 remedy note and the Ingest Probe Procedure above remain fully valid for
  CONNECTION-level issues (publish-succeeded-but-inactive/"No data" shapes and the probe
  discrimination path); this note covers only the content-level Stream-Health flag.

## Manual Verification Procedure (Operator-Run)

The canonical safe test plan (D-T4-8). It is performed by the human operator against a real YouTube account — the agent did not run, and never claims, this procedure; its result may be recorded only by the operator (e.g. in an issue/comment), never as build evidence.

**Operator status note (2026-10-14, user-supplied — never agent-observed):** the target channel is awaiting live-stream activation after live access was enabled (~24 h expected wait per the operator report; official guidance is that first-time enablement *"may take up to 24 hours"* — see [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md), Channel Live-Streaming Access). The REAL manual test below is ON HOLD until the channel has live-streaming access.

1. **Scope reminder (READ FIRST)**: this procedure is performed by the human operator against a real YouTube account. It is DISTINCT from every automated/container check in this work; the agent did not run, and never claims, this procedure; its result may be recorded only by the operator, e.g. in an issue/comment — never as build evidence.
2. **Privacy + secrets reminder**: keep OAuth tokens (`oauth.credentials_path`), the client secret (`oauth.client_config_path`), and any stream key/RTMPS URL you may inspect in YouTube Studio PRIVATE; never paste them into the repo, docs, issues, logs, or commits. Use a test broadcast with `privacy_status: "unlisted"` in `config/config.json`.
3. Prereq check: valid `config/config.json` (all six sections), FFmpeg installed and `ffmpeg.executable` correct, credentials authorized via the documented `auth` run.
4. Start: run the executable: `rust-youtube-streamer-service run`.
5. Expected startup output (order only, no secrets): config/authorization progress; one `broadcast prepared for streaming` line carrying the broadcast_id; the FFmpeg-started line; the `stream confirmed active on YouTube` line; the `public watch link for the live stream` line carrying the public watch URL (the watch page of the noted broadcast_id, non-secret; an unlisted broadcast is watchable by anyone holding the link). NOTE the exact broadcast_id shown — it identifies the resources in later steps.
6. Actual video arrival and live chat: open YouTube Studio → the broadcast enters live; verify real frames appear (continuous rendered video). Note: re-running `auth` is NOT required for chat (no scope change; `youtube.force-ssl` already covers the chat calls — run `auth` only per the existing recovery paths). Confirm video starts BEFORE any messages arrive (the stream opens black — pre-connection chat is history-discarded by design). Then, from a SEPARATE account, send ordinary text messages AFTER the stream is confirmed active; expect them to render as author/text on the watch page PROMPTLY: pushed messages are mapped the moment their batch arrives and appear on the NEXT frame render (sub-second at 30 FPS) after YouTube-side delivery — record the observed delay honestly; it includes YouTube-side latency the application does not control. Confirm the same messages appear in the chat log (`logs/chat.log`, or the configured `chat.log_file`). Exercise idle periods: a few minutes with no chat — frames continue, nothing duplicates, no history replays, and no reconnect churn occurs while the connection stays alive. Messages sent before the stream went live are discarded by design (the worker's FIRST baseline session is discarded whole — messages posted during that window are lost, the documented API-absence limitation). Confirm at least ~1–2 minutes of continuous video. Verify the broadcast is on the EXPECTED channel: one OAuth grant binds to one channel identity chosen at consent, and the application neither selects, surfaces, nor verifies which channel ([`docs/authentication.md`](authentication.md)) — if the broadcast appeared on a different channel, Ctrl+C-stop, re-run `auth` to select the intended channel, and repeat from step 4.
6b. OPTIONAL re-establishment check (interpreter-relevant drop discrimination): while the stream is live, briefly interrupt the network path (e.g. disconnect Wi-Fi or block outbound connectivity for roughly 10–15 seconds), then restore it. Expected: the video never stalls and the run keeps running; after restoration, NEW ordinary messages sent from the separate account appear on the watch page and in the chat log WITHOUT duplication (messages sent during the outage may be missing — the link was down; not everything pushed during a drop is recoverable). A prolonged outage can end the run with the single `live chat transport failed after repeated attempts:` error line and exit 1 after the bounded budget (10 consecutive transient failures, backoff capped at 60 s) — record whichever outcome occurs. A live re-establishment confirmation is the operator's evidence of the reconnect path the offline tests pin only structurally.
7. Ctrl+C shutdown: after confirming video, press Ctrl+C. Expected: frame loop stops; the stdout line `streaming stopped; the broadcast auto-completes on YouTube after ingestion ends`; process exit code 0.
8. Post-shutdown broadcast status: wait about 1–2 minutes (the `enableAutoStop` auto-complete window), then open YouTube Studio → confirm the broadcast identified in step 5 shows a completed/ended status (`complete`); record the observed transition moment. (This autoStop fact comes from the official Life-of-a-Broadcast documentation cited in the Phase-05 Task-1 plan — any deviation observed live is recorded honestly, never reconciled by editing the code.)
9. On any startup failure: capture the single redacted `error:` line, verify no token/key text appears in it, and use the failure/recovery tables (this document's "Broadcast/Stream Fate" section, [`docs/authentication.md`](authentication.md)) — again keeping secrets private.
10. Evidence-recording guidance for the operator: record the unlisted-broadcast creation, video arrival, Ctrl+C shutdown message, exit code, and post-shutdown status — each with timestamps — in whatever tracking artifact the operator prefers.

## Out of Scope

- Broadcast reuse, multi-broadcast support, or a web dashboard.
- Automated resource deletion or lifecycle transitions.
- Audio production or mixing (the MVP renders frames only; the v0.11.0 config-gated silent `anullsrc` audio track is an ingest-signal carrier shipped as the brief-section-10.4 explicit design change, not produced content).

## See Also

- [README](../README.md) — project overview and the Production Run section.
- [`docs/configuration.md`](configuration.md) — configuration fields, validation rules, and CLI exit codes.
- [`docs/authentication.md`](authentication.md) — the `auth` flow, credential files, unattended refresh, and the failure/recovery table.
- [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md) — the Google Cloud setup behind the prerequisites.
- [`docs/demo.md`](demo.md) — the local-only demo recording (no YouTube, no OAuth).

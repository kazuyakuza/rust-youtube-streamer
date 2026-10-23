# Production Run

Operator guide for the production `run` command: what one invocation does, what it needs, what happens at startup, what the logs show (without secrets), how shutdown and broadcast fate behave, and the manual verification procedure. This procedure is operator-run; it was NOT executed by the agent.

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [What Happens at Startup, in Order](#what-happens-at-startup-in-order)
- [What the Logs Look Like (Without Secrets)](#what-the-logs-look-like-without-secrets)
- [Shutdown Behavior](#shutdown-behavior)
- [Broadcast/Stream Fate (Policy and Honest Limitations)](#broadcaststream-fate-policy-and-honest-limitations)
- [Manual Verification Procedure (Operator-Run)](#manual-verification-procedure-operator-run)
- [Out of Scope](#out-of-scope)
- [See Also](#see-also)

## Overview

One `run` invocation:

- creates ONE new broadcast and ONE new live-stream resource from the validated `youtube.broadcast.*` configuration (title, description, privacy status) — there is no broadcast reuse and no multi-broadcast support;
- binds the two resources together, waits for the stream to become ready, and acquires the RTMPS ingestion destination from the API response;
- streams continuous rendered frames to YouTube Live over RTMPS through the configured external FFmpeg executable (`ffmpeg.executable`);
- writes no local file in stream mode (the local MP4 behavior belongs to the `demo` command only).

The streamed frame content contains NO chat text — Live Chat is Phase 06 (recorded on the TODO defect ruling); the frame loop is continuous and chat-free by design.

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
6. The broadcast resource is inserted (`liveBroadcasts.insert`, with `enableAutoStart=true`, `enableAutoStop=true`, and no monitor stream).
7. The broadcast is bound to the stream (`liveBroadcasts.bind`); the `boundStreamId` in the response is asserted.
8. A bounded readiness wait polls until the stream is ready — up to 180 seconds at a 5-second interval; the wait is cancellable by Ctrl+C.
9. The RTMPS ingestion destination is acquired from the API response; it is kept secret and never printed.
10. FFmpeg starts with the RTMPS-target argument set (FLV muxer, 2-second-target GOP derived from the configured FPS, no local output file). After FFmpeg has started, a post-start confirmation wait polls until YouTube reports `streamStatus == active` — up to 120 seconds at a 5-second interval, on a thread parallel with the frame loop (documented refinement of the startup sequence), then the continuous chat-free frame loop runs.

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
| 6 | (gate-timeout only) `confirmation gate timed out; last observed streamStatus` | info; the `stream_status` field is the raw documented `status.streamStatus` label from the ONE tolerated final lookup at failure time (not a per-attempt history) — appears instead of rows 4–5 when the post-start confirmation gate exhausts its budget |
| 6 | (gate-timeout, status unobservable) `confirmation gate timed out; the stream status could not be observed` | info; the tolerated final lookup yielded no label (the lookup itself failed, no stream was found, or the resource carries no label) |
| 7 | (end of run, clean exit) `FFmpeg streaming process exited cleanly (exit status 0)` | info; the `exit_code` field; the same event carries `captured diagnostics follow: <redacted stderr tail>`, or `captured diagnostics follow: none captured` when the tail is empty |
| 7 | (end of run, exit failure) `FFmpeg streaming process exited with a failure status` | **warn**; the `exit_code` field (`None` = signal death); the redacted tail follows in the same event — NEVER a second `error:` line (the single-`error:`-line contract holds) |

Ordering: the row-6 events fire only on the confirmation-gate timeout shutdown path, where the gate thread times out BEFORE the frame loop ends — the stop it requests ends the loop, so the row-7 exit evidence follows and both precede the single final `error:` line. Row 7 fires at the end of every run's frame loop: a normal Ctrl+C stop shows the clean-exit info line before the stdout stop line.

Standing secret guarantees: OAuth tokens, refresh tokens, client secrets, stream keys, full RTMPS URLs, and authorization codes NEVER appear in logs, error lines, or diagnostics. FFmpeg's own stderr echo of the output URL is replaced with `[redacted]` in captured diagnostics, and the stream key/RTMPS URL is never embedded in command logs, error text, or debug output.

Inspecting the logs: the `run` command writes no log file — every operational line is a `tracing` event on stderr. Run the executable in a terminal to read the lines live, or redirect stderr to a file for later inspection (the secret guarantees above keep captured lines safe to store). Verbosity follows the exact, case-sensitive `RUST_LOG` environment variable (default `info`); e.g. `RUST_LOG=debug rust-youtube-streamer-service run`. Full filter semantics: [README's Logging (RUST_LOG) guide](../README.md#logging-rust_log).

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
- Readiness timeouts — `YouTube readiness wait timed out (stage 'readiness')`, or the post-start confirmation timing out (stage `ingestion-confirmation`): both waits are bounded (180 s / 120 s at 5 s intervals); verify channel eligibility and API enablement, clean up leftover `ready` resources in YouTube Studio, then re-run. On an `ingestion-confirmation` timeout the run also logs the diagnostics evidence lines (table row 6 above); operator diagnosis procedure:

  1. `confirmation gate timed out; last observed streamStatus stream_status="ready"` — YouTube never registered the ingest: the encode side is healthy (see item 4), so investigate the YouTube side — check Studio (made-for-kids channel setting, stream routing, channel live access, `docs/google-cloud-prerequisites.md` eligibility) and delete the orphaned `ready` broadcast + liveStream pair (the "Broadcast/Stream Fate" section above) before re-running.
  2. `…last observed streamStatus` naming another label (`inactive`, `error`, `created`) — treat it per the documented `liveStreams` status semantics; the `error` label points at the stream's own diagnostics.
  3. `…the stream status could not be observed` — the tolerated final lookup itself failed (network/API): resolve connectivity, delete stale Studio resources, then re-run.
  4. The FFmpeg exit lines (table row 7 above) close the encode-side question: a clean exit (exit status 0) with a progress-line stderr tail means FFmpeg encoded and streamed frames for the full budget; a `warn` failure-status line names a broken-piped/encode process — its tail's last lines name the underlying cause, with the RTMPS URL pre-redacted.
  5. Studio cleanup reminder: each failed run leaves an orphaned `ready` pair per the fate policy — delete before re-running, or the account eventually hits `userBroadcastsExceedLimit`.
  6. Honest limits: the mid-loop broken-pipe shape (`writing a frame to the FFmpeg process failed: broken pipe …`) still cannot print the stderr tail (the drain threads are never joined on that path) — the single `error:` line remains the evidence surface there; and the logged streamStatus is the FINAL tolerated lookup's label at failure time, not a per-attempt history.

- Broadcast-state conflicts — `broadcast state forbids this operation (reason '...')`: typically stale resources from earlier failing runs; clean them up in YouTube Studio and re-run.
- Transient backend/network — `YouTube backend temporarily unavailable (reason '...')` or `unable to reach the YouTube API: ...`: resolve connectivity, then re-run.

## Manual Verification Procedure (Operator-Run)

The canonical safe test plan (D-T4-8). It is performed by the human operator against a real YouTube account — the agent did not run, and never claims, this procedure; its result may be recorded only by the operator (e.g. in an issue/comment), never as build evidence.

**Operator status note (2026-10-14, user-supplied — never agent-observed):** the target channel is awaiting live-stream activation after live access was enabled (~24 h expected wait per the operator report; official guidance is that first-time enablement *"may take up to 24 hours"* — see [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md), Channel Live-Streaming Access). The REAL manual test below is ON HOLD until the channel has live-streaming access.

1. **Scope reminder (READ FIRST)**: this procedure is performed by the human operator against a real YouTube account. It is DISTINCT from every automated/container check in this work; the agent did not run, and never claims, this procedure; its result may be recorded only by the operator, e.g. in an issue/comment — never as build evidence.
2. **Privacy + secrets reminder**: keep OAuth tokens (`oauth.credentials_path`), the client secret (`oauth.client_config_path`), and any stream key/RTMPS URL you may inspect in YouTube Studio PRIVATE; never paste them into the repo, docs, issues, logs, or commits. Use a test broadcast with `privacy_status: "unlisted"` in `config/config.json`.
3. Prereq check: valid `config/config.json` (all six sections), FFmpeg installed and `ffmpeg.executable` correct, credentials authorized via the documented `auth` run.
4. Start: run the executable: `rust-youtube-streamer-service run`.
5. Expected startup output (order only, no secrets): config/authorization progress; one `broadcast prepared for streaming` line carrying the broadcast_id; the FFmpeg-started line; the `stream confirmed active on YouTube` line; the `public watch link for the live stream` line carrying the public watch URL (the watch page of the noted broadcast_id, non-secret; an unlisted broadcast is watchable by anyone holding the link). NOTE the exact broadcast_id shown — it identifies the resources in later steps.
6. Actual video arrival: open YouTube Studio → the broadcast enters live; verify real frames appear (continuous rendered video; the Phase 05 stream has NO chat text — expected, chat is Phase 06). Confirm at least ~1–2 minutes of continuous video. Verify the broadcast is on the EXPECTED channel: one OAuth grant binds to one channel identity chosen at consent, and the application neither selects, surfaces, nor verifies which channel ([`docs/authentication.md`](authentication.md)) — if the broadcast appeared on a different channel, Ctrl+C-stop, re-run `auth` to select the intended channel, and repeat from step 4.
7. Ctrl+C shutdown: after confirming video, press Ctrl+C. Expected: frame loop stops; the stdout line `streaming stopped; the broadcast auto-completes on YouTube after ingestion ends`; process exit code 0.
8. Post-shutdown broadcast status: wait about 1–2 minutes (the `enableAutoStop` auto-complete window), then open YouTube Studio → confirm the broadcast identified in step 5 shows a completed/ended status (`complete`); record the observed transition moment. (This autoStop fact comes from the official Life-of-a-Broadcast documentation cited in the Phase-05 Task-1 plan — any deviation observed live is recorded honestly, never reconciled by editing the code.)
9. On any startup failure: capture the single redacted `error:` line, verify no token/key text appears in it, and use the failure/recovery tables (this document's "Broadcast/Stream Fate" section, [`docs/authentication.md`](authentication.md)) — again keeping secrets private.
10. Evidence-recording guidance for the operator: record the unlisted-broadcast creation, video arrival, Ctrl+C shutdown message, exit code, and post-shutdown status — each with timestamps — in whatever tracking artifact the operator prefers.

## Out of Scope

- Live Chat retrieval, message ingestion, chat logging, or chat rendering into the video (Phase 06).
- Broadcast reuse, multi-broadcast support, or a web dashboard.
- Automated resource deletion or lifecycle transitions.
- Audio.

## See Also

- [README](../README.md) — project overview and the Production Run section.
- [`docs/configuration.md`](configuration.md) — configuration fields, validation rules, and CLI exit codes.
- [`docs/authentication.md`](authentication.md) — the `auth` flow, credential files, unattended refresh, and the failure/recovery table.
- [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md) — the Google Cloud setup behind the prerequisites.
- [`docs/demo.md`](demo.md) — the local-only demo recording (no YouTube, no OAuth).

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

Standing secret guarantees: OAuth tokens, refresh tokens, client secrets, stream keys, full RTMPS URLs, and authorization codes NEVER appear in logs, error lines, or diagnostics. FFmpeg's own stderr echo of the output URL is replaced with `[redacted]` in captured diagnostics, and the stream key/RTMPS URL is never embedded in command logs, error text, or debug output.

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
- Failure recovery pointers: OAuth problems → [`docs/authentication.md`](authentication.md) (recovery table); Google Cloud/API-quota topics → [`docs/google-cloud-prerequisites.md`](google-cloud-prerequisites.md); any still-live broadcast after a crash → this document, §7 (there is no separate streaming-recovery document — the recovery content lives here).

## Manual Verification Procedure (Operator-Run)

The canonical safe test plan (D-T4-8). It is performed by the human operator against a real YouTube account — the agent did not run, and never claims, this procedure; its result may be recorded only by the operator (e.g. in an issue/comment), never as build evidence.

1. **Scope reminder (READ FIRST)**: this procedure is performed by the human operator against a real YouTube account. It is DISTINCT from every automated/container check in this work; the agent did not run, and never claims, this procedure; its result may be recorded only by the operator, e.g. in an issue/comment — never as build evidence.
2. **Privacy + secrets reminder**: keep OAuth tokens (`oauth.credentials_path`), the client secret (`oauth.client_config_path`), and any stream key/RTMPS URL you may inspect in YouTube Studio PRIVATE; never paste them into the repo, docs, issues, logs, or commits. Use a test broadcast with `privacy_status: "unlisted"` in `config/config.json`.
3. Prereq check: valid `config/config.json` (all six sections), FFmpeg installed and `ffmpeg.executable` correct, credentials authorized via the documented `auth` run.
4. Start: run the executable: `rust-youtube-streamer-service run`.
5. Expected startup output (order only, no secrets): config/authorization progress; one `broadcast prepared for streaming` line carrying the broadcast_id; the FFmpeg-started line; the `stream confirmed active on YouTube` line. NOTE the exact broadcast_id shown — it identifies the resources in later steps.
6. Actual video arrival: open YouTube Studio → the broadcast enters live; verify real frames appear (continuous rendered video; the Phase 05 stream has NO chat text — expected, chat is Phase 06). Confirm at least ~1–2 minutes of continuous video.
7. Ctrl+C shutdown: after confirming video, press Ctrl+C. Expected: frame loop stops; the stdout line `streaming stopped; the broadcast auto-completes on YouTube after ingestion ends`; process exit code 0.
8. Post-shutdown broadcast status: wait about 1–2 minutes (the `enableAutoStop` auto-complete window), then open YouTube Studio → confirm the broadcast identified in step 5 shows a completed/ended status (`complete`); record the observed transition moment. (This autoStop fact comes from the official Life-of-a-Broadcast documentation cited in the Phase-05 Task-1 plan — any deviation observed live is recorded honestly, never reconciled by editing the code.)
9. On any startup failure: capture the single redacted `error:` line, verify no token/key text appears in it, and use the failure/recovery tables (this document §7, [`docs/authentication.md`](authentication.md)) — again keeping secrets private.
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

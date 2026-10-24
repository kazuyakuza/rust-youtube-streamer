# Global Plan — Ingest "No data": Discriminate Cause; Conditional Silent-Audio Remedy

Plan date: 2026-10-24 · Owner: Planner Agent (Critical Workflow)
TODO: `.agent/todos/20261024/20261024-todo-1.md` · Type: FIX+CONDITIONAL-FEATURE (semver MINOR 0.10.0 → 0.11.0) · Front-end: NO

## Global Pre-Analysis

Evidence from the third native Windows run (`logs/run/20261024-160137.log`, v0.10.0 — the v0.10.0
diagnostics delivered as designed):

1. Startup is fully healthy: credentials (refreshed + verified) → `broadcast prepared for streaming broadcast_id="gHSIW9pN5xc"` (O1+O2+O3 bind + readiness wait: the stream was observed `ready`) → `FFmpeg started`.
2. FFmpeg streamed the FULL post-start budget: `frame= 3801` ≈ 126 s at 30 fps, `speed=0.999x`, and **exited 0** after the shutdown EOF — the RTMPS publish succeeded at the protocol level (any connect/publish failure exits non-zero and would surface as a pipeline error).
3. YouTube's side: the gate's final tolerated lookup observed `stream_status="inactive"` — the stream LEFT `ready` during the run (YouTube's edge noticed the connection) but never reached `active`; Studio's "No data … ensure … correct stream key" matches.
4. The encoded signal is degenerate: `video:288KiB audio:0KiB` → ~18.55 kbit/s total against `-b:v 6000k` — a pure-black source cannot be padded to the ABR target (x264 hit its QP floor: `Avg QP: 0.08`, `final ratefactor: -48.17`); FLV mux overhead 25.9%; the "Failed to update header" warnings are the standard non-seekable-output notices (harmless over RTMP).
5. Video-only ingest acceptability was EXPLICITLY left unverified by brief §10.4 — and our stream is video-only.

Candidate causes (ranked): **C1** the ingest requires/expects an audio track for the signal lock; **C2** the near-zero-bitrate all-black signal fails the lock; (a connection/URL-level cause is now WEAK: the publish completed cleanly).

Operator facts (user-supplied, 2026-10-24, never agent-observed): the `made_for_kids` config option worked correctly; every orphaned broadcast/stream pair from the failed runs — including this one (gHSIW9pN5xc) — was deleted in YouTube Studio (no accumulation).

## Encoded Technical Direction

- Task 1 = research + operator probe procedure, NO app behavior change. The probe procedure is the fastest honest discriminator: three manual FFmpeg commands the operator runs with their own fresh stream key (the app never prints keys; the operator pastes the key only into the local command): (i) rich source video-only (`testsrc2` at the locked streaming vector's parameters: 1080p30, veryfast, 6000k, g 60, yuv420p, flv) → if `active`/live: C2 (content-level); (ii) black source + silent AAC audio (`anullsrc` + `-shortest`, same vector + audio) → if active: C1 (audio-level); (iii) rich + audio control → isolates environment vs signal; if even (iii) fails: connection/URL-level (then revisit the weak C3). Every probe uses the SAME rtmpsIngestionAddress/streamName shape the app assembles (`address/streamName`).
- Task 2 = CONDITIONAL silent-audio remedy, gated on (a) official research OR (b) the operator's probe evidence; SKIP is a first-class outcome (recorded findings + next step). If implemented: config `ffmpeg.audio.enabled` (Option<bool>, absent/false = byte-identical video-only wire form); the streaming vector gains a SECOND input `-f lavfi -i anullsrc=channel_layout=stereo:sample_rate=48000` + `-c:a aac -b:a 128k` + `-shortest` (named constants, documented rationale; `-shortest` guarantees the stdin-EOF shutdown stays clean because the infinite anullsrc is cut when the video ends); the DEMO/local vector and the entire demo command stay byte-stable; no transport/lifecycle/redaction/exit-code/shutdown restructuring; the Task-1 diagnostics stay byte-stable.
- Brief §10.4 sanction: audio may be implemented ONLY as an explicit design change when ingestion requirements make it necessary — the plan approval below IS that user ruling.
- Version: MINOR 0.10.0 → 0.11.0 at workflow step 3 (conditional-task precedent: the bump lands regardless of the Task-2 verdict).

## Execution Map (steps 2–6; each task gets its own 4.1–4.6 cycle)

- Step 2: Git Feature Branch Setup => implementer — branch `fix/ingest-no-data` from `main` (clean, at `fe97b25`, synced with origin)
- Step 3: Version Update => implementer — 0.10.0 → 0.11.0, commit `chore: bump version to 0.11.0`, lockfile refresh verified `--locked`
- Task 1: 4.1b Analysis & Implementation Plan => architector (research citations + the docs edit spec; no front-end → no 4.1a)
- Task 1: 4.2 Implementation => implementer (docs + research record only)
- Task 1: 4.3 Code Review & Simplification => code-reviewer + code-simplifier; 4.3-fix => implementer
- Task 1: 4.4 Documentation => docs-specialist
- Task 1: 4.5b Overall Plan Adherence => architector
- Task 1: 4.6 Task Completion => implementer
- Task 2: 4.1b Analysis & Implementation Plan => architector (research gate verdict; implement-or-skip plan)
- Task 2: 4.2 Implementation => implementer (implement path OR skip-record path per the verdict)
- Task 2: 4.3 => code-reviewer + code-simplifier; 4.3-fix => implementer
- Task 2: 4.4 => docs-specialist
- Task 2: 4.5b => architector
- Task 2: 4.6 => implementer
- Step 5: TODO File Completion => implementer — `-DONE` rename, tmp hygiene, merge to `main`, push `origin` ONLY

## Risks and Mitigations

- Research may be inconclusive on video-only acceptance (YouTube's official docs may not state a hard audio requirement): then Task 2's gate needs the OPERATOR probe results — the workflow pauses at Task 2's 4.1b and asks the user to run Task 1's procedure and report the outcomes (operator-side evidence, never agent-run).
- If the cause turns out environment-level (probe iii fails): NO app change is warranted; the TODO closes with documented findings + operator environment guidance (firewall/URL/encoder-build checks) and the failure-accounting rule.
- `-shortest` subtleties: if the architector's research shows `-shortest` interacts badly with live FLV on some FFmpeg builds, the fallback is `-fflags +shortest -max_interleave_delta`-style variants — ONE mechanism must be picked and loopback-pinned, never shipped unverified.
- Each further failed run creates another orphaned pair (the operator has been deleting them — record this practice as the documented procedure).

# Global Plan — Stop-Path Fix (Ctrl+C Isolation) + Stream-Health Bitrate Remedies

Plan date: 2026-10-25 · Owner: Planner Agent (Critical Workflow)
TODO: `.agent/todos/20261024/20261024-todo-2.md` · Type: FIX (semver PATCH 0.11.0 → 0.11.1) · Front-end: NO

## Global Pre-Analysis

Fourth operator run (`logs/run/20261025-020431.log`, v0.11.0 — operator-supplied fact): the first
fully successful live broadcast (confirmed active + watch link at ~10 s; auto-complete after close
worked). Two defects remain, both now evidenced:

1. **Stop-path**: `Exiting normally, received signal 2` + `exit_code=Some(255)` — the console's
   Ctrl+C reached the FFmpeg child (bare spawn, no process-group isolation in
   `src/streaming/spawn.rs`), so FFmpeg self-terminated via its SIGINT path (exit 255) instead of
   finalizing on our stdin EOF (exit 0). `finish()` then correctly reported `ExitFailure` and the
   run exited 1 on a USER stop — violating the clean-stop contract (exit 0 + stop message). The
   v0.10.0 diagnostics machinery is exactly what exposed this.
2. **Health warnings**: video ~18.5 kbit/s vs the 6000k target (black cannot pad an ABR stream;
   YouTube recommends CBR — x264's `nal-hrd=cbr` filler is the documented CBR mechanism); audio
   actual ~2 kbit/s (AAC on silence encodes ~zero bits; MP3 CBR pads silence to the real bitrate;
   MP3 is an officially accepted YouTube audio codec per the Task-1 research of the previous TODO).

## Encoded Technical Direction (Task 1 — stop-path isolation)

- `src/streaming/spawn.rs::prepare_piped_command` gains platform-gated isolation via small helper fns:
  - `#[cfg(target_family = "windows")]`: `use std::os::windows::process::CommandExt;` + `command.creation_flags(CREATE_NEW_PROCESS_GROUP)` (const 0x00000200, named constant with a doc comment: processes in a new process group have Ctrl+C disabled by default — the console event no longer reaches the child).
  - `#[cfg(target_family = "unix")]`: `use std::os::unix::process::CommandExt;` + `command.process_group(0)` (stable since Rust 1.64 ≥ the pinned 1.82 toolchain; the child leaves the terminal's foreground process group, so the terminal's SIGINT targets the app only).
  - One cfg-gated helper per family (≤2 args, single-section bodies) called from `prepare_piped_command` — NO generic abstraction, no new dependency.
- Resulting contract (expected + test-pinned where container-feasible): Ctrl+C → the app's handler flips the flag → the loop breaks → stdin EOF → FFmpeg finalizes (exit 0) → `primary_shutdown_outcome(Ok(()), Confirmed/Cancelled)` → exit 0 + `RUN_STOP_MESSAGE`. The GateFinish matrix is unchanged.
- Offline pin (container/Linux, fake FFmpeg): spawn the fake child through the production spawn path, deliver SIGINT to the PARENT's process group (`libc::kill(0, SIGINT)`-shaped — the test process's own group), assert the child SURVIVES (isolation proof), then EOF-finalize and assert exit 0. Naming per the repo's D7 style. Windows-side: compile-checked by the cross-build; documented container-untestable.
- The demo mode shares `spawn_with` → inherits isolation; demo exit codes byte-stable (Interrupt → 1); `auth`/YouTube untouched.

## Encoded Technical Direction (Task 2 — streaming-vector remedies)

- In `src/streaming/stream_args.rs`, extend the audio-ON streaming vector (and ONLY it):
  - Video CBR (derived from the configured bitrate B, no new config fields): `-maxrate <B> -minrate <B> -bufsize <2×B>` + `-x264-params nal-hrd=cbr` (the filler mechanism; YouTube's own CBR recommendation; BUF_SIZE_MULTIPLIER named constant).
  - Audio: `-c:a libmp3lame -b:a 128k` replaces the aac pair (MP3 CBR pads silence to the real 128k; official YouTube codec; document the libmp3lame operator prerequisite — a build lacking it already fails with a clear typed spawn error).
  - The audio-OFF vector: decide ONE shape — the simplest honest rule: the audio-OFF vector ALSO gains the CBR video parameters (they are video-side; the warnings were health-metric-wide) while keeping NO audio track; encode this explicitly and pin both vectors exactly.
- Real-container pin: encode to a LOCAL FLV with the production args (loopback-style local output for measurement only, never Google) and ffprobe the measured total bitrate ≈ the configured target (record the tolerance honestly; the CBR filler makes it near-exact).
- Byte-stability: the demo/local vector + demo command + all YouTube/OAuth/diagnostic surfaces untouched; the Task-1 diagnostics warn/info lines unchanged.

## Execution Map (steps 2–6; each task gets its own 4.1–4.6 cycle)

- Step 2: Git Feature Branch Setup => implementer — branch `fix/stop-path-and-stream-health` from `main` (clean, at `964f817`, synced with origin)
- Step 3: Version Update => implementer — 0.11.0 → 0.11.1, commit `chore: bump version to 0.11.1`, lockfile refresh verified `--locked`
- Task 1: 4.1b => architector · 4.2 => implementer · 4.3 => code-reviewer + code-simplifier; 4.3-fix => implementer · 4.4 => docs-specialist · 4.5b => architector · 4.6 => implementer
- Task 2: 4.1b => architector · 4.2 => implementer · 4.3 => code-reviewer + code-simplifier; 4.3-fix => implementer · 4.4 => docs-specialist · 4.5b => architector · 4.6 => implementer
- Step 5: TODO File Completion => implementer — `-DONE` rename, tmp hygiene, merge to `main`, push `origin` ONLY

## Risks and Mitigations

- `nal-hrd=cbr` × FLV muxer compatibility: verify via the real-container pin (encode a local FLV with the production args); if the muxer/HRD combination fails, the encoded fallback decision procedure (like the 411 mechanism order) picks among: `nal-hrd=cbr:force-cfr=1` variants → `-fflags +bitexact`? NO — only documented CBR mechanisms; if none works with FLV, STOP and report to the planner (the audio remedy alone may still ship).
- MP3-in-FLV legality: official FFmpeg docs list MP3 as a supported FLV audio codec; the real-container pin verifies end-to-end.
- POSIX `process_group(0)`: if the container's test proves flaky (reaping races), the fallback is killing-then-reaping patterns already present in `FfmpegProcess::drop` — the pin decides; never ship unverified.
- The operator's FFmpeg build must include libmp3lame (gyan full builds do) — documented; a lacking build fails with the existing clear typed error.
- Console-close (not Ctrl+C) still kills both processes abruptly — documented limitation; never claim otherwise.

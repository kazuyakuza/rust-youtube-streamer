# Local Demo Recording

A local-only demonstration recording: predefined chat messages rendered into an MP4 by the shared chat → renderer → FFmpeg pipeline, with no YouTube contact of any kind.

## Table of Contents

- [What the Demo Does](#what-the-demo-does)
- [Prerequisites](#prerequisites)
- [Running the Demo](#running-the-demo)
- [Output](#output)
- [Ctrl+C and Shutdown](#ctrlc-and-shutdown)
- [What the Demo Does Not Do](#what-the-demo-does-not-do)
- [Verification Evidence and Limits](#verification-evidence-and-limits)
- [See Also](#see-also)

## What the Demo Does

- Runs a fixed, documented message timeline: **28 predefined messages** (`demo-NN` ids, authors cycling `alice`/`bob`/`charlie`/`dana`, text `demo message NN`).
- The timeline leads with a **30-frame black lead-in**, admits **at most one message every 15 frames**, and holds a **60-frame tail** after the last delivery — `30 + 27×15 + 60 = 495`, so the last rendered frame index is 495: **496 rendered frames ≈ 16.5 s at the example 30 FPS config**.
- At the example geometry the visible capacity is **26 lines**, so the oldest message is evicted (bounded FIFO) starting at message 27 — eviction is exercised by the shipped demo itself.
- Message ids `demo-NN` never appear on screen (`Display` is exactly `author: text`); message 15 is a long line that exercises the renderer's horizontal clipping path.
- Every frame is a complete RGB24 frame (`width × height × 3` bytes) piped to FFmpeg's stdin at the configured FPS; empty intervals render the current visible text (the black background before the first message).
- Finishes automatically after the timeline, closes stdin (orderly EOF), waits for FFmpeg, verifies the MP4 is finalized, and only then reports success.

## Prerequisites

- A working configuration file: copy `config/config.example.json` to `config/config.json` (or pass `--config <path>`).
- **FFmpeg is an external prerequisite** — install it separately; this project never installs or bundles it. Set `ffmpeg.executable` to an absolute path (e.g. `/usr/bin/ffmpeg` on Linux, `C:\ffmpeg\bin\ffmpeg.exe` on Windows); a bare `ffmpeg` resolved via `PATH` also works. Launch failures surface as a typed `unable to start the FFmpeg executable '<path>': …; check that the configured path is correct and FFmpeg is installed and runnable` error.
- A parseable font at `renderer.font`. The committed example points at `fonts/console.ttf`, whose provenance is indeterminate — prefer a documented font such as the OFL-licensed `fonts/Exo2-Light.otf` (license text in `fonts/OFL.txt`, provenance and SHA-256 in `fonts/README.md`), which the test suite uses.

## Running the Demo

```
rust-youtube-streamer-service demo
```

Configuration is loaded and validated first (same startup funnel as every command). Example success output — one stdout line after finalization, with the whitelisted `info` tracing events on stderr:

```
demo recording finalized: /path/to/videos/demo-20261012-040827.mp4
```

```
INFO rust_youtube_streamer_service::app: mode selected command="demo" config_path=config/config.json
INFO rust_youtube_streamer_service::app: configuration loaded and validated
INFO rust_youtube_streamer_service::app: mode finished command="demo" exit_code=0
```

Exit codes:

- `0` — success: the MP4 exists, is non-empty, and FFmpeg exited 0 before the path was printed.
- `1` — any demo failure, reported as exactly one `error:` line on stderr. `DemoError` delegates the inner cause verbatim: interrupt-handler installation, chat message stage, rendering, frame-size guards, or the FFmpeg spawn/write/wait/exit/finalization stages.
- `3` — **never used by `demo`** (it belongs to the `run` placeholder only).

## Output

- Files land in `videos/` **beside the executable**, never in the launch working directory; the directory is created on demand.
- Filenames are `demo-YYYYMMDD-HHMMSS.mp4`; a same-second collision walks deterministic suffixes (`_2`, `_3`, … up to `_1000`). Nothing is ever overwritten — a taken name is skipped, not replaced.
- Generated MP4s are gitignored by the `videos/` rule and are never committed.

## Ctrl+C and Shutdown

Ctrl+C is a cooperative shutdown **request** with documented limitations (source: `src/app/pipeline/interrupt.rs` module docs):

1. Installing the handler **replaces** default Ctrl+C termination: the handler only flips a shared stop flag, repeated Ctrl+C is inert, and if the pipeline is stuck in a long blocking operation (a stdin write to a hung FFmpeg, or the unbounded `finish()` wait) the operator must terminate the process with OS tools.
2. Ctrl+C during a pacing sleep is observed after the current sleep returns — shutdown latency is at most one frame interval.
3. Ctrl+C while rendering/writing a frame is observed at the next iteration top — the current frame completes, then shutdown (at most one extra frame).
4. In Docker/Linux the foreground process receives SIGINT normally; detached background processes have no console signal — no claims are made about daemonized execution.
5. The FFmpeg exit wait in `finish()` is intentionally unbounded in this phase (no timeout configuration field; hardening deferred to a later phase).

## What the Demo Does Not Do

- No YouTube contact: no OAuth, no broadcast or stream creation, no RTMPS, no live chat connection.
- No stdin message entry, interactive menu, hotkeys, or keyboard controls (deferred to a later phase).
- No screenshots; no audio (neither generated nor silent).
- Not a substitute for `run`: it never pretends to stream to YouTube.

## Verification Evidence and Limits

- Verified via the standard container gate (`docker compose … run --rm rust sh scripts/dev-checks.sh`, exit 0, `ALL CHECKS PASSED`, **147 tests**), which includes fake-FFmpeg process tests (POSIX-script stand-ins, no system FFmpeg needed) and a real-FFmpeg smoke test that records a 64×36@5fps clip and probes it with `ffprobe` (width, height, frame rate, duration).
- FFmpeg arguments and finalization were verified against the dev image's OBSERVED `ffmpeg version 5.1.9-0+deb12u1`; a once-per-phase VM demo smoke recorded a 1920×1080 `r_frame_rate=30/1` MP4 of `duration=16.534000` s and exit code 0.
- Native Windows/Linux runtime execution of the demo: **NOT RUN** — all verification is container-only; cross-compiling the Windows `.exe` is not native runtime validation.

## See Also

- [README](../README.md) — project overview and the [Local Demo Recording](../README.md#local-demo-recording) quick start.
- [Configuration](configuration.md) — field reference, CLI usage, and exit codes.

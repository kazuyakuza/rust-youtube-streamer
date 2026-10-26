# Configuration

The service loads a JSON configuration file at startup; two live paths exist — the committed example template and the operator-owned real file.

## Table of Contents

- [Configuration File Locations](#configuration-file-locations)
- [Getting Started (Create the Real Configuration)](#getting-started-create-the-real-configuration)
- [Field Reference](#field-reference)
- [Validation Rules](#validation-rules)
- [Command-Line Usage and Exit Codes](#command-line-usage-and-exit-codes)
- [Logging (RUST_LOG)](#logging-rust_log)
- [See Also](#see-also)

## Configuration File Locations

- Default: `config/config.json`, relative to the directory the executable is launched from; overridden by `--config <path>`.
- `config/config.json` is gitignored — never commit it, and do not rely on commit review to protect anything you put in it.
- `config/config.example.json` is committed as a safe, non-secret starting template; its example `privacy_status` is `unlisted`.
- The example is never loaded implicitly: when `--config` is absent, the executable always resolves to `config/config.json`.
- Only one configuration file is used per run: the default path or a single `--config <path>` value.

## Getting Started (Create the Real Configuration)

1. Copy `config/config.example.json` to `config/config.json`.
2. Edit `config/config.json` for your environment (see the field reference below).
3. Run the executable against the new file. `--config <path>` works before or after the command:

```
rust-youtube-streamer-service run
rust-youtube-streamer-service auth --config path/to/config.json
rust-youtube-streamer-service --config path/to/config.json run
rust-youtube-streamer-service --help
```

The example contains no secrets today, but treat the real `config/config.json` as private and keep it out of version control.

## Field Reference

All six top-level sections are required. Each field below lists its rule and the value from `config/config.example.json`. Section and field names must match exactly as written.

### youtube.broadcast

- `title` (string) — broadcast title; must be non-blank. Example: `Rust YouTube Streamer Prototype`.
- `description` (string) — broadcast description; must be non-blank. Example: `YouTube Live streaming prototype`.
- `privacy_status` (string) — one of `private`, `public`, `unlisted` (case-sensitive). Example: `unlisted`.
- `made_for_kids` (boolean, optional) — when present, the value is sent as `status.selfDeclaredMadeForKids` on the `liveBroadcasts.insert` request (a documented insert-settable property in the official liveBroadcasts reference): `true` declares the broadcast child-directed, `false` declares it not child-directed. When ABSENT the field is never sent and YouTube applies its own default handling — in practice the channel-level made-for-kids setting (YouTube Studio → Settings → Channel → Advanced section). Any non-boolean value is rejected at deserialize time. Example: `false`.

### oauth

- `client_config_path` (string) — OAuth client-configuration file path (the Google-issued "Desktop app" client JSON); must be non-blank. Default/example: `credentials/client_secret.json`.
- `credentials_path` (string) — issued-credentials file path (where `auth` saves tokens); must be non-blank. Default/example: `credentials/tokens.json`.
- Distinctness rule: the two paths must differ — validation rejects identical values, because the credential file would overwrite the client configuration. Both files are operator-owned; `credentials/*` is gitignored (only `.gitkeep` is tracked) and the real `config/config.json` is gitignored too. First-authorization procedure and the full credential contract: [`docs/authentication.md#first-authorization-interactive-once`](authentication.md#first-authorization-interactive-once).

### video

- `width` (integer) — frame width; must be greater than 0. Example: `1920`.
- `height` (integer) — frame height; must be greater than 0. Example: `1080`.
- `fps` (integer) — frames per second; must be greater than 0. Example: `30`.
- `pixel_format` (string) — must be exactly `rgb24`; any other value is rejected, never substituted. Example: `rgb24`.

### renderer

- `font` (string) — font file path; must be non-blank. Example: `fonts/console.ttf`. Runtime font must be a parseable font file; the renderer validates it at construction (typed `FontOpen`/`FontParse` errors). Test fixture `fonts/Exo2-Light.otf` ships under the SIL Open Font License 1.1 (`fonts/OFL.txt`); tracked `fonts/console.ttf` predates the renderer, has indeterminate provenance, and is referenced by no test — operators should point `font` at a documented, licensed font file.
- `font_size` (integer) — must be greater than 0. Example: `32`.
- `line_height` (integer) — must be greater than 0. Example: `40`.
- `left_margin`, `top_margin`, `right_margin`, `bottom_margin` (integers) — each must be 0 or greater. Example: `20` each.
- `text_color` (string) — must be non-blank. Example: `#FFFFFF`.
- `background_color` (string) — must be non-blank. Example: `#000000`.
- Vertical layout rule: `top_margin + bottom_margin + line_height` must not exceed `video.height`, so at least one rendered line fits.

### chat

- `log_file` (string) — chat log path; must be non-blank. Example: `logs/chat.log`.

### ffmpeg

- `executable` (string) — FFmpeg executable name or path; must be non-blank. Example: `ffmpeg`. Used by the `demo` and `run` commands: the process is spawned directly at this path (no shell); launch failures surface as `unable to start the FFmpeg executable '<path>': …; check that the configured path is correct and FFmpeg is installed and runnable`.
- `video_codec` (string) — must be non-blank. Example: `libx264`.
- `preset` (string) — must be non-blank. Example: `veryfast`.
- `bitrate` (string) — must be non-blank. Example: `6000k`. The streaming vector derives its CBR parameters from this value: `-maxrate <bitrate> -minrate <bitrate> -bufsize <2×bitrate> -x264-params nal-hrd=cbr` (the x264 constant-bit-rate filler; YouTube's own encoder-settings guidance recommends CBR).
- `audio` (object, optional) — silent-audio configuration for the production `run` streaming vector only (`demo` ignores it). Contains one field:
  - `enabled` (boolean, optional) — when `true`, the streaming FFmpeg command gains a silent audio track: a second `lavfi` input (`anullsrc=channel_layout=stereo:sample_rate=48000`) encoded by `libmp3lame` (the officially accepted alternative audio codec) at 128 kbit/s with an output `-shortest` bound ("Finish encoding when the shortest output stream ends.") so stdin-EOF shutdown stays clean and bounded. The codec/bitrate pair sits inside the official encoder-settings recommendation ("AAC or MP3"; 128-Kbps for stereo) and answers the documented ingest expectation that the stream "must contain exactly one audio stream" (`noAudioStream` health issue, liveStreams reference); MP3-CBR pads silence to the real bitrate where the AAC-on-silence shape cannot be padded. When the `audio` section is absent, or `enabled` is absent or `false`, the streaming vector carries the same derived CBR video parameters (video-side health remedy applied to both audio shapes); configuration files written for versions before 0.11.0 are unchanged and keep parsing (no validation-rule change; wrong boolean types are rejected at deserialize time; the rule count stays 21). The operator's FFmpeg build must include libmp3lame (the standard full builds do); a build lacking it fails at stream start with the clear spawn-error path.

Existence of the font file or the FFmpeg executable is checked by the consuming component, not at configuration load: the renderer opens and parses the configured font at construction with typed errors (no file-existence or parseable-font check at configuration load — this follows the Phase 01 validation precedent). The renderer and the FFmpeg process are wired into the runtime for local recording (`demo`) and the production RTMPS stream (`run`); launch-process failures surface the same typed error text for both modes. FFmpeg itself is a separately provided external prerequisite — the project never installs or bundles it, and both commands launch the configured executable (a missing or unrunnable path surfaces as `unable to start the FFmpeg executable '<path>': …; check that the configured path is correct and FFmpeg is installed and runnable`).

## Validation Rules

- Unknown fields are rejected at deserialize time.
- Wrong value types are rejected at deserialize time.
- Semantic checks (21 rules; all violations are collected and reported together):
  - `video.width`, `video.height`, `video.fps`, `renderer.font_size`, and `renderer.line_height` must be greater than 0.
  - `video.pixel_format` must be exactly `rgb24`.
  - Renderer margins must be 0 or greater, and `top_margin + bottom_margin + line_height` must not exceed `video.height`.
  - `privacy_status` must be one of `private`, `public`, `unlisted` (case-sensitive).
  - Required strings must be non-blank: broadcast `title` and `description`, `renderer.font`, `renderer.text_color`, `renderer.background_color`, `chat.log_file`, and all four `ffmpeg` fields.
  - `oauth.client_config_path` and `oauth.credentials_path` must be non-blank, and the two paths must be distinct (the credential file must not overwrite the client configuration).
- On failure (including a missing or unreadable configuration file): one actionable error names the configuration file and field, and the process exits with code 1.

## Command-Line Usage and Exit Codes

```
rust-youtube-streamer-service [--config <path>] <command>
rust-youtube-streamer-service auth|demo|run [--config <path>]
rust-youtube-streamer-service --help
```

- All commands load and validate the configuration before running.
- `auth` — performs the OAuth 2.0 installed-application authorization flow using the client file at `oauth.client_config_path`: prints the authorization URL, waits for the loopback callback, exchanges the code (PKCE S256), verifies the required YouTube scope, and saves the credentials atomically to `oauth.credentials_path`. Success exits 0 with one stdout success line; any failure (including configuration errors) exits 1 with exactly one redacted `error:` line on stderr — never 3. Full guide: [`docs/authentication.md`](authentication.md).
- `run` — the implemented production streaming mode: after configuration loads and validates, it consumes the Phase-04 credential service non-interactively (load → verify-first probe → refresh-once if rejected → atomic persist → re-verify), gates the required scope BEFORE any resource creation, creates ONE new broadcast and ONE new live-stream resource from the configured `youtube.broadcast.*` metadata/privacy, binds them, waits for readiness with bounded, cancellable waits (180 s at a 5 s interval; a post-start confirmation waits up to 120 s at a 5 s interval for `streamStatus == active`), acquires the RTMPS ingestion destination (never logged), and streams continuous rendered frames — with incoming ordinary Live Chat messages rendered as author/text when present (Phase 06 chat worker wired into `run`; details in [`docs/production-run.md`](production-run.md)) — through the configured external FFmpeg executable. Shutdown is ordered (stop flag + wait cancellation + FFmpeg stdin EOF + wait): a clean Ctrl+C stop and a pre-streaming Ctrl+C stop both exit 0; any startup or runtime fatal failure exits 1 with exactly one redacted `error:` line on stderr. Full operator guide: [`docs/production-run.md`](production-run.md).
- `demo` — runs the local-only FFmpeg demonstration recording. Config validated first through the same funnel; success exits 0 and prints `demo recording finalized: <path>` on stdout **after** the MP4 is finalized; any failure exits 1 with exactly one `error:` line on stderr (`DemoError` delegates the inner cause verbatim: interrupt-handler installation, chat message stage, rendering, frame-size guards, FFmpeg spawn/write/wait/exit/finalization). `demo` never exits 3 and never contacts YouTube, OAuth, broadcasts, or chat. Guide: [`docs/demo.md`](demo.md).
- `--config <path>` — configuration file to load; accepted before or after the command, at most once. Absent means the default `config/config.json`.
- `--help` — accepted only before the command; prints usage to stdout and exits 0.
- Exit codes: 0 = `--help`, a successful demo recording, a verified `auth` success, a clean `run` stop (Ctrl+C during streaming), or a pre-streaming `run` stop; 1 = configuration load/parse/validation or logging-init startup failure, demo-origin failures, any `auth` failure (the startup-failure class), and any run-origin fatal failure (startup or runtime); 2 = usage error (missing or unknown command/option, repeated `--config`, missing `--config` value, `--help` after the command, non-UTF-8 argument). Exit code 3 no longer exists in the CLI (the `run` placeholder was retired).
- The example template ships an `oauth` section (see above) with the two default paths; it contains no secrets and is safe to commit.

## Logging (RUST_LOG)

- Logging initializes once at process startup, before the configuration is loaded.
- Logs are written to standard error as structured events; the level filter comes from the exact, case-sensitive environment variable `RUST_LOG`.
- The default filter is `info` when `RUST_LOG` is unset, empty, or whitespace-only — applied silently.
- An invalid non-empty value prints exactly one `warning:` line to stderr, then falls back to `info`.
- A valid value (e.g. `debug`, `trace`) is honored verbatim after trimming surrounding whitespace.
- Operator guide: ["Logging (RUST_LOG)"](../README.md#logging-rust_log).

## See Also

- [README](../README.md) — project overview, commands, and the operator logging guide.
- [`docs/authentication.md`](authentication.md) — the OAuth authorization flow, credential files, and recovery guidance.
- [`config/config.example.json`](../config/config.example.json) — committed example template.

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
- Only one configuration file is used per run: the default path or the single `--config <path>` value, which may be given at most once.

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

The example contains no secrets today, but treat the real `config/config.json` as private: it is gitignored, must never be committed, and carries no commit-review protection.

## Field Reference

All five top-level sections are required. Each field below lists its rule and the value from `config/config.example.json`. Section and field names must match exactly as written; unknown or misspelled fields are rejected.

### youtube.broadcast

- `title` (string) — broadcast title; must be non-blank. Example: `Rust YouTube Streamer Prototype`.
- `description` (string) — broadcast description; must be non-blank. Example: `YouTube Live streaming prototype`.
- `privacy_status` (string) — one of `private`, `public`, `unlisted` (case-sensitive). Example: `unlisted`.

### video

- `width` (integer) — frame width; must be greater than 0. Example: `1920`.
- `height` (integer) — frame height; must be greater than 0. Example: `1080`.
- `fps` (integer) — frames per second; must be greater than 0. Example: `30`.
- `pixel_format` (string) — must be exactly `rgb24`; any other value is rejected, never substituted. Example: `rgb24`.

### renderer

- `font` (string) — font file path; must be non-blank. Example: `fonts/console.ttf`.
- `font_size` (integer) — must be greater than 0. Example: `32`.
- `line_height` (integer) — must be greater than 0. Example: `40`.
- `left_margin`, `top_margin`, `right_margin`, `bottom_margin` (integers) — each must be 0 or greater. Example: `20` each.
- `text_color` (string) — must be non-blank. Example: `#FFFFFF`.
- `background_color` (string) — must be non-blank. Example: `#000000`.
- Vertical layout rule: `top_margin + bottom_margin + line_height` must not exceed `video.height`, so at least one rendered line fits.

### chat

- `log_file` (string) — chat log path; must be non-blank. Example: `logs/chat.log`.

### ffmpeg

- `executable` (string) — FFmpeg executable name or path; must be non-blank. Example: `ffmpeg`.
- `video_codec` (string) — must be non-blank. Example: `libx264`.
- `preset` (string) — must be non-blank. Example: `veryfast`.
- `bitrate` (string) — must be non-blank. Example: `6000k`.

Existence of the font file or the FFmpeg executable is checked by the consuming component in a later phase, not at configuration load; configuration load checks values for non-blankness and path syntax only. FFmpeg itself is a separately provided external prerequisite — this phase neither installs nor launches it.

## Validation Rules

- Unknown fields are rejected at deserialize time.
- Wrong value types are rejected at deserialize time.
- Semantic checks (all violations are collected and reported together):
  - `video.width`, `video.height`, `video.fps`, `renderer.font_size`, and `renderer.line_height` must be greater than 0.
  - `video.pixel_format` must be exactly `rgb24`.
  - Renderer margins must be 0 or greater, and `top_margin + bottom_margin + line_height` must not exceed `video.height`.
  - `privacy_status` must be one of `private`, `public`, `unlisted` (case-sensitive).
  - Required strings must be non-blank: broadcast `title` and `description`, `renderer.font`, `renderer.text_color`, `renderer.background_color`, `chat.log_file`, and all four `ffmpeg` fields.
- On failure: one actionable error names the configuration file and field, and the process exits with code 1.
- A missing or unreadable configuration file is also a configuration failure: reported and exited with code 1.

## Command-Line Usage and Exit Codes

```
rust-youtube-streamer-service [--config <path>] <command>
rust-youtube-streamer-service auth|run [--config <path>]
rust-youtube-streamer-service --help
```

- Both `auth` and `run` load and validate the configuration before running.
- `auth` — runs the OAuth authorization flow; **not implemented yet**: after configuration loads and validates, it prints one `error: authentication is not implemented yet` line to stderr and exits 3. It never contacts YouTube and never requests or stores credentials.
- `run` — starts the streaming runtime; **not implemented yet**: after configuration loads and validates, it prints one `error: runtime pipeline is not implemented yet` line to stderr and exits 3. It never creates YouTube resources and never starts FFmpeg.
- `--config <path>` — configuration file to load; accepted before or after the command, at most once. Absent means the default `config/config.json`.
- `--help` — accepted only before the command; prints usage to stdout and exits 0.
- Exit codes: 0 = successful `--help` (and successful pipeline, though no command currently completes successfully), 1 = configuration load/parse/validation or logging-init failure, 2 = usage error (missing or unknown command/option, repeated `--config`, missing `--config` value, `--help` after the command, non-UTF-8 argument), 3 = selected mode not implemented yet (`auth` and `run`).

## Logging (RUST_LOG)

- Logging initializes once at process startup, before the configuration is loaded.
- Logs are written to standard error as structured events; the level filter comes from the exact, case-sensitive environment variable `RUST_LOG`.
- The default filter is `info` when `RUST_LOG` is unset, empty, or whitespace-only — applied silently.
- An invalid non-empty value prints exactly one `warning:` line to stderr, then falls back to `info`.
- A valid value (e.g. `debug`, `trace`) is honored verbatim after trimming surrounding whitespace.
- Operator guide: ["Logging (RUST_LOG)"](../README.md#logging-rust_log).

## See Also

- [README](../README.md) — project overview, commands, and the operator logging guide.
- [`config/config.example.json`](../config/config.example.json) — committed example template.

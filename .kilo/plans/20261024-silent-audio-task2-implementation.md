# Per-Task Implementation Plan — Ingest "No data" Task 2: Silent-Audio Track on the Streaming Vector (4.1b → 4.6)

Plan date: 2026-10-24 · Owner: Architector Agent (Critical Workflow sub-step 4.1b)
TODO: `.agent/todos/20261024/20261024-todo-1.md` · Global plan (BINDING): `.kilo/plans/20261024-ingest-no-data-audio-signal.md`
Task: **Task 2 — Silent-Audio Track on the Streaming Vector (CONDITIONAL)**
Branch: `fix/ingest-no-data` at HEAD `7ed6309`, tree clean. Version already **0.11.0** (workflow step 3 bumped it; NO version change in this task — do NOT touch `Cargo.toml`'s version, no lockfile change expected since NO dependencies change). NOT front-end → no 4.1a.

---

## 0. Scope Contract (READ FIRST)

- **Condition rising:** the research gate is CONFIRMED — implement the silent-audio remedy end-to-end (§1). If any stop-and-report condition in §10 fires BEFORE the code is written, the implementer switches to the recorded skip path only with the CALLER's explicit instruction — never alone.
- Code touches EXACTLY these files (no others, no new directories):
  - `src/config/model.rs` — add `AudioConfig` + `FfmpegConfig.audio` + `FfmpegConfig::audio_enabled`
  - `src/config/test_fixtures.rs` — mirror the example in `valid_config()`
  - `src/config/loader.rs` — wire ONE new test roster module
  - `src/config/ffmpeg_audio_parse_tests.rs` — NEW parse-pins roster
  - `src/streaming/args.rs` — the streaming-vector extension (constants + builder rework)
  - `src/streaming/args_tests.rs` — re-pin the video-only vector + exact with-audio vector pin
  - `src/streaming/ffmpeg_real_tests.rs` — ONE new real-FFmpeg loopback pin of the `-shortest` mechanism
  - `src/app/run/streaming_stage_tests.rs` — argv-pin extension (audio tokens)
  - `config/config.example.json` — `"audio": { "enabled": true }` inside `ffmpeg`
  - NEVER: `build_arguments`' output vector, `error.rs`, `ffmpeg.rs` logic, `spawn.rs`, `pipe_drain.rs`, `rtmps_destination.rs`, `sink.rs`, `video_output.rs`, `utc_timestamp.rs`, `local_finalize.rs`, `mod.rs` re-exports (no new public items), `validation.rs`, any `src/youtube/*`, any `src/app/run` source file, `src/app/demo/*`, `src/app/pipeline/*`, `Cargo.toml`, `Cargo.lock`, `Dockerfile`, `scripts/*`.
- Docs + metadata touches (committed in the docs commit, per caller division): `docs/configuration.md`, `docs/production-run.md`, `README.md`, `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md`.
- The implementer is a JUNIOR developer: every structural decision is encoded below (D1–D14). No judgment: if something is not specified, STOP and ask the caller.

---

## 1. RESEARCH-GATE CONFIRMATION (formal — record in this plan only; no new citation page is fetched)

The user rule (TODO Scope `Task 2`, mirroring the made-for-kids research gate): implement ONLY IF (a) the official research concludes the ingest requires/reliably needs an audio track, OR (b) the operator probe results identify the audio track as the discriminator.

**GATE VERDICT: IMPLEMENT — condition (a) is MET by official documentation.** The caller (the approving user) has explicitly ruled condition (a) as met and ruled that no operator probe results are required to proceed; this overriding ruling supersedes the global plan's Task-2 pause rule (which had reserved Task 2's 4.1b for operator probe consumption).

Carried evidence (official citations, recorded verbatim in `.kilo/plans/20261024-ingest-no-data-task1-implementation.md` §1 and mirrored in `.agent/project-info/context.md`'s 2026-10-24 Task-1 block):

- §1.1 — `liveStreams` health-status issue **`noAudioStream`**: "The ingestion stream does not contain any audio streams, but it **must contain exactly one audio stream**." (https://developers.google.com/youtube/v3/live/docs/liveStreams, "Last updated 2026-10-15 UTC") — the documented ingest expectation is exactly one audio stream; our video-only wire form contradicts it (C1 supported at the documentation level).
- §1.2 — encoder-settings page (https://support.google.com/youtube/answer/2853702): Audio codec "AAC or MP3"; audio sample rate "44.1 KHz for stereo audio, 48 KHz for 5.1"; audio bitrate "128-Kbps for stereo"; tests "should include audio and movement in the video". **The remedy constants (AAC, 48 kHz stereo, 128 kbit/s) sit exactly inside the official recommended set** (both 44.1 and 48 kHz are inside the recommended set per the `audioSampleRate` reconciliation note in Task-1 §1.2.4).
- §1.4 — community corroboration (LABELED community, never official): video-only ingest reproduces the incident; silent `anullsrc` is the working pattern (SO 64745099 "YouTube needs audio.", 38944747, 39660547, 12368151 canonical command form).
- §1.5 — FFmpeg facts backing the exact tokens: `anullsrc` options `channel_layout` (alias `cl`, default `stereo`) and `sample_rate` (alias `r`, default 44100) per `libavfilter/asrc_anullsrc.c`; long option names are the locked convention; `-shortest` = "Finish encoding when the shortest output stream ends." (ffmpeg.html §5.11; the documented caveat is buffering latency via `-shortest_buf_duration`; **no FLV/RTMP-specific incompatibility is documented**); `lavfi` is an internal input device present in standard Windows builds.

Honest bounds carried into every artifact (do NOT soften): the per-issue severity of `noAudioStream` is NOT published as a static table (the ingest-lock refusal is the operator probe's empirical question); whether current-era Studio accepts video-only is likewise probe-gated. **The operator probes REMAIN RECOMMENDED verification** — the remedy ships as the sanctioned design change (brief §10.4), not as a claim that the lock refusal is proven.

---

## 2. Encoded Design Decisions (D1–D14 — BINDING)

### D1 — Verdict

IMPLEMENT (no skip-record path is authored; §10's stop conditions can still pause the work with the caller).

### D2 — Config model shape (frozen)

In `src/config/model.rs`:

1. `FfmpegConfig` gains ONE field:

```rust
/// Optional silent-audio section for the production streaming vector. An
/// absent section (or an absent/false `enabled`) keeps the video-only
/// argument vector byte-identical to the pre-0.11 form.
pub audio: Option<AudioConfig>,
```

2. Add the new section struct right after `FfmpegConfig`:

```rust
/// Silent-audio settings for the streaming FFmpeg vector. The demo/local
/// recording vector is unaffected: `demo` never gains audio.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AudioConfig {
    /// `true` joins a silent AAC audio track (an `anullsrc` lavfi input at
    /// 48 kHz stereo, 128 kbit/s, cut by `-shortest`) to the STREAMING
    /// vector. Absent or `false` keeps the stream video-only.
    pub enabled: Option<bool>,
}
```

3. Add the accessor as an inherent method on `FfmpegConfig` (public surface stays private to the crate):

```rust
impl FfmpegConfig {
    /// Whether the production streaming vector extends with the silent
    /// audio track: the section and the flag are both optional, so
    /// section-absent, `enabled`-absent, and `false` all mean video-only.
    pub(crate) fn audio_enabled(&self) -> bool {
        self.audio
            .as_ref()
            .and_then(|audio| audio.enabled)
            .unwrap_or(false)
    }
}
```

Rationale for `Option<AudioConfig>` at the section level (deliberate, NOT the `made_for_kids` flat-field shape): configuration files written for versions before 0.11.0 contain NO `audio` object; a REQUIRED section would break every existing operator `config/config.json` at upgrade. The nested-object shape keeps those files loading (video-only) while `deny_unknown_fields` still rejects misspelled section/field names. WRONG types are rejected at deserialize time in every case (`"enabled": "yes"` → parse error; `"audio": "yes"` → parse error).

### D3 — Example JSON and fixture lock

1. `config/config.example.json` — inside `"ffmpeg"`, after the `"bitrate": "6000k"` line, add (the example demonstrates the shipping remedy):

```json
    "bitrate": "6000k",
    "audio": {
      "enabled": true
    }
```

2. `src/config/test_fixtures.rs::valid_config` mirrors the example exactly (the `made_for_kids` fixture precedent: the fixture carries `Some(false)` mirroring the example's `false` — here `Some(Some(true))` mirroring the example's `true`):

```rust
ffmpeg: FfmpegConfig {
    executable: "ffmpeg".to_string(),
    video_codec: "libx264".to_string(),
    preset: "veryfast".to_string(),
    bitrate: "6000k".to_string(),
    audio: Some(AudioConfig {
        enabled: Some(true),
    }),
},
```

(extend the existing `use crate::config::model::{...}` list with `AudioConfig`).

Consequence (encoded, deliberate): `valid_config()` → every streaming-vector test fixture carries audio ON, so the with-audio vector is the primary pinned argv (the remedy ships); the pre-0.11 video-only vector remains pinned by explicitly clearing the field in the dedicated tests (§S5). `build_arguments` (demo) ignores audio entirely, so the demo vector and all demo pins are byte-stable WITHOUT any test edit.

### D4 — Vector extension: EXACT argument order (frozen)

CONSTANTS (in `src/streaming/args.rs`, named — no magic values):

```rust
const AUDIO_CHANNEL_LAYOUT: &str = "stereo";
const AUDIO_SAMPLE_RATE: u32 = 48000;
const AUDIO_INPUT_FORMAT: &str = "lavfi";
const AUDIO_CODEC: &str = "aac";
const AUDIO_BITRATE: &str = "128k";
const VIDEO_SHORTEST_STREAM_BOUND: &str = "-shortest";
```

EXACT with-audio streaming vector (tokens in order; `<...>` substitutes; this 34-token list is the pin):

```
 1  -nostdin
 2  -n
 3  -f
 4  rawvideo
 5  -pix_fmt
 6  <video.pixel_format = rgb24>
 7  -s
 8  <width>x<height>
 9  -framerate
10  <fps>
11  -i
12  pipe:0
13  -f
14  lavfi
15  -i
16  anullsrc=channel_layout=stereo:sample_rate=48000
17  -c:v
18  <video_codec>
19  -preset
20  <preset>
21  -b:v
22  <bitrate>
23  -g
24  <2 × fps>
25  -pix_fmt
26  yuv420p
27  -c:a
28  aac
29  -b:a
30  128k
31  -shortest
32  -f
33  flv
34  <destination_url>
```

Ordering rationale (record in the module docs, short):

- The audio INPUT pair rides immediately after the video input pair (`-i pipe:0` … then `-f lavfi -i anullsrc=…`): each `-f` applies to the input that follows it, so `-f lavfi` cannot clash with `-f rawvideo`; the token run from `-nostdin` through `pipe:0` stays a byte-identical PREFIX of today's vector, and the output-token run from `-c:v` through `yuv420p` stays byte-identical too — the only insertions are tokens 13–16 (after `pipe:0`) and tokens 27–31 (after `yuv420p`).
- NO `-map` is needed: FFmpeg's default output selection automatically picks one stream of each type — the only video stream (rawvideo) and the only audio stream (anullsrc) exist, so default selection feeds both without explicit mapping.
- `-c:a aac -b:a 128k` sit in the output group after `-pix_fmt yuv420p` (all audio codec options grouped).
- **`-shortest` sits immediately BEFORE `-f flv`** (an output option; FFmpeg accepts output options in any order before the output filename): this keeps the LOCKED FINAL PAIR `-f flv <url>` intact, which is asserted by the existing `args_tests.rs` (`arguments[length - 3] == "-f"`, `arguments[length - 2] == "flv"`) and `streaming_stage_tests.rs` pins — reordering would churn three pinned tests for zero effect on FFmpeg behavior.
- Shutdown-cleanness argument (module doc line): at Ctrl+C the app writes stdin-EOF and the rawvideo input ends; the encoded monitor stream's video stream then ENDS, and `-shortest` ("Finish encoding when the shortest output stream ends.") cuts the infinite anullsrc at that shortest (video) stream end — the exit stays clean and bounded without ANY change to the app's shutdown order, `finish()` sequence, or exit-code mapping.

Const-note: `AUDIO_SAMPLE_RATE` is a plain `u32` and `VIDEO_SHORTEST_STREAM_BOUND` is placed near the OTHER stream constants (`STREAM_MUXER`, `GOP_MULTIPLIER` stay where they are).

### D5 — The `-shortest` mechanism pick + fallback decision procedure (ONE mechanism rule)

- PICKED: **`-shortest` alone** (mechanism 1). Officially documented semantics, no FLV/RTMP incompatibility documented, community/probe precedent uses exactly this token.
- FALLBACK decision procedure (ONLY if the new real-FFmpeg loopback pin of §3 S7 exhibits the unbounded-audio shape on the pinned dev-image FFmpeg `5.1.9-0+deb12u1`): mechanism 2 = **replace** `-shortest` with the two output tokens `-fflags +shortest` and `-max_interleave_delta 100` (the interleaving-variant form named in the global plan's risk). NEVER ship both. After the swap: re-run the real loopback pin, then the full gate.
- If BOTH mechanisms fail in the container environment: STOP and report (§10's list; §7 condition 7 in the implementer's copy of this plan). No third variant is improvised — the workflow pauses and the caller/adherence pass decides. Also: if the swap ever happens, the args tests' pinned 34-token vector pin and the §S8 stage asserts shift accordingly ONLY via a caller-approved plan amendment — never silently.
- Loopback-pinned only: the real pin runs against the LOCAL FLV file (§S7) — NO test pushes any RTMPS packet, and no test reaches any network endpoint; ffmpeg_stream fakes never spawn a real ffmpeg.

### D6 — Demo/local byte-stability

`build_arguments` in `args.rs` stays BYTE-UNTOUCHED (its body, its doc's local-vector paragraph, and its behavior). The demo command, usage text, `OutputKind::Demo`, `videos/` naming, demo tests: all untouched. Audio is streaming-only; `audio_enabled()` is never consulted by the local path.

### D7 — Task-1 diagnostics byte-stability

`StreamExit`/`exit_diagnostics`/redaction (`pipe_drain.rs`, `sink.rs`, `ffmpeg.rs::finish`), and the app-layer evidence helpers (`src/app/run/streaming.rs`, `src/app/run/gate_diagnostics.rs`): UNTOUCHED. The with-audio vector flows through the same `SpawnPlan`/spawn engine; diagnostics redaction keys on the destination URL only. No new log line, no new error variant.

### D8 — Validation: NO new rule count (21 stays)

Verified precedent: the made-for-kids option added ZERO validation rules (`.agent/project-structure.md`: "`made_for_kids` needs NO validation rule — serde's typed `Option<bool>` is the contract"). Same here: `ffmpeg.audio`/`ffmpeg.audio.enabled` need NO rule in `src/config/validation.rs` — wrong types are rejected at deserialize time by the typed `Option<AudioConfig>`/`Option<bool>` (both struct-level `deny_unknown_fields`). The "21 rules" count in `docs/configuration.md` and every recorded count stays 21. `validation.rs` is NOT edited.

### D9 — Test roster (D7-style names; final net delta +9; expected full-suite neighborhood 387 = 378 + 9, record the OBSERVED count)

**A. NEW `src/config/ffmpeg_audio_parse_tests.rs` (6 tests)** — wired from `loader.rs` exactly like `broadcast_made_for_kids_parse_tests` (add the `#[cfg(test)] #[path = "ffmpeg_audio_parse_tests.rs"] mod ffmpeg_audio_parse_tests;` foot):

1. `committed_example_ships_audio_enabled_true` — loads `config/config.example.json`; `loaded.ffmpeg.audio == Some(AudioConfig { enabled: Some(true) })`.
2. `audio_section_absent_masks_to_none` — removes the `"audio"` key from the example's `ffmpeg` object; `audio.is_none()`.
3. `audio_enabled_absent_inside_the_section_masks_to_none` — sets `"audio": {}`; `enabled` is `None`.
4. `audio_enabled_true_parses` — `Some(true)`.
5. `audio_enabled_false_parses` — `Some(false)`.
6. `audio_enabled_wrong_type_rejected` — `"enabled"` set to the string `"yes"`; `ConfigError::Deserialization`; the display names the temp-file path.

Helpers mirror the mfk roster exactly (`example_value()`, `load_modified(&Value)`, `write_temp_config(dir, contents)`, `expect_load_error(path)` — copy them from `broadcast_made_for_kids_parse_tests.rs` into THIS roster, same bodies; the mfk roster stays untouched).

**B. `src/streaming/args_tests.rs` (2 new + 1 modified)**:

- MODIFIED `stream_arguments_match_the_locked_output_shape` — insert `config.ffmpeg.audio = None;` directly after the `config.video` mutations (with the test-managed doc note); ALL existing assertions unchanged → the pre-0.11 video-only vector re-pin (byte-identical).
- NEW `stream_arguments_with_audio_match_the_locked_extended_shape` — uses `valid_config()` (audio ON via fixture) mutated only to `width = 1280`, `height = 720`, `fps = 25`; pins the FULL 34-token exact vector of §D4 (URL = `DESTINATION_FIXTURE_URL`); additionally asserts the exact insertion boundaries: token indexes 12–16 (`"pipe:0", "-f", "lavfi", "-i", "anullsrc=…"`) and the presence of one `-movflags`-free run (the pre-existing style).
- NEW `audio_absent_false_and_unset_enabled_hold_the_identical_video_only_vector` — three configs (`audio: None`, `audio: Some(AudioConfig { enabled: None })`, `audio: Some(AudioConfig { enabled: Some(false) })`) → three vectors; `assert_eq!` pairwise identical (the byte-identical-off-path guarantee).

`stream_gop_is_two_frames_per_second` stays byte-unchanged (gop holds in both shapes).

**C. NEW real-FFmpeg loopback pin in `src/streaming/ffmpeg_real_tests.rs` (1 test):**

`real_ffmpeg_audio_stream_bounds_the_infinite_silence_and_carries_aac_audio`

Shape (test-local vector; the argv-exactness is pinned by B; this test pins the MECHANISM):

- Requires `/usr/bin/ffmpeg` + `/usr/bin/ffprobe` via the existing `require_image_tool` (absent = loud suite failure, never a skip).
- One temp dir; output `real-audio-smoke.flv` inside it.
- Test-local vector (documented in the test header as the production with-audio shape with a LOCAL FLV target instead of an RTMPS destination — no network, ever):

```
-nostdin -n -f rawvideo -pix_fmt rgb24 -s 64x36 -framerate 5 -i pipe:0
-f lavfi -i anullsrc=channel_layout=stereo:sample_rate=48000
-c:v libx264 -preset veryfast -b:v 6000k -g 10 -pix_fmt yuv420p
-c:a aac -b:a 128k -shortest -f flv <output_path>
```

- Spawn via `std::process::Command` (piped stdin/stdout/stderr; no shell — mirror the file's style), write the existing `SMOKE_FRAME_COUNT` of `SMOKE_FRAME_BYTES` frames, DROP the stdin handle (the EOF), then the bounded wait: loop `child.try_wait()` bounded at `REAL_AUDIO_BOUND_WAIT_ATTEMPTS` = 15 × 1 s sleeps (`try_wait().expect("joining the audio smoke must not fail with io"); if let Some(status) → break`) — if the bound expires: `child.kill()` then `panic!("the anullsrc audio stream was not bounded by -shortest at stdin EOF; mechanism 1 failed (D5)")`. Thermal read: a HANG is the failure signature for an unbounded mechanism; the kill+panic converts it into loud evidence.
- On a clean exit: `assert!(status.success())`; ffprobe the FLV with `-v error -show_entries stream=codec_type,codec_name,sample_rate,channels -of default=noprint_wrappers=1` (probe BOTH streams, no `-select_streams v:0` here): assert one `codec_name=h264` line and one audio triplet `codec_name=aac`, `sample_rate=48000`, `channels=2` — extract via the existing `probe_value` helper (`probe_value(&stdout, "codec_name")` returns the FIRST match; for two values the test finds BOTH with fresh iter lines logic: collect all lines and assert the `aac` line exists alongside the `h264` line, then `probe_value`-style extraction pinned per value taken from the AUDIO line — keep it simple: parse lines into a map of finds; the exact assertion form is local detail within these semantics).

**D. MODIFIED `src/app/run/streaming_stage_tests.rs` (1 test extended, count unchanged):**

`stream_pipeline_passes_the_destination_to_the_ffmpeg_argv` gains audio-token assertions appended after the existing ones (the fixture now yields the with-audio argv). The implementer copies the EXACT appended block from §3 S8 verbatim — the four new asserts name the anullsrc source token, the `-c:a aac` pair, the `-b:a 128k` pair, and the `-shortest` token, each with its fixed message. The four existing assertions stay UNTOUCHED (destination-final, `-f`/`flv` before it, `-movflags` absent, `-g 60` present).

### D10 — Verification protocol (order fixed)

1. `cargo fmt --check` inside the Compose rust service (host: NEVER native `cargo`).
2. Targeted tests (two passes, loopback/offline): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked ffmpeg_audio` then `... cargo test --locked ffmpeg_real` (record `Exit code`/`Passed` lines).
3. FULL standard gate on the tree: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` — exit 0, four PASS lines (`fmt-check`, `check`, `test`, `clippy`), `ALL CHECKS PASSED`; record log path + the CHECKS `test` line's OBSERVED count (expected 387 by §D9 arithmetic, never guessed: `378 baseline + 6 parse + 2 args + 1 real = 387`).
4. BOTH release builds (whole-TODO TODO criterion 5 lands HERE — Task-1 §4.3 deferred because docs-only):
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-linux.sh` → exit 0, `dist/rust-youtube-streamer-service` non-empty (RECORD observed byte size — baseline context: 5211912 at v0.10.0, stale reference; the v0.11.0 sizes are RECORDED FRESH here).
   - `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/build-windows.sh` → exit 0, `dist/windows/rust-youtube-streamer-service.exe` non-empty (record OBSERVED size).
   - `ls -l` cross-check on BOTH items (total size line matches the visible byte size; byte-stability claim never carries without this).
5. §5.4-style audit (§9 checklist below — run AFTER the docs commit so it audits the FINAL tree).

### D11 — Commits (implementer; git SAFETY: read `.gitignore` + check `git status --porcelain` before EACH commit; never stage a gitignored path — `dist/`, `logs/*`, `videos/`, `config/config.json`, `target/` — and never `git add -A`)

1. **Commit 1 (plan tracking)**: stage ONLY `.kilo/plans/20261024-silent-audio-task2-implementation.md` → message: `chore: track ingest no-data Task 2 implementation plan` (caller-authorized per branch precedent `07f48c5`).
2. **Commit 2 (feat)**: stage ONLY `src/config/model.rs`, `src/config/test_fixtures.rs`, `src/config/loader.rs`, `src/config/ffmpeg_audio_parse_tests.rs`, `src/streaming/args.rs`, `src/streaming/args_tests.rs`, `src/streaming/ffmpeg_real_tests.rs`, `src/app/run/streaming_stage_tests.rs`, `config/config.example.json` → message: `feat: add the config-gated silent audio track to the streaming FFmpeg vector`.
3. **Commit 3 (docs)**: stage ONLY `docs/configuration.md`, `docs/production-run.md`, `README.md`, `.agent/project-structure.md`, `.agent/project-info/tech.md`, `.agent/project-info/context.md` → message: `docs: document the ffmpeg.audio.enabled silent audio track`.
4. NEVER stage: the dev-checks gate logs, the dist artifacts, config/config.json (ignored), and NEVER create the plan-tracking/pushes (push is workflow step 5, NOT the implementer).

### D12 — Docs targets (exact edit specs)

**`docs/configuration.md` — `### ffmpeg` Field Reference append** (after the `bitrate` bullet, before the closing "Existence…" paragraph):

```
- `audio` (object, optional) — silent-audio configuration for the production `run` streaming vector only (`demo` ignores it). Contains one field:
  - `enabled` (boolean, optional) — when `true`, the streaming FFmpeg command gains a silent audio track: a second `lavfi` input (`anullsrc=channel_layout=stereo:sample_rate=48000`) encoded as AAC at 128 kbit/s with an output `-shortest` bound ("Finish encoding when the shortest output stream ends.") so stdin-EOF shutdown stays clean and bounded. Constant choice sat inside the official encoder-settings recommendation set (AAC codec; 44.1/48 kHz; 128 kbit/s stereo) and answers the documented ingest expectation that the stream "must contain exactly one audio stream" (`noAudioStream` health issue, liveStreams reference). When the `audio` section is absent, or `enabled` is absent or `false`, the streaming vector stays video-only, byte-identical to the pre-0.11 form; configuration files written for versions before 0.11.0 are unchanged and keep parsing (no validation-rule change; wrong boolean types are rejected at deserialize time; the rule count stays 21).
```

**`docs/production-run.md` — TWO edits:**

1. Append at the END of `## Ingest Probe Procedure (Operator-Run)` (after the "Cleanup after the probes:" block, before `## Manual Verification Procedure (Operator-Run)`):

```
**Product-level status (v0.11.0):** the silent-audio remedy ships config-gated —
`ffmpeg.audio.enabled: true` (the example config enables it) adds probe (ii)'s
wire shape (`anullsrc` 48 kHz stereo → AAC 128k → `-shortest`) to the production
streaming vector, while absent/`false` keeps the byte-identical video-only
stream. Once a run ships with audio enabled, the remedy path SUPERSEDES this
procedure as the deployed fix; the probes remain the operator's verification
and diagnosis path: if an audio-enabled run still ends with the
publish-succeeded-but-inactive/"No data" shape, the outcome table above maps
the cause PAST C1 (content-level or environment — report the recorded results
to the planner rather than re-judging in the field).
```

2. The Out-of-Scope bullet CHANGE — the list item exactly `- Audio.` (under `## Out of Scope`) is REPLACED with:

```
- Audio production or mixing (the MVP renders frames only; the v0.11.0 config-gated silent `anullsrc` audio track is an ingest-signal carrier shipped as the brief-section-10.4 explicit design change, not produced content).
```

**`README.md`**: ONE expected drift-fix — the Current-Status/test-count sentence that names the frozen count "378 tests" becomes the OBSERVED full-gate count (§D10.3; the 2026-10-23 mfk fix precedent). NOTHING ELSE unless §7 catches contradictory claims.

### D13 — Metadata refreshes (same docs commit)

- `.agent/project-structure.md`:
  - `Cargo.toml` version label: **`v0.10.0` → `v0.11.0`** (document the already-bumped step-3 truth; the map's version lagged Task 1's docs-only window).
  - `src/config` block: pass — `model.rs` entry gains "…plus the optional `FfmpegConfig.audio` (the silent-audio streaming section: `AudioConfig { enabled: Option<bool> }`; section/flag absent or false = the byte-identical video-only streaming vector; the `audio_enabled()` accessor handles all three false-shapes)"; `loader.rs` entry gains the new roster wiring (the `ffmpeg_audio_parse_tests` roster, 6 tests); the config test-count sentence `25 → 31`.
  - `src/streaming` block: `args.rs` entry — replace the "the Phase-05 stream" tail with the optional-audio clause "…; when `config.ffmpeg.audio_enabled()` the streaming vector gains the silent lavfi audio input (constants `AUDIO_CHANNEL_LAYOUT="stereo"`, `AUDIO_SAMPLE_RATE=48000`, `AUDIO_INPUT_FORMAT="lavfi"`, `AUDIO_CODEC="aac"`, `AUDIO_BITRATE="128k"`, `VIDEO_SHORTEST_STREAM_BOUND="-shortest"`; NO `-map` needed — default stream selection picks the only audio/only video) plus the output group `-c:a aac -b:a 128k -shortest`, cut when the video stream ends (stdin-EOF shutdown stays bounded; brief §10.4 design change; the exact 34-token shape is plan §D4)"; the args.rs test counts gain (2 new + 1 modified); the `ffmpeg_real_tests.rs` entry size 1 → 2 tests (add the bounded-audio mechanism pin); the `streaming_stage_tests` one-liner in the run block's tests list gains "(extended with the audio-token asserts on the with-audio argv)".
  - Validate counts against the FINAL tree before the docs commit (the wrinkle below governs: the counts recorded here are the post-config-and-streaming expected values).
- `.agent/project-info/tech.md`: add ONE bullet under the dependency/runtime decision notes: "`ffmpeg.audio.enabled` (v0.11.0) ships the silent-audio streaming remedy (anullsrc 48 kHz stereo → native AAC 128k; `-shortest` bounds the infinite source at stdin EOF); no new dependency, no Tokio, the streaming boundary still holds std + config-only; the mechanism pin lives in the real-FFmpeg container smoke suite (pinned image FFmpeg 5.1.9-0+deb12u1)."

### D14 — Stop-and-report conditions (implementer holds them)

See §10 — the same list binds during implementation (the architector's §10 is shared verbatim).

---

## 3. Encoded Implementation Steps (S1 → S14, in this exact order)

### S0 — precondition checks (fail-fast)

1. `git status --porcelain` → expect EMPTY (clean tree at `7ed6309`).
2. `git log --oneline -4` → expect the branch at `7ed6309` (docs commit) with `983293e` (bump) and `7... plan-tracking` ancestors — NO version re-check (`0.11.0` already), NO branch setup (step 2 done).

### S1 — plan-tracking commit (caller-authorized)

Commit the plan file exactly per §D11.1. If the planner already tracked it (unavoidable from inside the sub-step), SKIP commit 1 and record the skip honestly in the S-report back — never double-commit an already-tracked file.

### S2 — `src/config/model.rs`

Apply §D2.1–D2.3 verbatim. Constrain: the whole FfmpegConfig addition block + AudioConfig + the accessor stay under ~28 lines; the file must stay under its 200-line cap (147 → ~173 — OK).

### S3 — `src/config/test_fixtures.rs`

Apply §D3.2 (with the `AudioConfig` import line and `audio: Some(AudioConfig { enabled: Some(true) })` inserted after `bitrate`).

### S4 — `config/config.example.json`

Apply §D3.1 EXACTLY (2-space indent; `"audio"` as the 5th key inside `"ffmpeg"`; NOTHING else changes).

### S5 — `src/streaming/args.rs` (line-budget-bound)

Exact edits:

1. Module doc: after the locked-streaming-vector paragraph (the `plan D-T3-5 F-S2` block), insert the SHORT:

```rust
//! Streaming audio extension (brief §10.4 explicit design change; plan
//! 20261024 §D4): when `ffmpeg.audio_enabled()` the streaming vector gains
//! the second input pair `-f lavfi -i <anullsrc source>` (the source token
//! is `anullsrc=channel_layout=stereo:sample_rate=48000`, kept as ONE argv
//! token) and the output tokens `-c:a aac -b:a 128k -shortest`. Constants
//! are the official-recommendation set (AAC; 44.1/48 kHz allowed — 48 kHz
//! here; 128 kbit/s stereo). Default output selection pairs the only video
//! stream with the only audio stream, so NO `-map` is needed. `-shortest`
//! sits immediately BEFORE `-f flv <url>`: an output option applied when
//! the shortest output stream ends, keeping the closed video stream the
//! terminator of the whole encode (FFmpeg HTML §5.11) so the stdin-EOF
//! shutdown stays clean and bounded; placing it before `-f flv` preserves
//! the locked `-f flv <url>` final pair asserted by the pinned tests.
```

(≤ 12 doc lines.)

2. Constants block appended right after `const GOP_MULTIPLIER: u32 = 2;` (§D4 list verbatim).
3. `build_stream_arguments` REWORK (DO NOT touch `build_arguments` or its doc block; the pixel-format guard stays FIRST in build_stream_arguments; the with/without shapes share ONE builder through a mut vec):

```rust
pub(super) fn build_stream_arguments(
    config: &AppConfig,
    destination: &RtmpsDestination,
) -> Result<Vec<String>, StreamingError> {
    if is_non_rgb24_pixel_format(&config.video.pixel_format) {
        return Err(StreamingError::UnsupportedPixelFormat {
            value: config.video.pixel_format.clone(),
        });
    }
    let mut arguments = stream_input_arguments(config);
    if config.ffmpeg.audio_enabled() {
        arguments.extend(audio_input_arguments());
    }
    arguments.extend(stream_video_output_arguments(config));
    if config.ffmpeg.audio_enabled() {
        arguments.extend(audio_output_arguments());
    }
    arguments.push(STREAM_MUXER.to_string());
    arguments.push(destination.as_url().to_string());
    Ok(arguments)
}

fn stream_input_arguments(config: &AppConfig) -> Vec<String> {
    vec![
        "-nostdin".to_string(),
        "-n".to_string(),
        "-f".to_string(),
        "rawvideo".to_string(),
        "-pix_fmt".to_string(),
        config.video.pixel_format.clone(),
        "-s".to_string(),
        format!("{}x{}", config.video.width, config.video.height),
        "-framerate".to_string(),
        format!("{}", config.video.fps),
        "-i".to_string(),
        PIPE_SOURCE.to_string(),
    ]
}

fn stream_video_output_arguments(config: &AppConfig) -> Vec<String> {
    vec![
        "-c:v".to_string(),
        config.ffmpeg.video_codec.clone(),
        "-preset".to_string(),
        config.ffmpeg.preset.clone(),
        "-b:v".to_string(),
        config.ffmpeg.bitrate.clone(),
        "-g".to_string(),
        format!("{}", stream_keyframe_interval(config.video.fps)),
        "-pix_fmt".to_string(),
        OUTPUT_PIXEL_FORMAT.to_string(),
    ]
}

fn audio_source_argument() -> String {
    format!(
        "anullsrc=channel_layout={}:sample_rate={}",
        AUDIO_CHANNEL_LAYOUT, AUDIO_SAMPLE_RATE
    )
}

fn audio_input_arguments() -> [String; 4] {
    [
        "-f".to_string(),
        AUDIO_INPUT_FORMAT.to_string(),
        "-i".to_string(),
        audio_source_argument(),
    ]
}

fn audio_output_arguments() -> [String; 5] {
    [
        "-c:a".to_string(),
        AUDIO_CODEC.to_string(),
        "-b:a".to_string(),
        AUDIO_BITRATE.to_string(),
        VIDEO_SHORTEST_STREAM_BOUND.to_string(),
    ]
}
```

HARD BUDGET: `args.rs` MUST stay ≤ 195 total lines after `cargo fmt` (baseline 155; encoded additions ≈ 36). If the final fmt'd file exceeds 195 lines: STOP and report (§7) — do NOT restructure further (the fallback path — a `stream_args.rs` sibling split — requires the caller's ruling).

### S6 — `src/streaming/args_tests.rs`

Per §D9-B: modify ONE test + add TWO tests (copy the exact 34-token vector from §D4; destination = the file's existing `DESTINATION_FIXTURE_URL`; same `valid_config()` idiom). In `stream_arguments_match_the_locked_output_shape`, add `config.ffmpeg.audio = None;` directly after the existing `config.video = ...` mutations with a one-line comment that the `None` pins the pre-0.11 wire form deliberately — NO other change, NO import growth.

### S7 — `src/streaming/ffmpeg_real_tests.rs`

Add ONE test per §D9-C verbatim (with the consts `REAL_AUDIO_BOUND_WAIT_ATTEMPTS: u64 = 15` + `REAL_AUDIO_BOUND_WAIT_SLEEP_MS: u64 = 1000` near the file's existing const block).

### S8 — `src/app/run/streaming_stage_tests.rs`

Extend `stream_pipeline_passes_the_destination_to_the_ffmpeg_argv` per §D9-D using this EXACT appended block (verbatim):

```rust
    assert!(
        argv.iter()
            .any(|argument| argument == "anullsrc=channel_layout=stereo:sample_rate=48000"),
        "the silent lavfi audio source rides as an explicit argv token"
    );
    assert!(
        argv.windows(2)
            .any(|pair| pair[0] == "-c:a" && pair[1] == "aac"),
        "the silent track is encoded as native AAC"
    );
    assert!(
        argv.windows(2)
            .any(|pair| pair[0] == "-b:a" && pair[1] == "128k"),
        "the stereo audio bitrate is 128 kbit/s"
    );
    assert!(
        argv.iter().any(|argument| argument == "-shortest"),
        "the -shortest bound terminates encoding at the shortest stream's end"
    );
```

### S9 — Wire the parse roster

`src/config/loader.rs`: append the `#[cfg(test)] #[path = "ffmpeg_audio_parse_tests.rs"] mod ffmpeg_audio_parse_tests;` foot exactly mirroring the mfk wiring.

### S10 — Write `src/config/ffmpeg_audio_parse_tests.rs`

Per §D9-A (6 tests, helpers copied verbatim from the mfk roster; the file opens with the doc header `//! Parse pins for the optional `ffmpeg.audio` silent-audio section:
//! section-absent/en-flag-absent → `None` (video-only), `true`/`false` parse
//! as the boolean, wrong types are rejected, and the committed example
//! ships `true`.`).

### S11 — Formatting + targeted tests + full gate (§D10.1–D10.3)

Run in order. Do not push forward on any non-zero exit — stop at the FIRST failing check, read the failure, fix ONLY within this plan's letter (no unrelated refactors), and re-run from the failed step.

### S12 — Both release builds + `ls -l` (§D10.4)

TWO fresh runs + the `ls -l` cross-checks; record OBSERVED sizes verbatim; write them into the S-report-back and the docs context block (§S13 slot).

### S13 — Context block skeleton (write into `.agent/project-info/context.md`)

Add the new block AFTER the last existing "## Recent Changes" heading (Task-1's block), titled EXACTLY:

`## Recent Changes (2026-10-24, ingest "No data" — Task 2 silent-audio remedy)`

Fill from actually-executed steps (`[GATE LOG PATH]`, `[TESTS OBSERVED]`, `[LINUX SIZE]`, `[WINDOWS SIZE]`, `[COMMITS]`); the skeleton content (bullet style of the file):

```
* Verdict IMPLEMENT at Task-2 4.1b (caller-ruled research gate: official docs conclude the ingest needs an audio track — `noAudioStream` "must contain exactly one audio stream" + the encoder-settings recommendation set carry the remedy constants; plan `.kilo/plans/20261024-silent-audio-task2-implementation.md` §1). Per-task plan-tracking `[COMMIT1]`; the feature commit `[COMMIT2]`; the docs commit `[COMMIT3]`.
* Config: optional `ffmpeg.audio` section (`Option<AudioConfig>` with `enabled: Option<bool>`) — section absent, flag absent, or false keep the STREAMING arg vector byte-identical to the pre-0.11 video-only wire form (`audio_enabled()` masks all three false-shapes); the example config ships `"audio": { "enabled": true }` and the fixture mirrors `Some(Some(true))` (the `made_for_kids` fixture-mirrors-example precedent); NO validation rule added (the rule count stays 21 — serde's typed `Option`/`deny_unknown_fields` rejection is the contract); the deliberate optional SECTION keeps pre-0.11 operator configs parsing.
* The streaming vector gains, when audio_enabled(): `-f lavfi -i anullsrc=channel_layout=stereo:sample_rate=48000` immediately after the video input pair, plus `-c:a aac -b:a 128k` after `-pix_fmt yuv420p`, plus `-shortest` immediately before the LOCKED `-f flv <url>` final pair (no `-map` needed — default output selection pairs the only video with the only audio stream; `-shortest` cuts the infinite silence when the stdin-fed video stream ends, keeping the stdin-EOF shutdown clean and bounded; brief §10.4 design change). `-shortest` is the ONE mechanism (global-plan risk note); the fallback (`-fflags +shortest -max_interleave_delta 100`) is encoded as a decision procedure only and stays UNTRIGGERED unless the container pin had failed.
* Demo byte-stability: `build_arguments` untouched (audio is streaming-only); the Task-1 diagnostics surface byte-stable; the shutdown order, single-`error:` contract and exit codes untouched.
* Tests: 6 new parse pins (roster `ffmpeg_audio_parse_tests`), 2 new args pins (exact 34-token with-audio vector + the absent/false/None identical-vector proof), 1 real-FFmpeg container pin (`-shortest` bounds the infinite silence; ffprobe aac/48000/stereo), 1 streaming_stage argv extension. Targeted passes `[TARGETED]`; full gate `[GATE LOG PATH]` exit 0, `[TESTS OBSERVED]` passed / 0 failed, `ALL CHECKS PASSED` (baseline 378 + 9 = 387 expected, recorded observed `[TESTS OBSERVED]`).
* Builds: `[LINUX SIZE]` + `[WINDOWS SIZE]`, both `ls -l`-cross-checked (TODO criterion 5 lands here; Task-1 deferred, docs-only).
* Docs: `configuration.md` field-reference bullet (constants + absent/false byte-stability + backward compatibility + 21-rule note); `production-run.md` product-level-status note (the remedy supersedes the probe as remedy path; the probes stay the operator's verification path) and the Out-of-Scope Audio bullet amendment. README test-count drift fix `[README COUNT]`. Metadata: structure map (model/streaming entries; version label v0.11.0) + tech.md audio-remedy bullet + THIS block.
* Secrets: none (no stream keys, no RTMPS URLs, no tokens — audio constants are public official-rec values).
```

### S14 — Docs + metadata + audit commits (§D12, §8, §9 audit, then §D11.3)

Write the D12 docs edits (configuration.md append; production-run.md two edits), then the D13 metadata refreshes, then the CONTEXT block (S13), then run §9's audit, then the docs commit.

---

## 4. Verbatim code snippets (the implementer copies exactly)

§2–§3 carry the code snippets verbatim; this section repeats only the two strings that must survive byte-identically:

1. The anullsrc token (ONE string, INSIDE the runtime vector): `anullsrc=channel_layout=stereo:sample_rate=48000`
2. The audio-flag group sequence: `-c:a aac -b:a 128k -shortest`

---

## 5. Ripples Audit (every construction site of the streaming vector — enumerated)

The implementer runs this first and RECONCILES the expectations below (grep: `rg -n "build_stream_arguments|start_stream|prepared_stream_pipeline" src/`):

| # | Expected Site | Ripple |
| --- | --- | --- |
| 1 | `src/streaming/args.rs::build_stream_arguments` (the sole builder) | §S5 rework |
| 2 | `src/streaming/ffmpeg.rs::start_stream` (the sole production caller) | NONE (call signature and body untouched) |
| 3 | `src/app/pipeline/runner.rs::prepared_stream_pipeline` | NONE (hits `FfmpegProcess::start_stream`) |
| 4 | `src/streaming/args_tests.rs` (2 exact pins + gop helper) | §S6 |
| 5 | `src/streaming/ffmpeg_stream_tests.rs` (start_stream_fake, 4 tests) | NONE (no argv asserts there; fake scripts) |
| 6 | `src/app/run/streaming_stage_tests.rs` (the process argv capture) | §S8 |
| 7 | `src/config/test_fixtures.rs` | §S3 fixture lock (`Some(Some(true))`) |
| 8 | `src/config/model.rs` | §S2 |
| 9 | docs `production-run.md` / `configuration.md` / README | §S14 |

If the grep finds MORE sites than the table, STOP and report (the table IS the plan; an extra site is an unencoded ripple).

---

## 6. Commits (mirror §D11)

1. `[plan-tracking] chore: track ingest no-data Task 2 implementation plan` — ONLY the plan file (S1; SKIP if already tracked).
2. `[feat] feat: add the config-gated silent audio track to the streaming FFmpeg vector` — ONLY §0's code paths (S5→S10, example, stage tests).
3. `[docs] docs: document the ffmpeg.audio.enabled silent audio track` — ONLY the six docs/metadata files of §0 (S14).

---

## 7. Stop-and-Report Conditions (any holder)

1. Tree-dirty S0 precondition fires (pre-existing edits — no clean `7ed6309`).
2. The §5 grep finds sites beyond the table.
3. The final `args.rs` exceeds 195 fmt'd lines (the fallback split needs a caller ruling).
4. The full gate exits non-zero on any internal/FAIL line: `fmt-check`, `check`, `test`, `clippy` (include `PASS`/`FAIL` + the failing sub-test in the report; every fix must stay inside this plan's letters).
5. The builds fail (`build-linux.sh`/`build-windows.sh` non-zero) or `ls -l` cross-check shows a 0-byte artifact.
6. The storage audit (§9) hits ANY of the denied conditions (staged ignore-hit, secret token count > baseline, a NEW `allow(` attribute line).
7. The real loopback pin shows mechanism 1 AND mechanism 2 both failing the bounded-exit (§D5).
8. ANYTHING demands touching a file or behavior beyond §0's closures (e.g. version re-bump — 0.11.0 already live; `Cargo.lock` would be touched by a `cargo test --locked` failure — STOP instead of re-locking).

---

## 8. Docs/Metadata Edit Spec Placeholders (verbatim-marked)

- `docs/configuration.md` §D12-block copy VERBATIM (the 4 sub-paragraphs above, exactly; COMMA placement matters not beyond quoted constant names).
- `docs/production-run.md` §D12-block 1 copy VERBATIM (the D12 "Product-level status" paragraph, 8 lines) + §D12-block 2 verbatim replacement (the Out-of-Scope bullet).
- `README.md` test-count is the ONLY permitted README edit at this step (the §S14 narrow line replacing the previously observed 378).

## 9. §5.4-Style Audit (final-tree; record PASS/FAIL outcomes verbatim)

Run ALL of these AFTER the docs commit (read `.gitignore` once BEFORE the commit, per the gitignore-compliance rule):

```
git status --porcelain                                       → expect EMPTY (the docs commit caught everything)
git ls-files dist/ videos/                                   → expect EMPTY
git check-ignore -v dist/rust-youtube-streamer-service       → expect a `dist/` rule hit (.gitignore line 34 per precedent)
git ls-files credentials/ | grep -v '\.gitkeep'              → expect EMPTY
git grep -n -E "ya29|AIza" -- src/ docs/                     → expect ZERO hits
git grep -n -F "key=" -- src/ docs/                          → expect the pre-existing doc-comment count ONLY (zero new; record the observed count)
git grep -n -F "allow(dead_code)" -- src/                    → expect exactly 8 attribute lines + 1 comment mention (the pre-existing 2026-10-14 baseline; zero new)
git grep -n -F "allow(unused_imports)" -- src/               → expect ZERO
git ls-tree -r --name-only HEAD -- src/ config/              → spot-verify the new test file is tracked
git diff 7ed6309..HEAD --name-only                           → expect EXACTLY the 15 paths of §6 (9 code/example + 6 docs/metadata) and nothing else
```

Failing any line: STOP and report (never self-heal a bad state).

## 10. Stop-and-Report (compiled)

The implementer holds §7's conditions verbatim — they ARE the compiled list (§7 condition 1–8); the architector (adherence, 4.5b) applies the same doors when checking the tree.

## 11. Traceability

| TODO bullet / criterion | Covered by |
| --- | --- |
| Task-2 condition gate | §1 |
| config model field `ffmpeg.audio.enabled` | §D2, §S2 |
| example JSON | §D3, §S4 |
| validation notes (no new rule count) | §D8 |
| streaming arg-vector extension | §D4, §S5 |
| from-config plumbing | §S2/S3/S5 (`audio_enabled` → builder) |
| offline pins (with/without) | §D9, §S6, §S8 |
| docs (configuration.md + production-run.md) | §D12, §S14 |
| standard gate / builds / audit cycle | §D10, §S11–S14, §9 |
| "no real-Google verification claims" | §9 scan + §D5 network prohibition + NO change to any google/youtube boundary; the remedy ships behind code, no claim anywhere of a verified YouTube result |

Honesty ledger: the severity verdict of `noAudioStream` remains UNPROVEN-OFFICIALLY (the operator probe results are the only empirical check; Task-1 §1.6 carried); the pinned container FFmpeg (5.1.9) may differ subtly from the operator's Windows FFmpeg build around the `-shortest`/lavfi capability, so the operator-run preflight line in the probe procedure stays mandatory; the demo vector byte-stability is guaranteed structurally — the ONLY site that consults the audio state is `build_stream_arguments`, while `build_arguments` remains byte-identical (its untouched exact-vector pin is the standing proof).

## 12. Downstream Ownership Map (for the planner — do not cross merge)

| Step | Owner | Content |
| --- | --- | --- |
| 4.2 | implementer | §3 S0–S14; commits §6 |
| 4.3 | code-reviewer + code-simplifier | review the feat diff against §D1–D9 (the vector ORDER, the fixture ALL-THREE shape) + simplifier plans to `.kilo/plans/20261024-silent-audio-task2-{review,simplification}.md` if fixes needed; simplifier edits may never reorder the vector, renumber constants, or touch the fixture/example truth |
| 4.3-fix | implementer | apply fix plans only |
| 4.4 | docs-specialist | drift-check README/demo.md/authentication.md/build*.md/demo configuration docs vs §D12; recheck §9 (audit) if docs touched; commit corrections |
| 4.5b | architector | adherence report vs THIS plan |
| 4.6 | implementer | TODO Task 2 `[DONE]` + acceptance-criteria flips (criterion 3 `[x]`; criterion 5 `[x]` after §S12's build sizes are recorded) |
| 5 | implementer | TODO rename to `-DONE`, hygiene sweep, merge to main, push `origin` ONLY |

## 13. NOT Researched / Consciously Out of Reach (capture in every honesty record)

1. Whether YouTube's CURRENT ingest refuses (vs merely flags) an audio-less stream — the operator probes stay the empirical check; the remedy ships on the documented-expectation basis.
2. Whether the native-aac 128k stereo 48kHz muxed-into-FLV stream is accepted by every PLAYER-side decoder (product-irrelevant: the play surface is Studio/YouTube's live monitor).
3. Whether `-shortest` behaves identically on the OPERATOR's native Windows FFmpeg build (the container pin uses 5.1.9; the docs preflight line covers the operator's side; the remedy defaults ON in the example, not hardwired in code — the config flag gives the operator a 1-line escape hatch).
4. No network dissection of RTMPS handshake internals; no video content redesign (the all-black source stays; only the signal carrier gains audio).

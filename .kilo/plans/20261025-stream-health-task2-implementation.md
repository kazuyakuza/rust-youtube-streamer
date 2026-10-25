# Task 2 Implementation Plan — Stream-Health Remedies: CBR Video + Real-Bitrate MP3 Silent Audio (streaming vector only)

Plan date: 2026-10-25 · Step 4.1b of the Critical Workflow (architector) · Implementer: junior developer under 50% restriction · Front-end: NO (no 4.1a)
TODO task: `.agent/todos/20261024/20261024-todo-2.md` **Task 2 only** (Task 1 is `[DONE]`; do NOT touch Task-1 surfaces)
Global plan (binding): `.kilo/plans/20261025-stop-path-and-stream-health.md` § Encoded Technical Direction (Task 2) + § Risks
Branch: `fix/stop-path-and-stream-health` · HEAD `a96d2b3` at plan-writing time (Task-1 4.4 done) · Version already 0.11.1 (step 3) — NO version change here.

---

## 1. Research verdicts (recorded with citations; binding)

### 1.1 libx264 `nal-hrd=cbr` mechanics — CONFIRM

- Official ffmpeg-codecs documentation (https://manpages.debian.org/bookworm/ffmpeg/ffmpeg-codecs.1.en.html, mirrors ffmpeg.org `ffmpeg-codecs`): `nal-hrd` — "Set signal HRD information (requires vbv-bufsize to be set)"; value `cbr` = "Constant bit rate (not allowed in MP4 container)"; and: "If maxrate and minrate are also set to the same value then it will use constant-bitrate mode, otherwise if crf is set as well then it will use constrained-quality mode." `maxrate` — "Set max bitrate tolerance (in bits/s). Requires bufsize to be set." `minrate` — "Set min bitrate tolerance (in bits/s). Most useful in setting up a CBR encode. It is of little use elsewise." `bufsize` — "Set ratecontrol buffer size (in bits)."
- The standard CBR invocation is therefore exactly `-b:v B -minrate B -maxrate B -bufsize 2×B -x264-params nal-hrd=cbr` (the FFmpeg wiki's CBR recipe, sampled via the search excerpt of https://trac.ffmpeg.org/wiki/Encode/H.264 §CBR: `ffmpeg ... -c:v libx264 -x264-params "nal-hrd=cbr" -b:v 1M -minrate 1M -maxrate 1M -bufsize 2M ...` — "bufsize is the rate control buffer, so it will enforce your requested average across that window"). The `cbr not allowed in .mp4` caveat applies to MP4 only (libx264 "nal-hrd" option help string, official libx264.c source); FLV is NOT MP4, and the container probe below CONFIRMS it works with FLV.
- `force-cfr` interplay: the community variant appends `:force-cfr=1` to force constant-framerate timestamps; it is NOT needed for the bitrate remedy and stays the UNTRIGGERED fallback (per the global plan's encode fallback order).
- **FLV legality — CONFIRMED empirically on the pinned dev image (FFmpeg 5.1.9-0+deb12u1, recorded probe, 2026-10-25)**: encoding with the exact production shape (rawvideo stdin black frames + anullsrc + `-b:v 6000k -minrate 6000k -maxrate 6000k -bufsize 12000k -x264-params nal-hrd=cbr -g 10 -pix_fmt yuv420p` + `-c:a libmp3lame -b:a 128k -shortest -f flv /tmp/probe-cbr-6m.flv`) exited 0; per-stream bit_rate reported `6000000` (video) and `128000` (audio); measured container total `bit_rate=5779767` at `duration=10.408`. The 300k variant behaved identically (`6000000`→`300000` exact). The global plan's FLV×HRD fallback procedure therefore stays UNTRIGGERED.
- **Argument-order fact (learned empirically, matters for the builder)**: all CBR video tokens are OUTPUT options — if any of them sits BEFORE the audio `-f lavfi -i anullsrc…` pair, FFmpeg rejects with "Option b:v … cannot be applied to input url anullsrc …". The production builder naturally places them after (video output args follow the audio input pair); the plan pins that ordering.

### 1.2 MP3-CBR silence + MP3-in-FLV — CONFIRM

- Official ffmpeg-all documentation §15.8 libmp3lame (https://ffmpeg.org/ffmpeg-all.html): option `b` — "Set bitrate expressed in bits/s for CBR or ABR. LAME bitrate is expressed in kilobits/s"; `abr` is OFF by default, so setting `-b:a` alone selects CBR mode. MP3 CBR frames carry the full target payload even on digital silence (silence frames are still full-size constant-rate frames) — this is CBR's defining semantic, and it is empirically pinned here: the container probe's ffprobe reported `codec_name=mp3`, `bit_rate=128000`, `sample_rate=48000`, `channels=2` on a purely silent (`anullsrc`) track (incident AAC shape measured ~2 kbit/s on the same silence: `audio:54KiB`/195 s in `logs/run/20261025-020431.log`).
- MP3-in-FLV legality: the FFmpeg-supported-media-types wiki table (https://trac.ffmpeg.org/wiki/SupportedMediaTypesInFormats) lists FLV audio codecs including "MP3 → libmp3lame (needs the external encoder library lame)"; the official FLV muxer (ffmpeg-formats §21.39 flv) is the "Adobe Flash Video Format muxer" with no mp3 prohibition; the container probe mended it end-to-end (exit 0, ffprobe mp3).
- YouTube acceptance: the official encoder-settings page (https://support.google.com/youtube/answer/2853702 — already cited by the Task-1 research of the previous TODO and recorded in `docs/production-run.md`) recommends audio codec **"AAC or MP3"**, audio bitrate **"128-Kbps for stereo"**, and **bitrate encoding CBR** — both remedy constants sit inside the official recommendation set.
- libmp3lame availability: the pinned dev-checks image has it (empirically listed among `-encoders`; the Dockerfile's Debian `ffmpeg` package enables `--enable-libmp3lame`). The operator's FFmpeg build must include libmp3lame (gyan.dev full builds do) — DOCUMENTED in this task; a build lacking it fails at spawn with the existing clear typed error already classified (`ffmpeg-codecs`: "Encoders requiring an external library must be enabled manually via the corresponding --enable-lib option"; a missing encoder produces FFmpeg's "Unknown encoder" diagnostic → `StreamingError::ExitFailure` diagnostics tail). NO new error variant is added for the libmp3lame prerequisite.

### 1.3 Measured-bitrate verification design — DECIDED

- Reuse the existing real-container mechanism pin's structure (`ffmpeg_real_audio_tests.rs`: local FLV output in a `tempfile::TempDir`, black rawvideo frames through stdin, bounded wait, ffprobe; never a network, never Google).
- ffprobe per-stream `bit_rate` entries are the ENCODER-DECLARED values (`6000000`, `128000` — exact); `format=bit_rate` is the MEASURED container average. On short clips the measured average sits below the declared sum (startup/mux overhead skew): recorded observations — 2.4 s clip: 340665/428000 ≈ 79.7%; 6.4 s: 399633/428000 ≈ 93.4%; 10.4 s at 6000k+128k: 5779767/6128000 ≈ 94.3%.
- Chosen assertion form (encoded, per the caller's candidate): a 10-second pin clip (`SMOKE_FRAME_COUNT` 10 → 50 at 5 fps) with the pinned floors (final shape in §3/S4 — the per-stream AUDIO constant is NOT separately pinned; the measured-total floor subsumes it):
  1. video stream `bit_rate ≥ 0.9 × 6,000,000 = 5_400_000` (encoder postfill exactness; observed exact B),
  2. measured container total `format=bit_rate ≥ 0.9 × (6,000,000 + 128,000) = 5_515_200` (subsumes the audio floor; observed 5779767 — 4.8% headroom),
  3. a duration plausibility bound (`duration` ∈ [8.0, 13.0] s) so a wrongly-`-shortest`-bound or truncated encode cannot fake the floors.
  Tolerance recorded honestly in code comments and docs: the CBR filler makes measured ≈ declared up to mux overhead + startup skew.

### 1.4 Rule caps (verified at plan time)

| File | Now | Planned | Verdict |
|---|---|---|---|
| `src/streaming/stream_args.rs` | 141 | ~176 (+constants, +CBR tokens fn +bufsize helper, doc rewrites) | ≤ 200 ✓ |
| `src/streaming/stream_args_tests.rs` | 168 | ~176 (2 pin re-cuts + 2 derivation tests) | ≤ 200 ✓ |
| `src/streaming/ffmpeg_real_audio_tests.rs` | 169 | ~194 (argv edit + ffprobe entries + floor asserts + counters) | ≤ 200 ✓ (tight — see S8 STOP rule) |
| `src/streaming/error.rs` | 179 | ~189 (+1 variant + Display arm + source arm) | ≤ 200 ✓ |
| `src/streaming/error_tests.rs` | 159 | ~163 (2 assert lines) | ≤ 200 ✓ |
| `src/app/run/streaming_stage_tests.rs` | 99 | ~107 (argv asserts) | ≤ 200 ✓ |
| `src/streaming/ffmpeg_isolation_tests.rs` | 197 | UNTOUCHED | belongs to Task 1 |
| `src/streaming/ffmpeg_tests.rs` | 173 | UNTOUCHED (fake patterns only) | — |

If ANY edited source file breaches 200 lines after rustfmt: STOP and report to the caller (the implementer has no split authority; the caller decides a sibling split per the 2026-10-24 precedent).

---

## 2. Encoded decisions (binding)

- **D1 — CBR video parameters, BOTH streaming vectors.** `stream_video_output_arguments` gains, immediately after `-b:v <bitrate>` and before `-g`: `-maxrate <bitrate>`, `-minrate <bitrate>`, `-bufsize <2×bitrate>`, `-x264-params nal-hrd=cbr`. The audio-OFF vector (section absent / `enabled` absent / `false`) gains the SAME CBR video parameters (video-side health issue; global-plan §Encoded Technical Direction decision), keeping no audio track. The demo/local `build_arguments` vector and `ffmpeg.rs` stay byte-untouched.
- **D2 — New named constants** in `stream_args.rs`'s constants block: `VIDEO_MAXRATE_TOKEN: &str = "-maxrate"`, `VIDEO_MINRATE_TOKEN: &str = "-minrate"`, `VIDEO_BUFSIZE_TOKEN: &str = "-bufsize"`, `VIDEO_X264_PARAMS_TOKEN: &str = "-x264-params"`, `VIDEO_HRD_CBR_PARAM: &str = "nal-hrd=cbr"` (ONE argv token), and `CBR_BUF_SIZE_MULTIPLIER: u32 = 2` (the named multiplier; doc comment: official x264 CBR recipe uses a 2× rate-control buffer — trac wiki §CBR).
- **D3 — buf-size derivation without panic.** New private helper `cbr_bufsize_bitrate(bitrate: &str) -> Result<String, StreamingError>`: finds the leading ASCII-digit run, splits into `numeric` + `suffix` (e.g. `"6000k"` → `"6000"` + `"k"`), parses the numeric part as `u64`, and returns `format!("{}{}", numeric * u64::from(CBR_BUF_SIZE_MULTIPLIER), suffix)`; an EMPTY or non-parseable numeric run returns a typed defensive error — **D4 — new `StreamingError` variant** `UnsupportedBitrateNotation { value: String }` (mirrors the `UnsupportedPixelFormat` defensive precedent; unreachable through the loader in practice): variant field comment ("the validator enforces non-blank only, so a non-numeric bitrate is rejected here defensively"`), Display string EXACTLY:
  `streaming pipeline CBR bufsize derivation requires a numeric-leading bitrate in configuration (got '{value}')`
  `source()` returns `None`; the "no panics" rule holds (never `.expect()` on the parse).
- **D5 — Audio codec swap.** `AUDIO_CODEC: &str = "aac"` → `"libmp3lame"`. `-b:a 128k`, the anullsrc input pair, and `-shortest` are UNCHANGED. No `-map`, no new config fields (rule count stays 21), no dependency or lockfile change.
- **D6 — Verified floor constants** (in `ffmpeg_real_audio_tests.rs`): `VIDEO_CBR_STREAM_BITRATE_FLOOR: u64 = 5_400_000`, `MEASURED_TOTAL_BITRATE_FLOOR: u64 = 5_515_200`, `MIN_MEASURED_DURATION_SECONDS: f64 = 8.0`, `MAX_MEASURED_DURATION_SECONDS: f64 = 13.0`. `SMOKE_FRAME_COUNT` 10 → 50 (the measured-total floor subsumes the audio floor; §1.3/S4.6). Existing bounded-wait constants UNCHANGED (the recorded probes encoded 10 s of filler in ~1 s wall time << the 15 s budget).
- **D7 — Byte-stability ledger:** demo/local vector, `start`/`start_stream` engines, spawn isolation (Task 1), finish/drop/error mappings, the single-`error:`-line contract, diagnostics constants, EVERY YouTube/OAuth/demo/auth surface, config schema (`FfmpegConfig.audio` fields, 21 validation rules), example config, and the Task-1 isolation roster untouched. The absent/false/None audio shapes NO LONGER equal the pre-0.11 wire form (they gain CBR video) — every doc sentence claiming "byte-identical to the pre-0.11 form" is corrected in this task (see §4).

---

## 3. Steps — code and tests (implementer, 4.2)

### S0. Plan tracking commit

1. Read this plan fully; confirm branch `fix/stop-path-and-stream-health` and `git status` clean.
2. Commit ONLY this plan file:
   ```
   git add .kilo/plans/20261025-stream-health-task2-implementation.md
   git commit -m "chore: track stream-health Task 2 implementation plan"
   ```
   (Task-1 precedent: per-task plan-tracking commits `7c6cd5d`/`ecbc325`; caller material.)

### S1. `src/streaming/error.rs` — the defensive bitrate-notation variant

1. In the `StreamingError` enum, after `UnsupportedPixelFormat`, add:

```rust
    /// The configured bitrate is not numeric-leading, so the CBR bufsize
    /// derivation cannot double it. Defensive check mirroring
    /// [`StreamingError::UnsupportedPixelFormat`]; the config validator
    /// accepts non-blank values only, making the reachable probability low
    /// but never zero. The FFmpeg invocation would fail anyway; this variant
    /// fails earlier and more clearly. The `value` field is the raw
    /// configured bitrate text.
    UnsupportedBitrateNotation { value: String },
```

2. In the `Display` match, after the `UnsupportedPixelFormat` arm add:

```rust
            StreamingError::UnsupportedBitrateNotation { value } => write!(
                formatter,
                "streaming pipeline CBR bufsize derivation requires a numeric-leading bitrate in configuration (got '{value}')"
            ),
```

3. In the `Error` impl's `source()` match add `StreamingError::UnsupportedBitrateNotation { .. } => None,` (either arm position; keep the existing 9 arms for the 9 other variants byte-identical).
4. In `src/streaming/error_tests.rs` `format_errors_without_io_source` add (aligned with the other asserts):

```rust
    assert_eq!(
        StreamingError::UnsupportedBitrateNotation {
            value: "6mega".to_string()
        }
        .to_string(),
        "streaming pipeline CBR bufsize derivation requires a numeric-leading bitrate in configuration (got '6mega')"
    );
```

and in `source_returns_io_error_where_defined` add the `UnsupportedBitrateNotation { value: "abc".to_string() }` binding + `assert!(…source().is_none());`.

### S2. `src/streaming/stream_args.rs` — the CBR builder + constants

1. Constants block: keep all existing constants; change `AUDIO_CODEC` to `"libmp3lame"` and append:

```rust
const VIDEO_MAXRATE_TOKEN: &str = "-maxrate";
const VIDEO_MINRATE_TOKEN: &str = "-minrate";
const VIDEO_BUFSIZE_TOKEN: &str = "-bufsize";
const VIDEO_X264_PARAMS_TOKEN: &str = "-x264-params";
const VIDEO_HRD_CBR_PARAM: &str = "nal-hrd=cbr";
/// The official x264 CBR recipe uses a 2× rate-control buffer
/// (ffmpeg wiki Encode/H.264 §CBR; `-bufsize` must exceed `-maxrate`).
const CBR_BUF_SIZE_MULTIPLIER: u32 = 2;
```

2. Rewrite the module header docs: the locked-vector paragraph gains the CBR parameters (both shapes), the MP3 swap, and the citation block (ffmpeg-codecs: maxrate/minrate/bufsize/cbr + "maxrate and minrate equal → constant-bitrate mode" + "cbr not allowed in MP4" (FLV is not MP4); FFmpeg wiki §CBR 2× bufsize; YouTube encoder-settings CBR + "AAC or MP3"/128-Kbps; the empirical FLV probe verdict). Delete/replace every sentence that now contradicts D1/D7 (the audio-off "pre-0.11 byte-identical" phrasing becomes "video-only, no audio track, carrying the same derived CBR video parameters").
3. Change `stream_video_output_arguments` to a Result-returning function that inserts the CBR block after `-b:v` (returning type `Result<Vec<String>, StreamingError>`; call site adds `?`):

```rust
fn stream_video_output_arguments(config: &AppConfig) -> Result<Vec<String>, StreamingError> {
    let cbr_bufsize = cbr_bufsize_bitrate(&config.ffmpeg.bitrate)?;
    let mut arguments = vec![
        "-c:v".to_string(),
        config.ffmpeg.video_codec.clone(),
        "-preset".to_string(),
        config.ffmpeg.preset.clone(),
        "-b:v".to_string(),
        config.ffmpeg.bitrate.clone(),
        VIDEO_MAXRATE_TOKEN.to_string(),
        config.ffmpeg.bitrate.clone(),
        VIDEO_MINRATE_TOKEN.to_string(),
        config.ffmpeg.bitrate.clone(),
        VIDEO_BUFSIZE_TOKEN.to_string(),
        cbr_bufsize,
        VIDEO_X264_PARAMS_TOKEN.to_string(),
        VIDEO_HRD_CBR_PARAM.to_string(),
        "-g".to_string(),
        format!("{}", stream_keyframe_interval(config.video.fps)),
        "-pix_fmt".to_string(),
        OUTPUT_PIXEL_FORMAT.to_string(),
    ];
    Ok(arguments)
}
```

4. Add the derivation helper (single behavioural block; ≤ 2 nesting depths):

```rust
fn cbr_bufsize_bitrate(bitrate: &str) -> Result<String, StreamingError> {
    let digits_end = bitrate
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(bitrate.len());
    let (numeric_part, suffix) = bitrate.split_at(digits_end);
    let numeric: u64 = numeric_part
        .parse()
        .map_err(|_| StreamingError::UnsupportedBitrateNotation { value: bitrate.to_string() })?;
    Ok(format!("{}{}", numeric * u64::from(CBR_BUF_SIZE_MULTIPLIER), suffix))
}
```

5. Update `build_stream_arguments` call site: `arguments.extend(stream_video_output_arguments(config)?);`. NO other ordering change: the audio input pair remains right after the video input pair; then audio output args; the `-f <STREAM_MUXER> <url>` final pair is the lock (the empirical probe proved the CBR tokens all land after the audio input pair, so the parse-rejection shape in §1.1 is structurally impossible).

### S3. `src/streaming/stream_args_tests.rs` — the re-cut vector pins

1. Update the doc-comment line if it names AAC or the pre-0.11 identical wire form.
2. Rename `stream_arguments_match_the_locked_output_shape` → `stream_arguments_without_audio_carry_the_cbr_video_parameters` (same body shape; keep `audio = None`; now expect EXACTLY this 33-token vector — token positions of the tail asserts (`-movflags` absence, `length-3 == "-f"`, `length-2 == "flv"`) are unchanged):

```rust
    assert_eq!(
        arguments,
        vec![
            "-nostdin",
            "-n",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            "1280x720",
            "-framerate",
            "25",
            "-i",
            "pipe:0",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-b:v",
            "6000k",
            "-maxrate",
            "6000k",
            "-minrate",
            "6000k",
            "-bufsize",
            "12000k",
            "-x264-params",
            "nal-hrd=cbr",
            "-g",
            "50",
            "-pix_fmt",
            "yuv420p",
            "-f",
            "flv",
            DESTINATION_FIXTURE_URL,
        ]
    );
```

3. Update `stream_arguments_with_audio_match_the_locked_extended_shape` to expect EXACTLY this 42-token vector (the `arguments[11..16]` audio-input-window assert keeps its indexes — the insertion is after the input pair):

```rust
    assert_eq!(
        arguments,
        vec![
            "-nostdin",
            "-n",
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            "-s",
            "1280x720",
            "-framerate",
            "25",
            "-i",
            "pipe:0",
            "-f",
            "lavfi",
            "-i",
            "anullsrc=channel_layout=stereo:sample_rate=48000",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-b:v",
            "6000k",
            "-maxrate",
            "6000k",
            "-minrate",
            "6000k",
            "-bufsize",
            "12000k",
            "-x264-params",
            "nal-hrd=cbr",
            "-g",
            "50",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "libmp3lame",
            "-b:a",
            "128k",
            "-shortest",
            "-f",
            "flv",
            DESTINATION_FIXTURE_URL,
        ]
    );
```

4. `audio_absent_false_and_unset_enabled_hold_the_identical_video_only_vector`: UNTOUCHED (the equality proof still holds — all three shapes now produce the CBR video-only vector). Only its doc phrase if the docs name "pre-0.11".
5. Add two tests (grouped right after the with-audio pin, named per the repo's explicit style):

```rust
#[test]
fn cbr_bufsize_doubles_the_configured_bitrate_keeping_the_suffix() {
    assert_eq!(
        cbr_bufsize_bitrate("6000k")
            .expect("6000k must double to 12000k"),
        "12000k"
    );
    assert_eq!(
        cbr_bufsize_bitrate("6M").expect("6M must double to 12M"),
        "12M"
    );
    assert_eq!(
        cbr_bufsize_bitrate("5000000").expect("a suffixless bitrate doubles in bits"),
        "10000000"
    );
}

#[test]
fn cbr_bufsize_non_numeric_bitrate_fails_as_typed_notation_error() {
    let notation = cbr_bufsize_bitrate("6mega")
        .expect_err("non-numeric bitrate must fail at derivation time");
    assert_eq!(
        notation.to_string(),
        "streaming pipeline CBR bufsize derivation requires a numeric-leading bitrate in configuration (got '6mega')"
    );
}
```

6. `stream_gop_is_two_frames_per_second` and the `gop_of` helper UNTOUCHED (note: it now also asserts on the CBR-bearing vectors — the `-g` position look-up still works).

### S4. `src/streaming/ffmpeg_real_audio_tests.rs` — the real-container bitrate-verification pin

1. Rename the test to the truthful remedied scope: `real_ffmpeg_stream_reaches_the_configured_bitrate_with_cbr_video_and_mp3_silence`; update the module and test doc comments (the pin now verifies the measured bitrate ~ the configured target; `-shortest` bounding, h264 + mp3 48 kHz stereo).
2. `SMOKE_FRAME_COUNT` 10 → 50 and add the D6 floor/duration constants; the constants' doc lines record §1.3 honestly (measured ≈ declared up to mux overhead + startup skew; floor = 90% of the declared sum).
3. In `spawn_audio_smoke`'s arg list, after `-b:v`, `6000k`, insert:
   `"-maxrate", "6000k", "-minrate", "6000k", "-bufsize", "12000k", "-x264-params", "nal-hrd=cbr",` and change `-c:a`,`"aac"` → `"-c:a","libmp3lame"`. NOTHING else changes (input pairs, `-g 10`, `-pix_fmt`, `-shortest`, `-f flv`).
4. Extend the ffprobe entries — replace `"stream=codec_type,codec_name,sample_rate,channels",` with `"stream=codec_type,codec_name,sample_rate,channels,bit_rate:format=bit_rate,duration",`.
5. Add a file-local helper mirroring `probe_value`'s style, reading the LAST matching value (the recorded ffprobe output order is: stream entries first, then the `format=` block last, so the FIRST `bit_rate=` is the video stream and the LAST `bit_rate=`/`duration` are the container-level `format=` entries):

```rust
fn probe_last_value<'a>(probe_stdout: &'a str, key: &str) -> Option<&'a str> {
    let prefix = format!("{key}=");
    probe_stdout
        .lines()
        .rev()
        .find_map(|line| line.strip_prefix(prefix.as_str()))
        .map(str::trim)
}
```

6. Rename `assert_smoke_streams_carry_h264_and_aac` → `assert_smoke_streams_carry_h264_and_mp3_at_the_measured_floors` with EXACTLY this assertion shape:

```rust
fn assert_smoke_streams_carry_h264_and_mp3_at_the_measured_floors(probe_stdout: &str) {
    let codec_names: Vec<&str> = probe_stdout
        .lines()
        .filter_map(|line| line.strip_prefix("codec_name="))
        .collect();
    assert!(
        codec_names.contains(&"h264"),
        "the video stream must be h264: {probe_stdout}"
    );
    assert!(
        codec_names.contains(&"mp3"),
        "the audio stream must be mp3: {probe_stdout}"
    );
    assert_eq!(probe_value(probe_stdout, "sample_rate"), Some("48000"));
    assert_eq!(probe_value(probe_stdout, "channels"), Some("2"));
    let video_bitrate: u64 = probe_value(probe_stdout, "bit_rate")
        .expect("ffprobe must report the stream bit_rate")
        .parse()
        .expect("the stream bit_rate must be numeric");
    assert!(
        video_bitrate >= VIDEO_CBR_STREAM_BITRATE_FLOOR,
        "the CBR-filled video stream must reach the encoder-declared floor: {video_bitrate}"
    );
    let measured_total: u64 = probe_last_value(probe_stdout, "bit_rate")
        .expect("ffprobe must report the container bit_rate")
        .parse()
        .expect("the container bit_rate must be numeric");
    assert!(
        measured_total >= MEASURED_TOTAL_BITRATE_FLOOR,
        "the measured container bitrate must reach the floor: {measured_total}"
    );
    let measured_duration: f64 = probe_last_value(probe_stdout, "duration")
        .expect("ffprobe must report the container duration")
        .parse()
        .expect("the container duration must be numeric");
    assert!(
        (MIN_MEASURED_DURATION_SECONDS..=MAX_MEASURED_DURATION_SECONDS).contains(&measured_duration),
        "expected ~10.4 s of filler measurement, got {measured_duration}"
    );
}
```

   Floor arithmetic (recorded honestly in the constants' doc comments): `VIDEO_CBR_STREAM_BITRATE_FLOOR = 0.9 × B = 0.9 × 6_000_000`, `MEASURED_TOTAL_BITRATE_FLOOR = 0.9 × (B + audio) = 0.9 × (6_000_000 + 128_000)` — this arithmetic SUBSUMES the audio floor (an audio stream short of ~115_200 kbit/s-equivalent contribution would drag the measured total below the floor); the per-stream audio 128000 value rides as the recorded empirical evidence in the floor comment (`AUDIO_CBR_STREAM_BITRATE_FLOOR: u64 = 115_200` is therefore NOT defined as a standalone pinned constant — do not create unused constants).
7. The `wait_audio_smoke_bounded`/frame-writing/d10/process-group mechanics UNTOUCHED except the frame count.

### S5. `src/app/run/streaming_stage_tests.rs` — production argv pin additions

In `stream_pipeline_passes_the_destination_to_the_ffmpeg_argv` (the test count stays 1), AFTER the existing `-shortest` assert add:

```rust
    assert!(
        argv.windows(2)
            .any(|pair| pair[0] == "-maxrate" && pair[1] == "6000k"),
        "the CBR maxrate mirrors the configured bitrate"
    );
    assert!(
        argv.windows(2)
            .any(|pair| pair[0] == "-minrate" && pair[1] == "6000k"),
        "the CBR minrate mirrors the configured bitrate"
    );
    assert!(
        argv.windows(2)
            .any(|pair| pair[0] == "-bufsize" && pair[1] == "12000k"),
        "the CBR bufsize doubles the configured bitrate (2× multiplier)"
    );
    assert!(
        argv.windows(2)
            .any(|pair| pair[0] == "-x264-params" && pair[1] == "nal-hrd=cbr"),
        "the x264 HRD filler mechanism signals constant bitrate mode"
    );
```

and UPDATE the existing codec pair assert `pair[0] == "-c:a" && pair[1] == "aac"` → `"-c:a", "libmp3lame"` with the message "the silent track is encoded as MP3 CBR (silence still carries full bitrate frames)". The module doc gets one sentence: the streaming argv now carries the derived CBR video parameters (both audio shapes) and MP3-CBR silent audio.

### S6. Doc-comment corrections in config model doc text (NOT logic)

In `src/config/model.rs` ONLY doc-comment edits (no code change):
- `FfmpegConfig.audio` doc: replace "keeps the video-only argument vector byte-identical to the pre-0.11 form" with "keeps the streaming vector without any audio track (it still carries the derived CBR video parameters)".
- `AudioConfig.enabled` doc: replace "keeps the stream video-only" with "keeps the stream without any audio track (the CBR video parameters stay)".

### S7. Format + targeted spot-checks (container)

Exactly these commands via the standard Compose invocation on the Alpine VM (`docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cmd>`, single invocations, no chained commands):
1. `cargo fmt --check` (exit 0 required; on failure run rustfmt normalization then re-run — never stage a hand-broken tree).
2. `cargo test --locked stream_args` — expect 6 passed (2 re-cut vector pins, 1 equality proof, 1 GOP, 2 derivation).
3. `cargo test --locked streaming::error` — expect 4 passed.
4. `cargo test --locked ffmpeg_real` — expect 2 passed (UNTOUCHED smoke + renamed bitrate pin).
5. `cargo test --locked streaming_stage` — expect 1 passed (unix-only; on a Windows shell, this roster is gated — verification happens inside the container).
6. `cargo clippy --locked -- -D warnings` — 0 warnings required (S4.6 already encodes the no-unused-constant rule; nothing should fire).

### S8. Full gate + builds + audits (whole-TODO criterion evidence)

1. FULL gate: `sh scripts/dev-checks.sh` in the recorded standard form — expect exit 0 with `ALL CHECKS PASSED` and the observed test count: **baseline 389 + 2 derivation pins = 391 EXPECTED; the OBSERVED count is authoritative** (recorded honestly if 4.3 later adds/removes).
2. Builds: `sh scripts/build-linux.sh` then `sh scripts/build-windows.sh` — record both printed sizes, then `ls -l /rust-youtube-streamer/dist /rust-youtube-streamer/dist/windows` cross-checks byte equality.
3. Hygiene/secret audit:
   - `git check-ignore -v dist/` (`.gitignore:34:dist/`); `git ls-files dist/ videos/` empty;
   - `git status` shows ONLY the task files;
   - `ya29`, `AIza`, `key=` word-count audit (zero new attribute/data hits beyond the pre-existing doc-context counts);
   - `allow(` in `src/` — zero new attributes;
   - `-c:a aac` token zero-hits across `src/` and `config/` docs after the swap (except any historical git history).

### STOP-and-report conditions (any single one)

1. **FLV × `nal-hrd=cbr` incompatibility**: if a real-container encode with the CBR tokens to FLV fails on the pinned image (unlike the recorded planning probes), the encoded fallback order from the global plan applies: try `nal-hrd=cbr:force-cfr=1` variants; if none works with FLV, STOP and report to the planner (the audio remedy alone may still ship).
2. Any edited source file exceeding 200 lines after rustfmt (§1.4) — STOP and report; no unauthorized sibling split.
3. The real-container pin floors failing beyond the recorded tolerance (5779767 observed vs 5515200 floor ⇒ 4.8% margin measured) — re-run once; if still failing, capture the ffprobe output and STOP (never weaken the floor without the caller).
4. Any ambiguity outside the plan's explicit text — STOP and ask the caller.

---

## 4. Docs and metadata steps (S9; docs-specialist-equivalent work inside the same 4.2 cycle, caller-ordered docs commit precedent)

### S9.1 `docs/configuration.md`

Line 81 `bitrate` bullet: append one sentence: "The streaming vector derives its CBR parameters from this value: `-maxrate <bitrate> -minrate <bitrate> -bufsize <2×bitrate> -x264-params nal-hrd=cbr` (the x264 constant-bit-rate filler; YouTube's own encoder-settings guidance recommends CBR)."
Line 82–83 `audio` bullet: replace the sentence pair so the audio bullet says: silent track encoded by `libmp3lame` at 128 kbit/s (the officially accepted alternative audio codec; sits inside the official recommendation "AAC or MP3"; MP3-CBR pads silence to the real bitrate — the AAC-on-silence shape cannot be padded), and the "byte-identical to the pre-0.11 form" clause becomes "carries the same derived CBR video parameters (video-side health remedy applied to both audio shapes)"; add the libmp3lame prerequisite sentence: "the operator's FFmpeg build must include libmp3lame (the standard full builds do); a build lacking it fails at stream start with the clear spawn-error path." Keep ALL other sentences (config rules count 21 etc.) verbatim.

### S9.2 `docs/production-run.md`

Replace the "**Product-level status (v0.11.0):**" paragraph with a "**Product-level status (v0.11.1):**" paragraph stating, honestly:
- the streaming vector now enforces true CBR video (maxrate/minrate/bufsize derived from the configured bitrate + the x264 `nal-hrd=cbr` filler) and encodes the silent track as MP3-CBR at 128 kbit/s — addressing the measured-bitrate shape of the two Studio health warnings ("not receiving enough video..." characterized at ~18.5 kbit/s vs the 6000k stance; "audio bitrate 0 → recommended 128 Kbps" at ~2 kbit/s on silence);
- what the remedy does NOT change: the video CONTENT stays black/filler-padded, the anullsrc audio is lossless silence, and any further Studio advisory seeded on content movement (Google's tests "should include audio and movement") may remain cosmetic;
- the libmp3lame prerequisite sentence (operator FFmpeg builds) with a pointer to `docs/configuration.md`;
- verification honesty: the mechanism is pinned by a local-FLV ffprobe floor gate (measured avg ≈ declared; mux overhead + startup skew inside the recorded tolerance); the real remedy evidence remains the OPERATOR's next run against real Studio (no real-Google claim anywhere). Keep the rest of the paragraph (the supersedes sentence + diagnosis path) intact.

### S9.3 Metadata refreshes

1. `.agent/project-structure.md`: update the `stream_args.rs` bullet (constants + CBR tokens + the `UnsupportedBitrateNotation` variant + "both audio shapes carry the CBR video parameters; 42-token/33-token pinned vectors"), the `stream_args_tests.rs` bullet (6 tests), the `ffmpeg_real_audio_tests.rs` bullet (renamed bitrate-verification pin), `error.rs` bullet (10 variants), `error_tests.rs` bullet stays (4 tests), the `*_tests.rs` aggregate line for streaming if it names rosters (re-grep and refresh truthfully), the config `model.rs` bullet doc note if it cites byte-identical video-only (it does — update it), `streaming_stage_tests` count bullet (1 test, extended asserts). Version label already v0.11.1 from Task 1.
2. `.agent/project-info/tech.md`: extend the audio-remedy bullet with a CBR/MP3 note (constants, the derived-no-new-field decision, libmp3lame prerequisite, FLV-compat verification evidence pointer).
3. `.agent/project-info/CONTEXT.md`: a new `## Recent Changes (2026-10-25, stream-health Task 2 …)` block (recorded only AFTER implementation and verification, capturing the delta above with the exact evidence).
4. `README.md`: possible test-count drift (389 → the observed count on line 46) and the FFmpeg-prerequisite paragraph gain the libmp3lame note (the ONE new sentence); NO other claims changed.
5. `docs/demo.md`: NO edit (demo stays audio-less and vector-byte-stable; verify with a `grep` for `aac|anullsrc|CBR|libmp3lame` — the only permitted hit is its existing "no audio" statement).
6. Drift-check (no-edit expected unless a claim contradicts): `docs/authentication.md`, `docs/build.md`, `docs/build-windows.md`, `docs/google-cloud-prerequisites.md`, `config/config.example.json` (the `audio` section unchanged — the codec is UNCONFIGURABLE, exactly as before).

---

## 5. Commit plan (implementer, after §3)

1. `chore: track stream-health Task 2 implementation plan` (S0)
2. `fix: stream CBR video and real-bitrate MP3 silence` — `src/streaming/stream_args.rs`, `stream_args_tests.rs`, `error.rs`, `error_tests.rs`, `ffmpeg_real_audio_tests.rs`, `src/app/run/streaming_stage_tests.rs`, `src/config/model.rs` doc lines (S1–S7)
3. `docs: record CBR video and MP3 streaming-remedy health guidance` (S9; AFTER the full gate so the README count is the observed truth; Task-1 precedent: docs edits may ride with 4.6 depending on the caller's rulings — the 4.3/4.4 passes capture the exact hashes)

## 6. Explicitly out of scope (never touched)

`demo`/local vector + `build_arguments`, `ffmpeg.rs` engines, `spawn.rs` isolation (Task 1), ChatFreeSource/source seam, YouTube/OAuth types, config schema fields, `validation.rs` (21 rules), Cargo.toml/Cargo.lock, example config, `ffmpeg_isolation_tests.rs` (Task-1 roster), the Task-1 diagnostics lines, version bump (already 0.11.1).

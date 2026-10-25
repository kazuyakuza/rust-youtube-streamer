# Task 2 Adherence Report — Stream-Health Remedies: CBR Video + Real-Bitrate MP3 Silent Audio (step 4.5b)

Report date: 2026-10-25 · Architector (read-only verification; only this report written)
Scope: branch `fix/stop-path-and-stream-health`, Task-2 chain `586b813` (plan) → `4616a83` (feat, 10 files) → `8120579` (docs, 6 files); task file `.agent/todos/20261024/20261024-todo-2.md` Task 2; binding plan `.kilo/plans/20261025-stream-health-task2-implementation.md` (D-decisions, §S1–S8, STOP conditions); global plan `.kilo/plans/20261025-stop-path-and-stream-health.md` (§ Encoded Technical Direction Task 2 + § Risks).

## Verdict

**ADHERENT WITH ACCEPTABLE DEVIATIONS** — zero NON-ADHERENT items. Every deviation from the binding plan's literal text is one of the caller's 7 recorded rulings (§2 below) or a recorded caller-ruling dependent. Task 2's owned acceptance criteria (3 and 4) are fully evidenced and ready to flip at 4.6.

## 1. Code-level verification

### 1.1 Plan D-decisions vs the tree (`git diff 586b813 8120579 -- src/` inspected hunk-by-hunk)

| Decision | Tree evidence | Verdict |
|---|---|---|
| D1 — CBR block in BOTH streaming vectors | `stream_video_output_arguments` now returns `Result<Vec<String>, StreamingError>` and inserts `-maxrate/-minrate/-bufsize/-x264-params nal-hrd=cbr` immediately after `-b:v <bitrate>` and before `-g`; the audio-OFF pin (audio = None, doc-comment "pins the audio-off video-only shape deliberately") expects the same CBR camera tokens | ADHERENT |
| D2 — named constants | `VIDEO_MAXRATE_TOKEN`/`VIDEO_MINRATE_TOKEN`/`VIDEO_BUFSIZE_TOKEN`/`VIDEO_X264_PARAMS_TOKEN`/`VIDEO_HRD_CBR_PARAM="nal-hrd=cbr"`/`CBR_BUF_SIZE_MULTIPLIER: u32 = 2` present verbatim in `stream_args.rs`'s constants block, with the prescribed doc comment | ADHERENT |
| D3 — panic-free bufsize derivation | private `cbr_bufsize_bitrate`: leading-ASCII-digit split (`find(!is_ascii_digit)` + `unwrap_or(len)` → `split_at`), `u64::parse` with `map_err(UnsupportedBitrateNotation)`, `numeric * u64::from(CBR_BUF_SIZE_MULTIPLIER)` + suffix; never `.expect()`; ≤ 2 nesting depths | ADHERENT |
| D4 — `UnsupportedBitrateNotation` | 10-variant enum; variant + doc comment placed after `UnsupportedPixelFormat`; Display EXACT "streaming pipeline CBR bufsize derivation requires a numeric-leading bitrate in configuration (got '{value}')"; `source() → None`; other 9 arms byte-unchanged (diff = pure insertions: 13+/0−) | ADHERENT |
| D5 — codec swap | `AUDIO_CODEC: "aac"` → `"libmp3lame"`; `-b:a 128k`, the anullsrc input pair, `-shortest`, no `-map`, no new config field, no dependency/lockfile change (Cargo.toml/Cargo.lock absent from the diff) | ADHERENT |
| D6 — floors | `SMOKE_FRAME_COUNT` 10 → 50; `VIDEO_CBR_STREAM_BITRATE_FLOOR = 5_400_000`, `MEASURED_TOTAL_BITRATE_FLOOR = 5_515_200`, duration ∈ [8.0, 13.0] s; no standalone audio-floor constant (no unused constants); the per-stream audio 128000 value rides as the recorded evidence in the helper's tolerance doc comment | ADHERENT WITH ACCEPTABLE DEVIATION (constants live in `ffmpeg_real_probe_helpers.rs`, not `ffmpeg_real_audio_tests.rs`, per caller ruling 3) |
| D7 — byte-stability ledger | numstat 586b813→8120579 over `src/` touches EXACTLY the 10 declared files; `git diff HEAD` shows only `.agent/project-info/context.md` (the 4.4 record). Demo/local vector: `args.rs`, `video_output.rs`, demo module — zero hits. `args.rs`, `spawn.rs`, `pipe_drain.rs`, `sink.rs`, `rtmps_destination.rs`, `local_finalize.rs`, `ffmpeg_isolation_tests.rs`, the whole `app/auth/`, `app/demo/`, `app/pipeline/`, `youtube/`, `config/` (minus model.rs docs) — untouched. `ffmpeg.rs` = +4 lines: ONLY the two `#[cfg(all(test, unix))] #[path] mod` registrations (`ffmpeg_real_probe_helpers`); engines and `StreamExit`/`StreamingError` Display strings byte-stable. Example config, `validation.rs` (21 rules), diff absent | ADHERENT |

### 1.2 §S3 exact token arrays vs the tree

- Audio-OFF pin `stream_arguments_without_audio_carry_the_cbr_video_parameters`: the diff insertion places `-maxrate 6000k -minrate 6000k -bufsize 12000k -x264-params nal-hrd=cbr` exactly between `-b:v 6000k` and `-g 50`, before `-pix_fmt yuv420p -f flv <DESTINATION_FIXTURE_URL>` — byte-matching the plan's 33-token array (renamed from `stream_arguments_match_the_locked_output_shape` per S3.2).
- With-audio pin `stream_arguments_with_audio_match_the_locked_extended_shape`: same CBR block plus `-c:a libmp3lame` replacing `-c:a aac`, `-b:a 128k -shortest -f flv <url>` intact — byte-matching the plan's 42-token array; the `arguments[11..16]` audio-input-window assert keeps its indexes (the insertion sits after the input pair).
- S3.4 untouched-test verdict confirmed: `audio_absent_false_and_unset_enabled_hold_the_identical_video_only_vector` not in the diff (still holds; all three shapes now the CBR video-only vector).
- S3.6: `stream_gop_is_two_frames_per_second`/`gop_of` untouched ✓.

### 1.3 Empirical floors + variant Display + helper shape

- Floors re-derived: 0.9 × 6,000,000 = 5,400,000 ✓; 0.9 × (6,000,000 + 128,000) = 5,515,200 ✓; matches the recorded probe (5,779,767 at 10.408 s ⇒ 4.8% headroom); duration [8.0, 13.0] s plausibility bound present, asserting the compound `(MIN..=MAX).contains(&d)` as one named-range condition (single-section, two named bounds).
- Variant Display: `error_tests.rs` pins the exact Display string with the `"6mega"` fixture (line 25/28) and `source()` is-none with `"abc"` — matches S1.4 verbatim.
- `ffmpeg_real_probe_helpers.rs`: 67 lines; `probe_last_value` (reverse-scan last-match, trims) + `assert_smoke_streams_carry_h264_and_mp3_at_the_measured_floors` at `pub(super)` + the 4 constants; NO `#[test]` inside (structurally not a test roster but a test-only helper module, wired `#[cfg(all(test, unix))]` from `ffmpeg.rs`); `ffmpeg_real_audio_tests.rs` keeps ONE test (renamed `real_ffmpeg_stream_reaches_the_configured_bitrate_with_cbr_video_and_mp3_silence`), keeps `require_image_tool` import, and its ffprobe `-show_entries` string carries the prescribed `bit_rate:format=bit_rate,duration` extension; the argv's CBR insert and codec swap in `spawn_audio_smoke` match S4.3; input pairs, `-g 10`, `-shortest`, `-f flv` untouched; bounded-wait constants unchanged.
- Consumer shape: `stream_args_health_tests.rs` (31 lines) consumes the sibling's PRIVATE helpers directly via `use super::*` — works because the sibling registration makes it one module subtree; the 2 tests' bodies match S3.5 modulo ruling 6 (§2).
- `streaming_stage_tests.rs`: the 4 new `windows(2)` asserts appended AFTER the existing `-shortest` assert, and the `-c:a` pair assert updated to `libmp3lame` with the prescribed message — matches S5 verbatim (test count stays 1).

### 1.4 Rule caps — fresh recount of all 10 files (byte-count lines, ReadAllLines)

| File | Lines | Cap |
|---|---|---|
| `src/streaming/stream_args.rs` | 193 | ≤ 200 ✓ (ruling 1: 193) |
| `src/streaming/stream_args_tests.rs` | 184 | ≤ 200 ✓ (ruling 2: 184) |
| `src/streaming/stream_args_health_tests.rs` | 31 | ≤ 200 ✓ |
| `src/streaming/ffmpeg_real_audio_tests.rs` | 165 | ≤ 200 ✓ (ruling 3: 165) |
| `src/streaming/ffmpeg_real_probe_helpers.rs` | 67 | ≤ 200 ✓ |
| `src/streaming/error.rs` | 192 | ≤ 200 ✓ |
| `src/streaming/error_tests.rs` | 170 | ≤ 200 ✓ |
| `src/app/run/streaming_stage_tests.rs` | 121 | ≤ 200 ✓ |
| `src/config/model.rs` | 176 | ≤ 200 ✓ |
| `src/streaming/ffmpeg.rs` | 161 | ≤ 200 ✓ |

Hygiene re-verified on the tree: `allow(` in `src/` = 8 attribute lines + 1 comment mention (youtube/mod.rs line 68) — the pre-existing inventory, ZERO new attributes; `aac` zero hits in `src/` (only `error_tests.rs`'s `"6mega"` Display-fixture lines remain, matching the ruling-6 split); no commented-out code introduced (all diff hunks are code/docs/tests, none commented-out bodies); secret-free sources (only the pre-existing `…example.test…` fixture URL).

## 2. The 7 caller rulings — adjudication

| # | Ruling | Class | Evidence |
|---|---|---|---|
| 1 | `stream_args.rs` doc re-wrap 193 | ADHERENT WITH ACCEPTABLE DEVIATION | Singlepliz prescribed line insufficient, so the caller re-wrapped header prose; file exactly 193 ≤ 200; all D2 constants + module cliffs land; STOP-2 never breached |
| 2 | `stream_args_health_tests.rs` split (184/31; private-const consumption) | ADHERENT WITH ACCEPTABLE DEVIATION | Split follows the 2026-10-24 precedent (caller-only split authority, exercised by the caller); 31-line roster consumes the sibling's private consts/fns via `use super::*`; both registered via `#[cfg(test)] #[path]` from `stream_args.rs` |
| 3 | `ffmpeg_real_probe_helpers.rs` extraction (165/67; assert fn pub(super), NO #[test]) | ADHERENT WITH ACCEPTABLE DEVIATION | 165/67 exactly as ruled; the assert fn is the only `pub(super)` item; zero `#[test]` (keeps clippy dead-locks off both files); `ffmpeg_real_audio_tests.rs` keeps exactly 1 test; D6 constants MOVED into the helper (file-local, no visibility gain) — this is the ruling's own trade |
| 4 | D1 `let arguments` | ADHERENT WITH ACCEPTABLE DEVIATION | `stream_video_output_arguments` binds `let arguments` non-mut and returns `Ok(arguments)`; the plan's `mut` would fail `unused_mut` under the crate's own `-D warnings` gate — zero-instance deviation |
| 5 | D2 model.rs MP3 doc | ADHERENT WITH ACCEPTABLE DEVIATION | `AudioConfig.enabled` doc: "silent MP3 audio track" replacing "silent AAC audio track"; `FfmpegConfig.audio` doc gains the "still carries the derived CBR video parameters" claim; doc-comment-only, no logic |
| 6 | The "mega" test-input correction | ADHERENT | The plan's own S3.5 example (`expect_err("6mega")`) contradicted binding D3: `"6mega"` has a leading-digit run that PARSES ⇒ it must yield `Ok("12mega")`, so the literal test could not pass the plan's own arithmetic. Correcting the test INPUT to `"mega"` restores fidelity to D3 (the higher-authority decision); test name, typed-error contract and Display line unchanged; `error_tests.rs` keeps `"6mega"` as a pure Display fixture (no derivation involvement) |
| 7 | S9's 2 plan-authorized production-run drift corrections | ADHERENT WITH ACCEPTABLE DEVIATION | (i) Probe i's options sentence now records "this probe predates the v0.11.1 derived CBR block … and omits it" (the probe command stays a historical training-week fact, no longer claimed to "mirror exactly"); (ii) probe ii's heading now reads "the v0.11.0 candidate remedy's wire shape" — both are honest drift corrections preventing contradictions with the new README/production-run CBR claims; authorized and truthful |

Minor recorded note (no action required): the feature commit `4616a83` message reads `feat: force CBR video and real-bitrate silent audio on the streaming vector` where plan §5.2 sketched the `fix:` preamble — a commit-preamble wording drift with no tree effect; recorded for the ledger.

## 3. The 4.4 edits truthfulness (the uncommitted file)

`git status` shows exactly ONE modified file: `.agent/project-info/context.md` (UNCOMMITTED — belongs to 4.6's commits; flagged for the 4.6 caller). Its uncommitted delta is ONE bullet added to the 2026-10-25 stream-health Task-2 block (context.md lines ~239): the 4.3 review + simplification + 4.4 docs-alignment pass record.
Truthfulness check against the tree, point by point:
- code-reviewer "no fixes required" — consistent with §1.1/1.2 above (every binding shape re-verified independently by this report).
- code-simplifier "not required" — consistent (no simplification commit exists in the chain `git log`; zero code edits after 4616a83).
- The deferred hardening candidate is stated FACTUALLY: the u64 multiply in `cbr_bufsize_bitrate` can wrap for parse values at/above the 2^63 scale (true for a pathological config), correctly tagged unreachable-in-practice and explicitly NOT a Task-2 fix; the report-only disposition respects the plan's typed-error decision.
- The verify-only claims (structure-map roster numbers 25/12/13; README 391; the 21-rule note; demo.md zero codec hits) — re-verified TRUE by this report's own greps/counts.
- No secret material: the bullet names no key/URL/token; the health-warning numbers trace to the operator-supplied log fact already attributed in the committed record.
Verdict: the 4.4 edits are TRUTHFUL.

## 4. Criteria matrix — Task 2 bullets + the Task-2-owned acceptance criteria

| TODO bullet / criterion (Task 2) | Tree evidence | State |
|---|---|---|
| Bullet 1 — Video CBR: derive `-maxrate`/`-minrate`/`-bufsize` (bufsize = 2×, named constant) + `nal-hrd=cbr` filler; real-container pin proving measured ≈ target with honest tolerance | D1–D3 lands (§1.1); `ffmpeg_real_probe_helpers.rs` floor gate + named `CBR_BUF_SIZE_MULTIPLIER`; tolerance recorded in the helper's doc comment (5,779,767/6,128,000 ≈ 94.3% ⇒ floors at 90%) | EVIDENCED |
| Bullet 2 — Audio: MP3 CBR `-c:a libmp3lame -b:a 128k`; libmp3lame prerequisite documented; build-lacking-it = the existing typed error | D5 lands (`AUDIO_CODEC` swap; no new error variant per plan §1.2); prerequisite sentences in `docs/configuration.md` (audio bullet) + `docs/production-run.md` status paragraph + README's prerequisite sentence | EVIDENCED |
| Bullet 3 — Update pinned vectors/argv tests + real-container pin; docs: streaming-vector doc lines + production-run health guidance (what CBR/MP3 fix and what remains cosmetic) + context record | S3 re-cuts (33/42-token) + the 2 new derivation pins + S5 argv asserts + S4 rename/floors; `docs/configuration.md` bitrate/audio bullets; `docs/production-run.md` v0.11.1 status paragraph incl. "what does NOT change" (black/filler content, lossless silence, cosmetic-movement advisory), the verification-honesty note (mechanism pinned ONLY by the local-FLV floor gate; real remedy evidence = OPERATOR's next Studio run; no agent-verified Studio claim); context.md 2026-10-25 stream-health block (committed 4.2 portion + the uncommitted 4.4 bullet) | EVIDENCED (context bullet commit scheduled 4.6) |
| Criterion 3 — streaming vector carries CBR video + MP3-CBR audio derived from config; demo/local byte-stable; with/without-audio + CBR vector pins updated offline | All four pinned by the green gate rosters (`stream_args_tests` 4, `stream_args_health_tests` 2, `streaming_stage` 1 unix; offline fakes/loopback/local toolchain only); demo vector + config-schema untouched (D7 ledger, §1.1) | EVIDENCED — `[ ]` may flip `[x]` at 4.6 |
| Criterion 4 — real-container pin verifies measured ≈ configured, honest tolerance | The renamed real-FFmpeg pin: h264 + mp3/48000/stereo; video stream bit_rate ≥ 5,400,000; measured container total ≥ 5,515,200 (subsumes the audio floor); duration ∈ [8.0, 13.0] s; the 4.2 record holds `ffmpeg_real_audio 1 PASSED` ⇒ STOP-3 never triggered; the floor is NOT weakened | EVIDENCED — `[ ]` may flip `[x]` at 4.6 |
| (shared criteria 5–6, whole-TODO) | Gate 391/0 `ALL CHECKS PASSED` on `logs/checks/20261025T045635Z.log` FINAL tree; builds 5224152/8639517 B `ls -l`-cross-checked; audits clean; v0.11.1 already from Task 1 (no bump here); docs+structure map+context updated truthfully; NO real-Google claim anywhere in the diffs (checked — production-run.md claims explicitly operator-run) | Consistent for the whole TODO |

## 5. Unverifiable-directly items (host-side)

- The gate log, both build sizes and the audit greps run on the Alpine VM; `logs/checks/20261025T045635Z.log` and the `dist/` artifacts are gitignored VM-side and absent from the host working tree — this report CITES the 4.2/4.4 records for them (391/0; 5224152/8639517) instead of re-observing. Everything code-level was re-verified host-side from the tree/diffs.
- The real-container floor pass is cited from the 4.2 record's targeted run; the plan's STOP-3 rerun-once procedure never fired.

## 6. For 4.6

- Commits: the uncommitted `.agent/project-info/context.md` (the 4.4 bullet) plus the 4.6 completion marks; then step 5 (TODO `[DONE]` rename, merge to `main`, push `origin` ONLY).
- Fill criteria 3 and 4 `[x]` and Task-2 `[DONE]` on the evidence of §4 above.

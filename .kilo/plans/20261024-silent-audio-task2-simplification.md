# Task 2 Silent-Audio Simplification Plan (4.3-fix — junior-executable)

Plan date: 2026-10-24 · Owner: Code Simplifier (Critical Workflow sub-step 4.3)
Applies to: `.kilo/plans/20261024-silent-audio-task2-implementation.md` (BINDING) · TODO `.agent/todos/20261024/20261024-todo-1.md` Task 2
Branch: `fix/ingest-no-data` at HEAD `9a9bd13` · Baseline: 387/0 green (`logs/checks/20261024T183305Z.log`)

## 0. Status and authority

- The simplifier reviewed the 15 Task-2-changed files. Direct `src/**` edits were permission-blocked for the simplifier, so THIS file is the implementation mechanism: the 4.3-fix implementer applies the steps below verbatim, verifies, and commits.
- Exactly 4 findings qualify as genuine simplifications with zero behavior change: Steps 1–3 (code) + Step 4 (one metadata line REQUIRED for consistency once Step 1 lands). Nothing else in the changed set is to be touched.
- The pinned vectors stay byte-identical: no token, no order, no constant numbering, no fixture/example truth changes. Every pinned assert remains in place; Steps 1–3 do not alter any generated argv.

BINDING GUARDRAILS (restate — violation = STOP): do NOT merge `stream_args.rs` back into `args.rs`; do NOT merge the new test rosters; do NOT touch the byte-pinned vector literals; do NOT edit `src/config/loader_tests.rs` (AT CAP — zero edits); do NOT touch `validation.rs`, O1–O5 forms, `src/app/demo/*`, `Cargo.toml`, `Cargo.lock`, `.gitignore`; container-only verification, offline tests only; NEVER `git add -A`; never stage `logs/*`, `dist/`, `videos/`, `target/`, `config/config.json`.

## 1. Step 1 — `src/streaming/args.rs`: undo the unnecessary `pub(super)` widening

`REQUIRED_PIXEL_FORMAT` was widened to `pub(super)` by the split commit, but nothing outside `args.rs` references it (`stream_args.rs` imports only `is_non_rgb24_pixel_format`, `OUTPUT_PIXEL_FORMAT`, `PIPE_SOURCE`; verified by repo-wide grep). Narrow it back to module-private.

Edit file `src/streaming/args.rs`, line 47. Replace exactly:

```rust
pub(super) const REQUIRED_PIXEL_FORMAT: &str = "rgb24";
```

with exactly:

```rust
const REQUIRED_PIXEL_FORMAT: &str = "rgb24";
```

File stays 94 lines. The other three `pub(super)` items and `build_arguments` (pre-existing `pub(crate)`, plan §D6 byte-untouched) stay AS-IS.

## 2. Step 2 — `src/streaming/stream_args.rs`: evaluate the audio gate once

`build_stream_arguments` calls the pure accessor `config.ffmpeg.audio_enabled()` twice. Bind it once (zero behavior change; both `if` arms and the token order are untouched).

Edit `src/streaming/stream_args.rs`, inside `build_stream_arguments`. Replace exactly:

```rust
    let mut arguments = stream_input_arguments(config);
    if config.ffmpeg.audio_enabled() {
        arguments.extend(audio_input_arguments());
    }
    arguments.extend(stream_video_output_arguments(config));
    if config.ffmpeg.audio_enabled() {
        arguments.extend(audio_output_arguments());
    }
```

with exactly:

```rust
    let audio_enabled = config.ffmpeg.audio_enabled();
    let mut arguments = stream_input_arguments(config);
    if audio_enabled {
        arguments.extend(audio_input_arguments());
    }
    arguments.extend(stream_video_output_arguments(config));
    if audio_enabled {
        arguments.extend(audio_output_arguments());
    }
```

File becomes 141 lines (cap 200 — OK). Do not touch any other part of the file (constants block, helpers, docs — all plan-verbatim).

## 3. Step 3 — `src/streaming/stream_args_tests.rs`: build the fixture destination once

In `audio_absent_false_and_unset_enabled_hold_the_identical_video_only_vector`, three inline `RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string())` constructions duplicate two needless allocations and diverge from the file's own `let destination = ...` style (used by the other three tests). No pinned vector literal is in this test — the assert blocks (pairwise `assert_eq!`) stay untouched.

Edit `src/streaming/stream_args_tests.rs`. Replace exactly:

```rust
    let section_absent = build_stream_arguments(
        &without_section,
        &RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string()),
    )
    .expect("section-absent audio must build the video-only vector");
    let flag_absent = build_stream_arguments(
        &without_flag,
        &RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string()),
    )
    .expect("flag-absent audio must build the video-only vector");
    let disabled = build_stream_arguments(
        &explicitly_disabled,
        &RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string()),
    )
    .expect("disabled audio must build the video-only vector");
```

with exactly:

```rust
    let destination = RtmpsDestination::new(DESTINATION_FIXTURE_URL.to_string());
    let section_absent = build_stream_arguments(&without_section, &destination)
        .expect("section-absent audio must build the video-only vector");
    let flag_absent = build_stream_arguments(&without_flag, &destination)
        .expect("flag-absent audio must build the video-only vector");
    let disabled = build_stream_arguments(&explicitly_disabled, &destination)
        .expect("disabled audio must build the video-only vector");
```

File becomes 168 lines (cap 200 — OK). The replacement text is pre-formatted in the file's existing rustfmt shape (mirrors its lines 22–23 pattern).

## 4. Step 4 — `.agent/project-structure.md`: REQUIRED consistency line

Step 1 makes the current map claim ("all `pub(super)` for the sibling") false. Replace the unique fragment (line 94):

```
(`REQUIRED_PIXEL_FORMAT`, `PIPE_SOURCE`, `OUTPUT_PIXEL_FORMAT`, `is_non_rgb24_pixel_format`, all `pub(super)` for the sibling)
```

with:

```
(`PIPE_SOURCE`, `OUTPUT_PIXEL_FORMAT`, `is_non_rgb24_pixel_format` are `pub(super)` for the sibling; `REQUIRED_PIXEL_FORMAT` stays module-private)
```

No other line of the map changes. (If the caller prefers, this one-liner may instead be handed to the 4.4 docs-specialist — the default per this plan is to commit it WITH the code in Step 6 so the map never lies between commits.)

## 5. Step 5 — verification (container only; ONE command per call; stop at the FIRST failure and report)

Run each command EXACTLY (host shell, repo root `C:\repo\rust-youtube-streamer`):

1. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check`
   → expect exit 0, no diff output. On FAIL: STOP and report — do NOT hand-reformat or run `cargo fmt` (the plan text above is already rustfmt-stable; a fmt diff means a transcription error to re-check against this file).
2. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked`
   → expect exit 0. Any compile error: STOP (proves a guardrail was broken).
3. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked args_tests`
   → expect `test result: ok. 8 passed; 0 failed; 379 filtered out` (4 `args_tests` + 4 `stream_args_tests`; the exact 34-token pin and the video-only pin live here — their green proves the pinned vectors are byte-identical).
4. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked audio`
   → expect `test result: ok. 9 passed; 0 failed; 378 filtered out` (6 parse roster + 2 stream-args audio + 1 real-FFmpeg mechanism pin).
5. `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked stream_pipeline`
   → expect `test result: ok. 1 passed; 0 failed; 386 filtered out` (argv end-to-end pin).
6. Diff audit (host): `git diff --stat` → expect EXACTLY 4 modified files: `src/streaming/args.rs`, `src/streaming/stream_args.rs`, `src/streaming/stream_args_tests.rs`, `.agent/project-structure.md`. `git diff` → confirm no hunk touches a vector literal line, the constants block, `loader_tests.rs`, `model.rs`, fixtures, example JSON, or the rosters. Anything else: STOP and report.
7. Full standard gate (writes a fresh log under `logs/checks/`): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
   → expect exit 0, four PASS lines (`fmt-check`, `check`, `test`, `clippy`), `ALL CHECKS PASSED`, test line `387 passed; 0 failed` (NO count delta — these are simplifications, not new tests), and the warning total still exactly the 6 adjudicated test-profile warnings (unused `wait_until_stream_active` import/assoc-fn/const/const/fn + `token_type` field). NEW warning = STOP and report. Record the fresh log path.

## 6. Step 6 — commit (single fix commit; gitignore-compliance first)

1. Read `.gitignore`; run `git status --porcelain` → the ONLY modified entries must be the 4 files of Step 5.6 (a fresh untracked `logs/checks/*.log` is ignored — never stage it).
2. Stage ONLY those 4 files (explicit paths, never `-A`).
3. `git commit -m "refactor: apply Task 2 silent-audio simplification review"`.
4. Post-check: `git status --porcelain` → empty; `git log --oneline -3` shows the new commit above `9a9bd13`.
5. Do NOT push (workflow step 5 owns that). Do NOT amend.

## 7. Reviewed and deliberately NOT changed (binding rationale)

- `stream_input_arguments`/`stream_video_output_arguments` intermediate `Vec` + `extend` shape: plan §S5 verbatim builder decomposition — changing helper signatures is a structural decision beyond this sub-step (one-time argv construction, zero hot-path cost).
- Duplicated `SMOKE_FRAME_BYTES`/`SMOKE_FRAME_COUNT` consts in `ffmpeg_real_audio_tests.rs`: consolidating them would require NEW `pub(super)` widenings in `ffmpeg_real_tests.rs`; duplication preserves minimal visibility (prefer-private-members rule). Rejected.
- Helper duplication (`example_value`/`load_modified`/`write_temp_config`/`expect_load_error`) between `ffmpeg_audio_parse_tests.rs` and the mfk roster: plan §D9-A mandates same-body copies per roster; merging rosters is a caller-issued guardrail violation. Kept.
- Audio parse roster true/false tests not folded into a shared assert helper: folding would diverge from the mfk roster plan-shape (§D9-A "mirror exactly"). Kept unfolded.
- `streaming_stage_tests.rs` four appended asserts and the trailing shape asserts in `stream_arguments_match_the_locked_output_shape` (the final `-f`/`flv` pair re-check duplicates the full-vector pin): verbatim/adjudicated plan §S8/§D9-B text — redundancy IS the pinned contract. Kept.
- `format!("{}", config.video.fps)` → `.to_string()`: plan §S5/§D6 verbatim code; clippy-clean as-is. Kept.
- Docs commit `9a9bd13` wording (configuration.md, production-run.md, README 387 line, tech.md, context.md): every passage is plan §D12/§D13 verbatim or caller-adjudicated; no tightening is authorized here. Report-only note: none needed.
- `build_arguments` `pub(crate)` (callers are crate-internal only): pre-Task-2 visibility, plan §D6 orders it byte-untouched — see Deferred.

## 8. Deferred candidates (caller/architor decision, NOT for this fix)

1. Narrow `args.rs::build_arguments` `pub(crate)` → `pub(super)` in a future phase with §D6 reauthorization.
2. Re-express the two segment helpers as `push`/`extend`-on-reference builders to drop two intermediate Vecs — structural, needs plan-amendment authority.
3. Shared image-constants module for the two real-FFmpeg rosters if a third roster ever appears.

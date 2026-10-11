# Simplification Plan — Phase 03, Task 1 (streaming module + main.rs)

- **Scope reviewed**: `src/streaming/{mod,error,error_tests,args,args_tests,utc_timestamp,utc_timestamp_tests,video_output,video_output_tests,sink,ffmpeg}.rs` and the `src/main.rs` addition, as introduced in `3bde337..7b19901` on `feat/phase03-ffmpeg-local-pipeline`.
- **Governing plan**: `.kilo/plans/20261011-phase03-task1-ffmpeg-wrapper.md` (respected everywhere; see §4 rejections).
- **Verdict**: implementation is already close to house style. Five small, semantics-preserving edits remain. No redesign needed; this stays inside Task 1 — no new TODO file required.
- **Implementer**: junior, 50% restriction. Execute steps in order, exactly as written. No judgment calls, no extra changes, no touching any other file.

## 0. Locked invariants (must not change; every step below preserves them)

- FFmpeg argument vector content/order (`args.rs` untouched).
- `FrameSink`/`CompletedVideo` seam (`sink.rs` untouched), drain-before-wait supervision and Drop-kill (`ffmpeg.rs` semantics untouched).
- All `StreamingError` message texts, variants, `source()` behavior (`error.rs` Display/logic untouched).
- Collision-safe naming contract: same candidate strings, `attempts: 1000` (`video_output.rs` outputs identical).
- `main.rs` and `mod.rs` untouched. No test added, removed, or renamed.

## 1. Ordered steps (each atomic, each verified before the next)

All verification runs in the Alpine VM (call `alpine-vm_vm_status` first). Standard per-step gate:

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked streaming
```

Expect exit 0 with 21 streaming tests passing (identical count before and after every step).

### Step S1 — `src/streaming/error.rs`: narrow helper visibility (prefer-private-members)

`is_broken_pipe` (line 144) and `is_file_not_found` (line 148) are used only inside `error.rs` (verified by grep across the module); the `cfg(test)` submodule can see private items too.

- Change `pub(super) fn is_broken_pipe(source: &io::Error) -> bool {` → `fn is_broken_pipe(source: &io::Error) -> bool {`
- Change `pub(super) fn is_file_not_found(source: &io::Error) -> bool {` → `fn is_file_not_found(source: &io::Error) -> bool {`
- Do NOT touch `trailing_bytes` (consumed by `ffmpeg.rs`; stays `pub(super)`).
- Gate: step command above, exit 0.

### Step S2 — `src/streaming/utc_timestamp.rs`: name the leap-year length (no-magic-numbers)

- Add `const LEAP_YEAR_DAYS: u64 = 366;` directly after `const NON_LEAP_YEAR_DAYS: u64 = 365;` (line 15).
- In `days_in_year`, replace the body's `NON_LEAP_YEAR_DAYS + 1` with `LEAP_YEAR_DAYS`. Leave the `if is_leap_year(year)` shape and everything else unchanged.
- Gate: step command above, exit 0.

### Step S3 — `src/streaming/video_output.rs`: stem-based candidate names (dedup, removes unreachable fallback)

`first_free_suffixed_name` currently re-strips the extension with an `unwrap_or(base_name)` fallback that is dead in practice (the only input comes from `base_name()`), and the `.mp4` assembly exists in two places. Restructure to build from the stem. Exact replacement for the three private fns plus `next_available` (doc comments on `next_available` stay verbatim):

- Replace `fn base_name(&self) -> String` (lines 73–81) with:

```rust
    fn base_stem(&self) -> String {
        format!(
            "{}-{}",
            self.kind.filename_prefix(),
            format_utc_timestamp(self.epoch_seconds)
        )
    }

    fn with_extension(stem: &str) -> String {
        format!("{stem}{FILE_EXTENSION_SEPARATOR}{FILENAME_EXTENSION}")
    }
```

- Replace `next_available` body (lines 66–70) with:

```rust
        let stem = self.base_stem();
        let base_name = Self::with_extension(&stem);
        if !exists(&self.output_directory.join(&base_name)) {
            return Ok(base_name);
        }
        self.first_free_suffixed_name(&stem, exists)
```

- Replace `first_free_suffixed_name` (lines 83–102) with:

```rust
    fn first_free_suffixed_name(
        &self,
        stem: &str,
        exists: &mut dyn FnMut(&Path) -> bool,
    ) -> Result<String, StreamingError> {
        for suffix in FIRST_COLLISION_SUFFIX..=MAX_SUFFIX_ATTEMPTS {
            let candidate =
                Self::with_extension(&format!("{stem}{COLLISION_SUFFIX_SEPARATOR}{suffix}"));
            if !exists(&self.output_directory.join(&candidate)) {
                return Ok(candidate);
            }
        }
        Err(StreamingError::OutputNameExhausted {
            directory: self.output_directory.clone(),
            attempts: MAX_SUFFIX_ATTEMPTS,
        })
    }
```

- Every returned string is byte-identical to today (`demo-YYYYMMDD-HHMMSS.mp4`, `..._2.mp4`, ...). Do not change constants, `resolve_output_path`, or any test.
- Gate: step command above, exit 0 (naming tests `next_available_*` and `generated_base_name_matches_timestamped_format` are the safety net).

### Step S4 — `src/streaming/ffmpeg.rs`: three micro-fixes

a) Standard-library dedup + honest name. `append_arguments` hand-rolls a loop and also configures stdio (name lies). Replace lines 73–81 with:

```rust
fn prepare_piped_command(command: &mut Command, arguments: &[String]) {
    command
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
}
```

and update its single call site in `start` (line 47) to `prepare_piped_command(&mut command, &arguments);`. `Command::args` appends in slice order — the locked vector is unchanged.

b) `capture_pipe` (lines 87–107): the `match` builds `CapturedPipe` three times. After the existing let-else branch, replace the match block with:

```rust
    let mut collected_bytes = Vec::new();
    let final_error = pipe.read_to_end(&mut collected_bytes).err();
    CapturedPipe {
        collected_bytes,
        final_error,
    }
```

`Result::err()` yields `None` on `Ok` and `Some(error)` on `Err`, and `read_to_end` still appends every byte read before the failure — identical semantics, fewer lines.

c) Drop-doc wording (line 183–184 region): change "so an unrevisited `FfmpegProcess` cannot leak" → "so an un-awaited `FfmpegProcess` cannot leak" (matches plan §1.8 wording).

- Gate: step command above, then additionally (each exit 0):

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo clippy --locked -- -D warnings
```

### Step S5 — `src/streaming/error_tests.rs`: satisfy the ≤50-line method rule

`each_variant_renders_its_display_contract` has a ~90-line body. Keep the test name and all 12 exact expected strings; move the data into two private fixture fns in the same file (each ≤ 50-line body). Exact shape:

```rust
fn plain_display_cases() -> Vec<(StreamingError, &'static str)> { ... }
fn io_cause_display_cases() -> Vec<(StreamingError, &'static str)> { ... }

#[test]
fn each_variant_renders_its_display_contract() {
    let mut cases = plain_display_cases();
    cases.extend(io_cause_display_cases());
    for (error, expected) in &cases {
        assert_eq!(error.to_string(), *expected);
    }
}
```

- `plain_display_cases` entries (in this order), each a `(variant, message)` pair moved VERBATIM from the current test body (retype nothing): UnsupportedPixelFormat → ExecutableLocation → OutputNameExhausted → ExitFailure{Some(1)} → ExitFailure{None} → OutputFinalize.
- `io_cause_display_cases` entries (in this order): OutputDirectoryCreate → Spawn(NotFound hint case) → Spawn(PermissionDenied plain case) → FrameWrite(BrokenPipe hint case) → FrameWrite(PermissionDenied plain case) → WaitFailed. Reuse the existing `permission_denied()` helper.
- `source_returns_io_error_where_defined` (43-line body) and every other test: do NOT touch.

### Step S6 — Format, final gate, commit point

1. VM: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt` then `... cargo fmt --check` (both exit 0).
2. VM full gate: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh` → exit 0, `ALL CHECKS PASSED`, total tests unchanged (81 baseline + 21 streaming = 102).
3. Host (single commands): `git status --porcelain` → ONLY these 5 modified: `src/streaming/error.rs`, `src/streaming/utc_timestamp.rs`, `src/streaming/video_output.rs`, `src/streaming/ffmpeg.rs`, `src/streaming/error_tests.rs`. Nothing under `videos/`, `logs/`, `target/` (gitignore-compliance).
4. Commit (one commit for the whole pass): `refactor: simplify streaming internals after task 1 review` — stage only the 5 files above.

## 2. Tests that must stay green (unchanged names, unchanged count)

`streaming::args_tests` (4), `streaming::utc_timestamp_tests` (5), `streaming::video_output_tests` (10, incl. `generated_base_name_matches_timestamped_format` and `next_available_*` — they pin S3), `streaming::error_tests` (2, `each_variant_renders_its_display_contract` keeps its name in S5), plus all 81 pre-existing tests. S5 changes test-file internals only; S1–S4 must not alter any observable behavior.

## 3. Rule-driven outcomes

- `error.rs` 168→168, `ffmpeg.rs` 188→~180, `video_output.rs` 167→~165, all ≤ 200; every method body ≤ 50 lines after S5; depth ≤ 2 and ≤ 2 params everywhere (unchanged); no commented-out code anywhere in the module (verified); no new public API surface (S1 shrinks it).

## 4. Considered and rejected (do NOT implement)

- Collapsing the two-arm `Display` branches for `Spawn`/`FrameWrite` — plan §1.3 explicitly mandates "two write arms on the helper"; message strings are locked.
- Reformatting the `vec![...to_string()...]` construction in `args.rs` — order-locked, currently explicit and test-pinned.
- Removing `output_path.clone()` in `finish` — `FfmpegProcess` implements `Drop`, so the field cannot be moved out; the clone is required.
- `skip_whole_years`'s `&mut` parameter style — churn without readability gain; `is_non_rgb24_pixel_format` negative naming — reads correctly at its single call site.
- Mixed positional/inline format args in `Display` — touching them risks the locked strings.
- `.agent/project-structure.md` still says "placeholder; future phase" for `streaming/` — stale, but structure/doc updates are Task 5 scope (governing plan §5); out of bounds for Task 1.

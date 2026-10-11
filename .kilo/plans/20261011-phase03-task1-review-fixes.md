# Fix Plan — Phase 03, TODO Task 1 Code Review

Review findings and exact fixes for the implementer. All changes stay inside Task 1's existing files; no scope expansion, no new modules, no architectural changes.

---

## Findings

1. **UTC civil-date algorithm incorrect across non-leap centuries** — *high / correctness*
   - File: `src/streaming/utc_timestamp.rs`
   - The 4-year block acceleration assumes every 4-year period has 1461 days, which fails across century years not divisible by 400 (e.g., 2100).
   - Verified manually: `format_utc_timestamp(4_233_772_800)` (2104-03-01T00:00:00Z) currently returns `21040229-000000` instead of `21040301-000000`.
   - Fix: remove the 4-year block step; after the 400-year block, walk whole years directly.

2. **`StreamingError::fmt` body exceeds the 50-line method limit** — *medium / style*
   - File: `src/streaming/error.rs`
   - The `fmt` body is approximately 68 lines.
   - Fix: extract the `Spawn` and `FrameWrite` formatting arms into small private helpers.

3. **`each_variant_renders_its_display_contract` test body exceeds the 50-line limit** — *medium / style*
   - File: `src/streaming/error_tests.rs`
   - The test body is approximately 91 lines.
   - Fix: split the assertions into three focused test functions.

4. **Process leak on the missing-stdin defensive path** — *medium / robustness*
   - File: `src/streaming/ffmpeg.rs`
   - If `child.stdin.take()` returns `None`, the spawned child is dropped without kill/wait.
   - Fix: kill the child and wait to reap it before returning the typed error.

5. **`Drop` does not reap the killed child** — *low / robustness*
   - File: `src/streaming/ffmpeg.rs`
   - `Drop` calls `kill()` but not `wait()`, leaving zombie processes on Unix when `finish()` is not reached.
   - Fix: call `wait()` after `kill()` in `Drop`.

---

## Ordered fix commits

### Commit 1 — `fix: correct UTC civil date algorithm across non-leap centuries`

Files touched: `src/streaming/utc_timestamp.rs`, `src/streaming/utc_timestamp_tests.rs`

#### Edit 1 — `src/streaming/utc_timestamp.rs`

Remove the now-unused `DAYS_PER_4_YEARS` constant and the 4-year block step.

Old code block:

```rust
const DAYS_PER_4_YEARS: u64 = 1_461;
const DAYS_PER_400_YEARS: u64 = 146_097;
```

New code block:

```rust
const DAYS_PER_400_YEARS: u64 = 146_097;
```

Old function:

```rust
fn civil_date_from_epoch_days(days: u64) -> (u64, u32, u32) {
    let mut remaining_days = days;
    let mut year = EPOCH_YEAR + 400 * (remaining_days / DAYS_PER_400_YEARS);
    remaining_days %= DAYS_PER_400_YEARS;
    year += 4 * (remaining_days / DAYS_PER_4_YEARS);
    remaining_days %= DAYS_PER_4_YEARS;
    year = skip_whole_years(year, &mut remaining_days);
    let (month, day) = month_and_day(year, remaining_days);
    (year, month, day)
}
```

New function:

```rust
fn civil_date_from_epoch_days(days: u64) -> (u64, u32, u32) {
    let mut remaining_days = days;
    let mut year = EPOCH_YEAR + 400 * (remaining_days / DAYS_PER_400_YEARS);
    remaining_days %= DAYS_PER_400_YEARS;
    year = skip_whole_years(year, &mut remaining_days);
    let (month, day) = month_and_day(year, remaining_days);
    (year, month, day)
}
```

#### Edit 2 — `src/streaming/utc_timestamp_tests.rs`

Append two regression tests after `leap_day_tests_the_civil_conversion`.

Old closing of the test module (the last test ends at line 35, then the file ends):

```rust
#[test]
fn leap_day_tests_the_civil_conversion() {
    // Verified: 946_684_800 = 2000-01-01T00:00:00Z
    // (https://unixepoch.net/ reference table), plus 59 days to
    // 2000-02-29 (31 days of January + 28 of February): 946_684_800 +
    // 59 * 86_400 = 951_782_400. The plan draft's 951_868_800 is one day
    // later (2000-03-01); the plan mandates this online verification.
    assert_eq!(format_utc_timestamp(951_782_400), "20000229-000000");
}
```

New closing of the test module:

```rust
#[test]
fn leap_day_tests_the_civil_conversion() {
    // Verified: 946_684_800 = 2000-01-01T00:00:00Z
    // (https://unixepoch.net/ reference table), plus 59 days to
    // 2000-02-29 (31 days of January + 28 of February): 946_684_800 +
    // 59 * 86_400 = 951_782_400. The plan draft's 951_868_800 is one day
    // later (2000-03-01); the plan mandates this online verification.
    assert_eq!(format_utc_timestamp(951_782_400), "20000229-000000");
}

#[test]
fn post_non_leap_century_march_date_formats_correctly() {
    // 2104-03-01T00:00:00Z; the 2100 non-leap century used to shift this
    // by one day when the 4-year block acceleration was misapplied.
    assert_eq!(format_utc_timestamp(4_233_772_800), "21040301-000000");
}

#[test]
fn leap_day_after_non_leap_century_formats_correctly() {
    // 2104-02-29T00:00:00Z.
    assert_eq!(format_utc_timestamp(4_233_686_400), "21040229-000000");
}
```

#### Verification

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked streaming::utc_timestamp
```

Expected: exit 0, all utc_timestamp tests pass.

---

### Commit 2 — `refactor: split oversized StreamingError Display impl and tests`

Files touched: `src/streaming/error.rs`, `src/streaming/error_tests.rs`

#### Edit 1 — `src/streaming/error.rs`

Add two private formatting helpers immediately after `is_file_not_found`.

Old code block (lines 144–150):

```rust
pub(super) fn is_file_not_found(source: &io::Error) -> bool {
    source.kind() == io::ErrorKind::NotFound
}

/// Keeps only the trailing `limit` bytes of a captured diagnostic stream so
```

New code block:

```rust
pub(super) fn is_file_not_found(source: &io::Error) -> bool {
    source.kind() == io::ErrorKind::NotFound
}

fn format_spawn_error(
    formatter: &mut fmt::Formatter<'_>,
    executable: &str,
    source: &io::Error,
) -> fmt::Result {
    if is_file_not_found(source) {
        write!(
            formatter,
            "unable to start the FFmpeg executable '{}': {}; check that the configured path is correct and FFmpeg is installed and runnable",
            executable,
            source
        )
    } else {
        write!(
            formatter,
            "unable to start the FFmpeg executable '{}': {}",
            executable,
            source
        )
    }
}

fn format_frame_write_error(
    formatter: &mut fmt::Formatter<'_>,
    source: &io::Error,
) -> fmt::Result {
    if is_broken_pipe(source) {
        write!(
            formatter,
            "writing a frame to the FFmpeg process failed: {}; the FFmpeg process exited while receiving frames",
            source
        )
    } else {
        write!(
            formatter,
            "writing a frame to the FFmpeg process failed: {}",
            source
        )
    }
}

/// Keeps only the trailing `limit` bytes of a captured diagnostic stream so
```

#### Edit 2 — `src/streaming/error.rs`

Replace the inline `Spawn` and `FrameWrite` arms in `fmt::Display` with helper calls.

Old arm:

```rust
            StreamingError::Spawn { executable, source } => {
                if is_file_not_found(source) {
                    write!(
                        formatter,
                        "unable to start the FFmpeg executable '{}': {}; check that the configured path is correct and FFmpeg is installed and runnable",
                        executable,
                        source
                    )
                } else {
                    write!(
                        formatter,
                        "unable to start the FFmpeg executable '{}': {}",
                        executable,
                        source
                    )
                }
            }
```

New arm:

```rust
            StreamingError::Spawn { executable, source } => {
                format_spawn_error(formatter, executable, source)
            }
```

Old arm:

```rust
            StreamingError::FrameWrite { source } => {
                if is_broken_pipe(source) {
                    write!(
                        formatter,
                        "writing a frame to the FFmpeg process failed: {}; the FFmpeg process exited while receiving frames",
                        source
                    )
                } else {
                    write!(
                        formatter,
                        "writing a frame to the FFmpeg process failed: {}",
                        source
                    )
                }
            }
```

New arm:

```rust
            StreamingError::FrameWrite { source } => {
                format_frame_write_error(formatter, source)
            }
```

#### Edit 3 — `src/streaming/error_tests.rs`

Replace the single oversized test with three focused tests. The assertions and expected strings stay identical.

Old test (the entire `each_variant_renders_its_display_contract` function, lines 14–106):

```rust
#[test]
fn each_variant_renders_its_display_contract() {
    assert_eq!(
        StreamingError::UnsupportedPixelFormat {
            value: "yuv420p".to_string()
        }
        .to_string(),
        "streaming pipeline requires pixel format 'rgb24' in configuration (got 'yuv420p')"
    );
    assert_eq!(
        StreamingError::ExecutableLocation {
            reason: "no executable".to_string()
        }
        .to_string(),
        "unable to resolve the application executable location: no executable"
    );
    assert_eq!(
        StreamingError::OutputDirectoryCreate {
            path: PathBuf::from("/opt/app/videos"),
            source: permission_denied(),
        }
        .to_string(),
        "unable to create video output directory '/opt/app/videos': permission denied"
    );
    assert_eq!(
        StreamingError::OutputNameExhausted {
            directory: PathBuf::from("/opt/app/videos"),
            attempts: 1000,
        }
        .to_string(),
        "video file naming gave up in directory '/opt/app/videos' after 1000 collision suffixes; the recording was not started"
    );
    assert_eq!(
        StreamingError::Spawn {
            executable: "ffmpeg".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "missing binary"),
        }
        .to_string(),
        "unable to start the FFmpeg executable 'ffmpeg': missing binary; check that the configured path is correct and FFmpeg is installed and runnable"
    );
    assert_eq!(
        StreamingError::Spawn {
            executable: "/usr/bin/ffmpeg".to_string(),
            source: permission_denied(),
        }
        .to_string(),
        "unable to start the FFmpeg executable '/usr/bin/ffmpeg': permission denied"
    );
    assert_eq!(
        StreamingError::FrameWrite {
            source: io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"),
        }
        .to_string(),
        "writing a frame to the FFmpeg process failed: broken pipe; the FFmpeg process exited while receiving frames"
    );
    assert_eq!(
        StreamingError::FrameWrite {
            source: permission_denied(),
        }
        .to_string(),
        "writing a frame to the FFmpeg process failed: permission denied"
    );
    assert_eq!(
        StreamingError::WaitFailed {
            source: io::Error::new(io::ErrorKind::Interrupted, "interrupted"),
        }
        .to_string(),
        "waiting for the FFmpeg process failed: interrupted"
    );
    assert_eq!(
        StreamingError::ExitFailure {
            exit_code: Some(1),
            diagnostics: "Unknown encoder 'libx264'".to_string(),
        }
        .to_string(),
        "FFmpeg exited with failure status 1; captured diagnostics follow: Unknown encoder 'libx264'"
    );
    assert_eq!(
        StreamingError::ExitFailure {
            exit_code: None,
            diagnostics: "killed".to_string(),
        }
        .to_string(),
        "FFmpeg terminated by a signal; captured diagnostics follow: killed"
    );
    assert_eq!(
        StreamingError::OutputFinalize {
            path: PathBuf::from("/opt/app/videos/demo.mp4"),
        }
        .to_string(),
        "FFmpeg reported success but the output file '/opt/app/videos/demo.mp4' is missing or empty; the video was not finalized"
    );
}
```

New tests:

```rust
#[test]
fn format_errors_without_io_source() {
    assert_eq!(
        StreamingError::UnsupportedPixelFormat {
            value: "yuv420p".to_string()
        }
        .to_string(),
        "streaming pipeline requires pixel format 'rgb24' in configuration (got 'yuv420p')"
    );
    assert_eq!(
        StreamingError::ExecutableLocation {
            reason: "no executable".to_string()
        }
        .to_string(),
        "unable to resolve the application executable location: no executable"
    );
    assert_eq!(
        StreamingError::OutputDirectoryCreate {
            path: PathBuf::from("/opt/app/videos"),
            source: permission_denied(),
        }
        .to_string(),
        "unable to create video output directory '/opt/app/videos': permission denied"
    );
    assert_eq!(
        StreamingError::OutputNameExhausted {
            directory: PathBuf::from("/opt/app/videos"),
            attempts: 1000,
        }
        .to_string(),
        "video file naming gave up in directory '/opt/app/videos' after 1000 collision suffixes; the recording was not started"
    );
}

#[test]
fn format_errors_with_io_source() {
    assert_eq!(
        StreamingError::Spawn {
            executable: "ffmpeg".to_string(),
            source: io::Error::new(io::ErrorKind::NotFound, "missing binary"),
        }
        .to_string(),
        "unable to start the FFmpeg executable 'ffmpeg': missing binary; check that the configured path is correct and FFmpeg is installed and runnable"
    );
    assert_eq!(
        StreamingError::Spawn {
            executable: "/usr/bin/ffmpeg".to_string(),
            source: permission_denied(),
        }
        .to_string(),
        "unable to start the FFmpeg executable '/usr/bin/ffmpeg': permission denied"
    );
    assert_eq!(
        StreamingError::FrameWrite {
            source: io::Error::new(io::ErrorKind::BrokenPipe, "broken pipe"),
        }
        .to_string(),
        "writing a frame to the FFmpeg process failed: broken pipe; the FFmpeg process exited while receiving frames"
    );
    assert_eq!(
        StreamingError::FrameWrite {
            source: permission_denied(),
        }
        .to_string(),
        "writing a frame to the FFmpeg process failed: permission denied"
    );
    assert_eq!(
        StreamingError::WaitFailed {
            source: io::Error::new(io::ErrorKind::Interrupted, "interrupted"),
        }
        .to_string(),
        "waiting for the FFmpeg process failed: interrupted"
    );
}

#[test]
fn format_exit_and_finalize_errors() {
    assert_eq!(
        StreamingError::ExitFailure {
            exit_code: Some(1),
            diagnostics: "Unknown encoder 'libx264'".to_string(),
        }
        .to_string(),
        "FFmpeg exited with failure status 1; captured diagnostics follow: Unknown encoder 'libx264'"
    );
    assert_eq!(
        StreamingError::ExitFailure {
            exit_code: None,
            diagnostics: "killed".to_string(),
        }
        .to_string(),
        "FFmpeg terminated by a signal; captured diagnostics follow: killed"
    );
    assert_eq!(
        StreamingError::OutputFinalize {
            path: PathBuf::from("/opt/app/videos/demo.mp4"),
        }
        .to_string(),
        "FFmpeg reported success but the output file '/opt/app/videos/demo.mp4' is missing or empty; the video was not finalized"
    );
}
```

#### Verification

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo test --locked streaming::error
```

Expected: exit 0, all error tests pass.

---

### Commit 3 — `fix: kill and reap FFmpeg child on missing stdin and in Drop`

File touched: `src/streaming/ffmpeg.rs`

#### Edit 1 — `src/streaming/ffmpeg.rs`

In `FfmpegProcess::start`, handle a missing stdin pipe by killing and reaping the child before returning the typed error.

Old line:

```rust
        let stdin = child.stdin.take().ok_or_else(missing_stdin_error)?;
```

New block:

```rust
        let stdin = match child.stdin.take() {
            Some(stdin) => stdin,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(missing_stdin_error());
            }
        };
```

#### Edit 2 — `src/streaming/ffmpeg.rs`

In `Drop`, wait after killing so the child is reaped.

Old block:

```rust
impl Drop for FfmpegProcess {
    /// Kills a still-running child so an unrevisited `FfmpegProcess` cannot
    /// leak; kill errors on an already-exited process are ignored.
    fn drop(&mut self) {
        let _kill_result = self.child.kill();
    }
}
```

New block:

```rust
impl Drop for FfmpegProcess {
    /// Kills a still-running child so an unrevisited `FfmpegProcess` cannot
    /// leak, then waits to reap it. Errors on an already-exited process are
    /// ignored.
    fn drop(&mut self) {
        let _kill_result = self.child.kill();
        let _wait_result = self.child.wait();
    }
}
```

#### Verification

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo check --locked
```

Expected: exit 0.

---

## Final verification

After all three commits, run the full standard gate:

```text
docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh
```

Expected: exit 0, `ALL CHECKS PASSED`, and the test count increases from 102 to 104 because of the two new UTC regression tests.

Repo host checks:

```text
git status --porcelain   # expect clean
git check-ignore -v videos/   # expect the videos/ rule
git ls-files videos/   # expect empty
```

# Adherence Report — Phase 03, TODO Task 1 (4.5b Overall Plan Adherence)

- **Assessed by**: architector (verifier), 2026-10-11
- **Authority chain**: `.agent/todos/20261011/20261011-todo-1.md` §1 + standing constraints; implementation plan `.kilo/plans/20261011-phase03-task1-ffmpeg-wrapper.md`; review-fix plan `.kilo/plans/20261011-phase03-task1-review-fixes.md` (df9b00a, 21189b8, 3bb24f1); simplification plan `.kilo/plans/20261011-phase03-task1-simplification.md` (32ee507; S4c+S5 skipped per caller-approved conflict resolution); doc pass 087989f.
- **Scope**: adherence assessment only. No source files modified. Later sub-steps not executed.

---

## 1. Commit chain vs plan commit structure

Plan steps (§2) vs actual `git log` on `feat/phase03-ffmpeg-local-pipeline`:

| Plan commit | Actual | Files touched (verified via `git log --stat`) | Verdict |
| --- | --- | --- | --- |
| Step 1 `feat: add streaming module with typed process errors` | `3bde337` | main.rs (+5), error.rs, mod.rs — **plus error_tests.rs (151 l.)** | MATCH (see D7 staging diff) |
| Step 2 `feat: build ffmpeg raw-video arguments purely from config` | `bb7c5ac` | args.rs, args_tests.rs, mod.rs | MATCH |
| Step 3 `feat: add pure utc timestamp formatting for recording names` | `9c4514d` | utc_timestamp.rs, utc_timestamp_tests.rs, mod.rs | MATCH |
| Step 4 `feat: resolve beside-exe video output with collision-safe names` | `2bcadfc` | video_output.rs, video_output_tests.rs, mod.rs | MATCH |
| Step 5 `feat: define frame sink seam for ffmpeg frame delivery` | `0a51be6` | sink.rs, mod.rs | MATCH |
| Step 6 `feat: supervise ffmpeg process with drain threads and typed exits` | `57c2803` | ffmpeg.rs, mod.rs | MATCH |
| Step 7 `chore: ignore local videos output directory` | `7b19901` | .gitignore (+3) | MATCH (exact 3-line narrow diff) |
| Fix 1 `fix: correct UTC civil date algorithm across non-leap centuries` | `df9b00a` | utc_timestamp.rs (−3), utc_timestamp_tests.rs (+13) | MATCH fix plan |
| Fix 2 `refactor: split oversized StreamingError Display impl and tests` | `21189b8` | error.rs, error_tests.rs | MATCH fix plan |
| Fix 3 `fix: kill and reap FFmpeg child on missing stdin and in Drop` | `3bb24f1` | ffmpeg.rs (+12/−3) | MATCH fix plan; also carries S4c Drop-doc wording ("un-awaited") |
| Simplification one-commit pass | `32ee507` | error.rs, ffmpeg.rs, utc_timestamp.rs, video_output.rs | MATCH S1–S4, commit message and file set exact |
| — (extra) `docs: track task 1 planning documents` | `e007235` | 3 `.kilo/plans/` files +1318 | Extra vs original plan commit structure; benign plan tracking |
| — (extra) `docs: align streaming module comments with task 1 implementation` | `087989f` | utc_timestamp.rs (comment-only, 3 l.) | Extra; the caller-declared doc pass |

**Chain verdict: MATCH.** No commit missing, none out of order, no unknown source commits. Version bump `35ee4ff` (0.6.0) predates the chain as required; `Cargo.toml` verifies `0.6.0`.

## 2. TODO Task 1 bullet-by-bullet

| TODO §1 bullet (requirement) | Evidence in code | Verdict |
| --- | --- | --- |
| External child process; RGB24 via stdin; close-stdin/finish; waits; typed actionable errors | `ffmpeg.rs` (`Command::new` + spawn; `write_frame` → `write_all`; `finish` sets `stdin = None`, joins drains, `wait()`); `error.rs` 9 typed variants with hand-written Display | ADHERENT |
| Arguments without a shell: rawvideo + rgb24 + video-size + framerate + pipe:0; configured codec/preset/bitrate; broadly compatible MP4 config | `args.rs` locked vector `-nostdin -n -f rawvideo -pix_fmt rgb24 -s <w>x<h> -framerate <fps> -i pipe:0 -c:v -preset -b:v -pix_fmt yuv420p -movflags +faststart <out>`; rationale record in module docs (R1–R8); `prepare_piped_command` has no `sh -c` anywhere | ADHERENT |
| MP4 finalized before success; no false success | `finish()` requires `status.success()` AND file exists AND non-empty (`is_output_finalized` → `OutputFinalize` on doubt; conservative direction documented in comment) | ADHERENT |
| No pipe deadlock; no secret/arg-vector leaking; useful diagnostics | Drain threads joined **before** `child.wait()`; `DIAGNOSTIC_CAPTURE_LIMIT_BYTES = 4096` applied to the trailing tail at error-construction time; argument vector never echoed (`ExitFailure.diagnostics` owns stderr text only; `append_read_failure_note` adds read-failure text only) | ADHERENT |
| `videos/` beside executable, create if needed, clear errors otherwise | `video_output.rs`: `current_exe()` → `ExecutableLocation`; `resolve_videos_directory` (parent walk); `create_directory` → `OutputDirectoryCreate` | ADHERENT |
| Distinct demo-/session- timestamped names; collision-safe, tested | `OutputKind` prefixes; `RecordingNameSpec` `_2.._1000` suffixes; `OutputNameExhausted { attempts: 1000 }`; conservative probe `try_exists().unwrap_or(true)`; tests pin all paths | ADHERENT |
| MP4 never committed; narrow ignore rule | `.gitignore` appended exactly 3 lines (`# Local video recordings…`, `videos/`); `git check-ignore -v videos/` → `.gitignore:55:videos/`; `git ls-files videos/` empty; `git status --porcelain` clean | ADHERENT |
| Small frame-sending/finishing interface | `sink.rs`: `FrameSink` (2 methods) + `CompletedVideo` (private field, `path()`, `pub(super)` constructor); process types confined to `ffmpeg.rs` | ADHERENT |
| Independence from YouTube/chat/render types | `mod.rs` docs state dependency rules; imports are only `std` + `crate::config::model`; sink receives opaque `&[u8]` | ADHERENT |

Task-1-relevant acceptance criteria (TODO §98–111): infrastructure bullets (shell-free launch, finalization-before-success, `videos/` placement, collision-safe names, ignore rule) are directly satisfied by the code above; the wiring-dependent criteria (demo command, frame pacing, `run` boundary) belong to Tasks 2–4 and are correctly NOT in this diff (§5 out-of-scope respected — no CLI, demo, pacing, audio, or screenshot code appears).

## 3. Plan adherence (tight items)

| Plan item | Actual | Verdict |
| --- | --- | --- |
| File layout §1.1 (7 files + 4 test files) | All present under `src/streaming/`; test files wired via `#[cfg(test)] #[path = …]` | ADHERENT |
| `mod.rs` docs + private submodule declarations, no re-exports | Verbatim §1.2 paragraph + §1.1 independence bullets + extra access-path note (D11) | ADHERENT |
| `StreamingError` 9 variants, exact fields, `source()` map | error.rs lines 16–55 and 102–115 match §1.3 field-for-field (9 variants) | ADHERENT |
| Display contracts exact strings | All error-test strings byte-match §1.3 (incl. NotFound/broken-pipe hint arms, `exit_failure_description` split) | ADHERENT |
| Locked argument vector order | `args_tests::valid_config_builds_exact_locked_vector` asserts the full `Vec` verbatim incl. 1280x720@25 fixture | ADHERENT |
| Pixel-format defensive guard | `is_non_rgb24_pixel_format` before emitting; test asserts variant + exact Display | ADHERENT |
| Drain-before-wait order, no-unwrap joins | `join_drain` maps missing handle + panicked thread to `WaitFailed` with `io::Error::other`, one argument as plan specified | ADHERENT |
| 4096-byte trailing diagnostics | `trailing_bytes(bytes, DIAGNOSTIC_CAPTURE_LIMIT_BYTES)` at error-build time (not capture time) per plan | ADHERENT |
| Naming contract | Prefixes, `_2..=1000` suffix loop, `OutputNameExhausted { attempts: 1000 }`, no truncation, conservative probe (compile-level `unwrap_or(true)`), clock fallback to 0 documented (D10) | ADHERENT |
| `FrameSink`/`CompletedVideo` shape | Matches §1.7 verbatim (plus necessary `pub(super) new` constructor) | ADHERENT |
| spawn-step sequence §1.8 (build args → piped command → spawn → stdin take → drains → assemble) | Matches, with fix commit 3's kill+reap on missing stdin | ADHERENT |
| `Drop` kills child | Fix commit 3 hardened to kill AND reap; doc wording matches S4c ("un-awaited") | ADHERENT |
| Test suite: 4 args + 7 utc + 10 video + 4 error = 25 streaming tests | Gate log counts: args 4, utc 7, video 10, error 4 → 25; total 106 | ADHERENT |
| Out-of-scope (§5) absent from diff | No pacing/demo/CLI/docs-for-task-2 changes in any Task-1 commit | ADHERENT |

### Forced code-semantic clarifications (each judged)

1. **Leap-day epoch corrected**: plan draft expected `951_868_800`; implementer verified online (unixepoch.net citation in test comment) that 2000-02-29T00:00:00Z = `951_782_400`. Plan §1.5 itself mandated pre-write online verification → **plan-compliant, documented**.
2. **Post-century regression tests** (`post_non_leap_century…`, `leap_day_after_non_leap_century…`): review-fix plan's own additions → **plan-compliant**.
3. **Drain handles as `Option<JoinHandle<CapturedPipe>>`** instead of plan's plain `JoinHandle` fields: required so `finish(self)` can `take()` handles on a struct implementing `Drop`; `None` mapped to a typed error, preserving the plan's "never unwrap" intent → **acceptable, semantics-preserving**.
4. **`epoch_seconds_now()` clock-failure fallback to 0** with rationale comment (suffix uniqueness prevents overwrite): plan did not specify clock-failure behavior; conservative, satisfies the no-overwrite bullet → **acceptable**.
5. **Error-test split honored fix-commit shape** (3 named tests) rather than simplification S5's fixture-fn shape: S5 was skipped by caller-approved conflict resolution; ≤50-line goal met by the fix commit's split → **caller-approved, acceptable**.
6. **`pub(super) CompletedVideo::new`**: minimal internal constructor not in §1.7's sketch; needed by `finish()` → **acceptable** (field stays private, `path()` only accessor).
7. **Extra `main.rs` 2-line comment above `mod streaming;`** and **extra mod.rs access-path note**: doc-level clarifications consistent with self-documenting-code rule → **acceptable**.
8. **Step 1 also staged `error_tests.rs`** (plan Step 1 listed only error.rs/mod.rs/main.rs): the §3 items 14–15 error tests were the deliverable; benign early wiring → **acceptable minor staging diff**.
9. **Extra docs commits** `e007235` (tracked plans) and `087989f` (comment alignment): outside the original commit table but in Task-1 scope and plan-tracking convention → **acceptable**.

Recorded `#[allow(dead_code)]`: exactly one, on `mod streaming;` in `main.rs` with the retire-on-Task-2 note, per plan §4.

## 4. Rule compliance snapshot (.kilo/rules)

| Rule | Finding |
| --- | --- |
| Max lines/file (≤200) | error.rs 179, ffmpeg.rs 190, video_output.rs 167, args.rs 93, utc_timestamp.rs 90, sink.rs 41, mod.rs 30; test files ≤159 — all pass |
| Max lines/method (≤50) | Display `fmt` body ≈42 after fix 2; largest test body ≈41; `start` ≈25; `finish` ≈14 — pass |
| Max depth (≤2) | No nesting beyond 2 levels anywhere in the module — pass |
| Max args (≤2) | Exceptions: `RecordingNameSpec::new` (3× ctor, documented in plan §4), `first_free_suffixed_name(&self, stem, exists)` — same documented parameter-object family as `next_available`; `format_spawn_error` (formatter + 2 payload) introduced by the architector's own fix plan for the ≤50-line rule — documented trade-off, no call-site complexity |
| Single-section booleans | Compound leap-year expression isolated inside descriptive `is_leap_year` (the rule's prescribed remedy); `days_in_year` uses `if is_leap_year(year)`; `is_output_finalized` is itself the plan-specified single-point helper with `output_file_is_non_empty` extracted — pass |
| No commented code | None (only doc comments and explanatory comments; verified across all 11 files) |
| Prefer private members | All fields private; helpers `fn` after S1 narrowed `is_broken_pipe`/`is_file_not_found`; only the seam (`FrameSink`, `CompletedVideo::path`) and within-module `pub(crate)` items are public |
| Newline prevention | Files read cleanly with real line breaks (no literal `\n` content) |
| No global installs / external paths | No installs; all checks through the Compose `rust` service; no tmp paths outside the workdir |

## 5. Final gate evidence

- Log `logs/checks/20261011T232229Z.log` (latest in the directory, all 45 logs reviewed by date): `PASS fmt-check exit=0`, `PASS check exit=0`, `PASS test exit=0`, `PASS clippy exit=0`, summary `ALL CHECKS PASSED`; `running 106 tests` → `106 passed; 0 failed` (81 baseline + 25 streaming = 106, matching the expected post-fix arithmetic 21 + 2 UTC regressions + 2 from the 1→3 error-test split).
- Tracking re-checks fresh on the final tree: `git status --porcelain` clean; `git check-ignore -v videos/` → `.gitignore:55:videos/`; `git ls-files videos/` empty.
- Loose ends correctly deferred to Task 5 by design (plan §2 Step 8): no real-FFmpeg run exists yet (pinned `rust:1.82` image has no ffmpeg; installing is forbidden) — the process layer is compile/lint/unit-test verified only; the NOT-RUN record belongs to the Task-5 completion report. `.agent/project-structure.md` still describes `streaming/` as placeholder — also Task-5 scope (simplification plan §4, last bullet).

## 6. Diffs found (summary)

All deviations are documented and non-blocking:

- D1 leap-day epoch correction (plan-mandated online verification) — plan-compliant.
- D2 UTC algorithm hardened across non-leap centuries (fix commit 1) — fix-plan-driven.
- D3 Display helpers extracted (fix commit 2) — fix-plan-driven; supersedes §1.3's "two arms on `is_file_not_found`" wording with equivalent semantics; locked strings unchanged.
- D4 Error tests split 1→3 (fix commit 2; S5 skipped per caller-approved conflict resolution) — approved.
- D5 Drop reaps; missing-stdin path kills+reaps (fix commit 3) — fix-plan-driven.
- D6 Simplification S1–S4 applied exactly — verified against the 32ee507 diff, byte-consistent with S3/S4 shapes.
- D7 Step-1 staging included error_tests.rs — benign.
- D8 `Option<JoinHandle>` drain fields; `pub(super)` `CompletedVideo::new`; clock-0 fallback; extra comment notes — acceptable clarifications (§3 items 3, 4, 6, 7).
- D9 Extra tracking/doc commits e007235 + 087989f — acceptable.

**No adversarial deviation found. No change plan required.**

## 7. Verdict

**ADHERENT.** TODO Task 1 §1 is fully satisfied, the implementation plan and its two approved amendment plans are faithfully applied, all rule-snapshot checks pass, and the documented final gate evidence (exit 0, `ALL CHECKS PASSED`, 106 tests) is present at `logs/checks/20261011T232229Z.log`. Task 1 is ready to proceed to sub-step 4.6 (next-step planning / Task 2 handoff by the caller).

# Task 1 Implementation Plan — Stop-Path Fix: Isolate the FFmpeg Child from the Console Ctrl+C

Plan date: 2026-10-25 · Author: Architector (4.1b) · Parent: global plan
`.kilo/plans/20261025-stop-path-and-stream-health.md` § Encoded Technical Direction (Task 1);
TODO `.agent/todos/20261024/20261024-todo-2.md` § Task 1. Branch `fix/stop-path-and-stream-health`
at `1b71a19` (v0.11.1 bumped). Front-end: NO. Executor: JUNIOR implementer (4.2).

## §1 Research Verdicts (recorded, with sources)

### 1.1 Windows — `CREATE_NEW_PROCESS_GROUP` (0x00000200)

OFFICIAL (Microsoft Learn, *Process Creation Flags (WinBase.h)*,
https://learn.microsoft.com/en-us/windows/win32/procthread/process-creation-flags —
"Last updated 2025-07-16"): for `CREATE_NEW_PROCESS_GROUP` · 0x00000200 —

> "The new process is the root process of a new process group. The process group includes all
> processes that are descendants of this root process. … If this flag is specified, CTRL+C
> signals will be disabled for all processes within the new process group."

Meaning, applied to this fix: a console Ctrl+C (`GenerateConsoleCtrlEvent(CTRL_C_EVENT, …)`,
what the operator's terminal does) is delivered to all processes attached to the console but
processes in a NEW process group have Ctrl+C disabled by default — they never (re)act to it.
Important honesty note, same page: CTRL_BREAK is NOT disabled by the group flag — a hard
`CtrlBreak` would still reach the group; the documented, supported graceful stop is Ctrl+C
(`docs/production-run.md`), so the limitation is truthful and narrow. The std bridge:
`std::os::windows::process::CommandExt::creation_flags` (stable since Rust 1.9.0) passes the
value as `dwCreationFlags` to `CreateProcess`; no unsafe, no new dependency.

Rx (pipes unaffected): console control events are not pipe traffic — the stdin/stdout/stderr
pips survive verbatim (`docs/production-run.md` incident evidence: the pipes already worked
during the Ctrl+C because FFmpeg streamed normally; only the exit code changed).

### 1.2 POSIX — `CommandExt::process_group(0)`

Rust std (library/std/src/os/unix/process.rs): `std::os::unix::process::CommandExt::process_group`
— "the child process will call `setpgid(pgroup, pgroup)`" — stable since **Rust 1.64** (the
pinned toolchain is `rust:1.82`; no version hazard). Semantics verified by its implementation:
`pgroup` is handed to the platform command's `pgroup()` mutator; with `0` the child calls
`setpgid(0, 0)` BEFORE exec, making the CHILD the leader of a NEW process group (pgid = its own pid).

- Terminal-generated SIGINT (the console Ctrl+C shape on POSIX, per POSIX XCU 11.1.9) targets the
  FOREGROUND process group. After `process_group(0)` the child is NOT a member of the foreground
  group, so it never receives it. **Ctrl+C reaches ONLY our application.**
- The application's `ctrlc` handler (installed via `install_stop_request_handler` /
  `install_interrupt_handler`) REPLACES default SIGINT termination (documented
  `src/app/pipeline/interrupt.rs` limitation 1): so the only observable effect in OUR process is
  the stop flag flipping. Nothing dies prematurely.

Rx (reaping/kill interplay with `FfmpegProcess`): `Child::wait`/`try_wait` reap via
`waitpid(child_pid, …)` (std sources: `wait → waitpid(pid, &status, 0)`, `try_wait →
waitpid(pid, &status, WNOHANG)`) and `Drop`'s `kill` is `kill(child_pid)`. A DIRECT PARENT may
wait/kill its direct child across process-group boundaries — process-group membership never
filters these calls. Verdict: `FfmpegProcess::drop` and `finish()` are unaffected by the
isolation; no change needed there.

### 1.3 The offline test mechanism (container/Linux; std-only)

Mechanism encoded (verified against the codebase's spawn topology):

1. **Deliver the signal to the parent's OWN process group** without libc and without pgid magic:
   - read the test process's pgid from `/proc/self/stat` with `std::fs::read_to_string` ONLY
     (documented Linux procfs format: `pid (comm) state ppid pgrp …` — pgrp is the 3rd
     whitespace field AFTER the final `)`; parse by skipping to the LAST `)` so a comm with
     parens/spaces cannot poison the parse);
   - spawn the sender from the TEST process as `/bin/kill -s INT -- -<pgid>` (GNU coreutils
     `kill`: Debian titles coreutils as `priority:required`, so `/bin/kill` exists in the
     pinned `rust:1.82` image; the `--` guards against the leading-dash pid being parsed as a
     flag); the sender gets `.process_group(0)` TOO, so the group-directed SIGINT does NOT
     signal the sender itself and the sender exits 0;
   - the group's remaining member is the TEST process itself — protected by installing the
     application-shaped handler via the existing `ctrlc` crate (`ctrlc::set_handler` retains
     default-termination-replacement semantics — same mechanism `install_interrupt_handler`
     uses; NOT imported from `crate::app` because the streaming module is sealed to
     std+config and the crate dep surfaces identically).
2. **Zero collateral on sibling tests** (`cargo test` runs tests in parallel threads IN ONE
   process): every child the post-fix code spawns through the production `spawn_with` is
   isolated (own group). The only children spawned OUTSIDE `spawn_with` in the whole suite are
   exactly 4 spawn sites — `src/app/auth/console_tests.rs` L29 (re-exec of the test binary),
   `src/streaming/ffmpeg_real_audio_tests.rs` L49/L89 (real ffmpeg encode), and
   `src/streaming/ffmpeg_real_tests.rs` L40/L130 (ffprobe) — each gains the SAME
   `#[cfg(unix)]`-gated `.process_group(0)` (test-only collateral-immunity change, D6). After
   that, the ONLY non-isolated process in the group is the test process whose handler absorbs
   the SIGINT. Deterministic; no flake window.
3. **Child-survival proof**: before the signal, `child.try_wait()`-shaped aliveness is checked
   via the `FfmpegProcess` public surface? NO — a bounded lifecycle window is asserted after
   the handler flag flips: repeatedly (~20 × 50 ms, named constants) confirm the fake child is
   STILL alive (`finish`-callable) — with the pre-fix (bare spawn) shape the same group SIGINT
   kills the `sh` fake by default disposition, so this test is also a NEGATIVE
   regression pin for the very incident.
4. **EOF finalize**: then `stdin` EOF via `finish()`; the fake script is
   `cat > /dev/null; exit 0` (the existing `STDIN_DRAIN_EXIT_ZERO_BODY` shape): cat reads until
   EOF, exits 0; `finish()` returns `CompletedOutput::Stream(exit_code = Some(0))`.
5. Feasibility: the container runs the POSIX gated rosters today (387-test green gate); the
   Windows side is compile-checked by the standard cross-build + documented
   container-untestable (the global plan's rule).

### 1.4 Rule-cap arithmetic (roster placement decision)

- `src/streaming/spawn.rs` is 77 lines → +~20 (one helper per family + constants) ⇒ ~95–100 ≤ 200. OK.
- `src/streaming/ffmpeg_tests.rs` is EXACTLY 200 lines (at the cap) → the new isolation test
  CANNOT extend it ⇒ D2: NEW sibling roster `src/streaming/ffmpeg_isolation_tests.rs`
  (~150 lines projected), wired in `src/streaming/ffmpeg.rs` with the established
  `#[cfg(all(test, unix))] #[path = "…"] mod …;` pattern. Small helpers (fake-writer, config,
  ETXTBSY retry) are replicated per the existing sibling precedent
  (`ffmpeg_stream_tests.rs` duplicates them; module-private anyway).
- `src/app/auth/console_tests.rs` is 81 lines → +~4 ⇒ fine.

## §2 High-Level Approach

The single shared spawn shaping (`prepare_piped_command`) gains a one-call platform-gated
isolation helper so BOTH `start` (demo/local) and `start_stream` (production RTMPS) inherit it
— the incident is by construction impossible for either path. A new unix-gated process test
proves (through the production spawn path + the app-shaped handler) the reversed incident:
group SIGINT → child survives + app handler flag flips → stdin EOF → child exits 0. Nothing
else changes: args vectors, error types, diagnostics redaction, shutdown order, exit mappings,
demo contract — all byte-stable. `interrupt.rs`'s module-doc limitation list gains one truthful
line about the NEW isolation fact.

## §3 Decisions (encoded; no judgment left)

- D1: helper name `isolate_from_console_control(&mut Command) → ()`, `fn` private in
  `spawn.rs`, ONE per family:
  - `#[cfg(target_family = "windows")]`: `use std::os::windows::process::CommandExt;` +
    `command.creation_flags(CREATE_NEW_PROCESS_GROUP);`.
  - `#[cfg(target_family = "unix")]`: `use std::os::unix::process::CommandExt;` +
    `command.process_group(ISOLATED_PROCESS_GROUP);` where
    `const ISOLATED_PROCESS_GROUP: i32 = 0;`.
- D2: constants live where used, named, each with a doc comment:
  - `spawn.rs`: `CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200` (windows-gated; doc comment
    quoting the MS effect sentence); `ISOLATED_PROCESS_GROUP: i32 = 0` (unix-gated; doc: 0 ⇒
    `setpgid(0,0)` ⇒ the child leads its own group).
  - `ffmpeg_isolation_tests.rs`: `PARENT_GROUP_SIGINT_PROOF_WINDOW_MS: u64 = 50`,
    `PARENT_GROUP_SIGINT_PROOF_ATTEMPTS: u8 = 20`, the fake body constant
    `GROUP_SIGINT_IMMUNE_STDIN_READER_BODY: &str = "cat > /dev/null; exit 0"`, and the
    `/proc/self/stat`-parse helper `child_process_group_from_stat`.
- D3: NO generic abstraction between families, NO new dependency (std CommandExt +
  builtin ctrlc dep; coreutils /bin/kill is image-present tooling, not a dep). NO `allow(`
  attributes anywhere (D12 standing rule).
- D4: `prepare_piped_command` calls `isolate_from_console_control(command);` AFTER the pipe
  config, as the LAST statement of the (still 3-arg-free ≤2-params) builder.
- D5: The new roster: `src/streaming/ffmpeg.rs` gets ONE additional decl:
  `#[cfg(all(test, unix))]` `#[path = "ffmpeg_isolation_tests.rs"]` `mod ffmpeg_isolation_tests;`
  immediately after the `ffmpeg_tests` decl.
- D6: The roster contains TWO tests, exactly:
  1. `stream_child_survives_parent_group_sigint_and_finalizes_on_stdin_eof` — the full
     reversed-incident pin (suite name pins the adjacency: sigint, survival, EOF exit 0).
  2. `stream_child_leads_its_own_process_group` — the structural fact pin: spawn through
     `FfmpegProcess::start_stream`, read `/proc/<child-pid>/stat` via `std::fs` while the child
     runs, parse pgrp, `assert_eq!(pgrp, child.pid)` (isolation is structural, not behavioral).
     NOTE: `FfmpegProcess`'s `child` field is `pub(super)` and the roster is a descendant of
     `streaming::ffmpeg`, so `process.child.pid` is readable in-test (same visibility the
     sibling rosters already exploit).
- D7: The Ctrl+C handler in test 1 is installed DIRECTLY as `ctrlc::set_handler(...)` flipping
  a local `Arc<AtomicBool>` (never via `crate::app::pipeline::install_interrupt_handler` — the
  streaming module stays sealed to std + config even in tests). Handler installation order:
  BEFORE the child spawn so a racy early signal can never terminate the test process.
- D8: The sender: `Command::new("/bin/kill").args(["-s", "INT", "--", &format!("-{pgid}")])`
  with `#[cfg(unix)] use std::os::unix::process::CommandExt;` + `.process_group(0)`, then
  `.output()` (bounded status read) and assert `status.success()` — a sender that FAILED to
  signal (e.g. its own arg parsing changed) must fail THIS test, not silently skip.
- D9: `pgid` derivation helper (unix-only, in the roster):
  `fn process_group_from_stat(text: &str) -> Option<i32>` — skip to the last `)`, trim, take
  whitespace-fields `[1..=2]` (`ppid`, `pgrp` → actually index 2 of the post-`)` list is
  `state; [0]=state, [1]=ppid, [2]=pgrp`), parse `pgrp` as `i32`, `Option`-typed, no panics.
- D10: Collateral-immunity patches (test-only, `#[cfg(unix)]` scoped): add
  `.process_group(ISOLATED_PROCESS_GROUP)` (a single shared constant shape per site; the
  unix `CommandExt` import is module-local in each file):
  - `src/app/auth/console_tests.rs` → the re-exec `Command::new(current_exe)` chain (L29);
    import gated per-file, e.g. `#[cfg(unix)] use std::os::unix::process::CommandExt;`
  - `src/streaming/ffmpeg_real_audio_tests.rs` → `spawn_audio_smoke`'s chain
    (`.spawn()` at L89 — apply the group call BEFORE the final `.spawn()`).
  - `src/streaming/ffmpeg_real_tests.rs` → both `ffprobe` `.output()` chains (L40 `probe_recording`; L130 the second probe helper — read the file first).
  Rationale line (goes IN the added code as a brief doc note): keeps every suite child out of
  the test process group so the isolation pin's group-directed SIGINT can only reach the
  handler-protected test process. These patches are NOT behavior changes to production.
- D11: `interrupt.rs` module-doc extension (LIMIT 1 new numbered paragraph, appended as item 6;
  NO code change in that file):
  > 6. As of the stop-path fix (2026-10-25): the FFmpeg child is spawned out of the console
  >    Ctrl+C reach — Windows via `CREATE_NEW_PROCESS_GROUP` (Ctrl+C disabled for the process
  >    group), POSIX via its own process group (`process_group(0)`); the console signal reaches
  >    ONLY this application. Console/window CLOSE still terminates both abruptly (limitation
  >    above); CTRL_BREAK on Windows still reaches the child group (documented narrow caveat).
- D12 (byte-stability locks): NOTHING changes in: `args.rs`/`stream_args.rs` vectors; error
  types/Display incl. the ExitFailure mapping; `finish()`/`Drop`; demo vector + demo exit codes
  (Interrupt → 1 mapping tests untouched → they must still pass unchanged); every exit-mapping
  unit test; the single-`error:`-line contract; the Task-1 diagnostics warn/info lines and
  constants; the `RUN_STOP_MESSAGE`/`RUN_STARTUP_STOP_MESSAGE` path; shutdown ordering;
  YouTube/auth surfaces. The `streaming_stage_tests.rs` argv pins hold untouched (the
  spawn-level isolation adds NO argv token). Expected test arithmetic: 387 → **389** (+2).

## §4 Tiny Steps (4.2 implementer; each verifiable)

- S1 (0.5): Read the global plan §Tech-Direction + Risks, THIS plan fully, and the four code
  surfaces: `src/streaming/spawn.rs`, `src/streaming/ffmpeg.rs`,
  `src/streaming/ffmpeg_tests.rs` + `ffmpeg_stream_tests.rs`,
  `src/app/pipeline/interrupt.rs`. Create the task stop marker file `.agent/STOP` NOW (empty)
  and remove it at S14 — it guards the scope-shrink signal.
- S2 (kernel): edit `src/streaming/spawn.rs` — add the two namespace-gated constants (D2) and
  the two `isolate_from_console_control` helpers (D1), and extend `prepare_piped_command` with
  the single trailing call (D4). Before/after of the builder:
  - BEFORE (L71–77): `command .args(...) .stdin(...) .stdout(...) .stderr(...);` (7 lines).
  - AFTER: same chain, then a new line `isolate_from_console_control(command);` and below the
    fn the two gated helper fns + constants. Body of each helper = ONE statement.
- S3 (docs line): edit `src/streaming/spawn.rs` module doc: append one doc paragraph naming the
  new duty: "the piped command is isolated from the console's Ctrl+C before spawn (Windows
  `CREATE_NEW_PROCESS_GROUP`; POSIX the child's own process group) so a user stop reaches only
  the application's handler."
- S4 (roster): create `src/streaming/ffmpeg_isolation_tests.rs` with: module doc (purpose =
  the reversed incident pin; the D6/D7/D8/D9 shapes; the ETXTBSY-start precedent note), the
  replicated `write_fake_executable`/`stream_config`/`start_stream_fake` (ETXTBSY retry) trio —
  copied from `ffmpeg_stream_tests.rs` VERBATIM except the fake-script filename becomes
  `"fake-ffmpeg-isolation.sh"` — and the two tests (D6) + constants (D2) + the `pgid` parser
  (D9) + `/bin/kill` sender (D8).
  - Test 1 flow (exact assertable contract):
    1. tempdir; write the fake; `start_stream_fake(&config, FIXTURE_URL)` → a spawned
       `FfmpegProcess` (through the PRODUCTION `spawn_with` path).
    2. install the handler: `let handler_flag = Arc::new(AtomicBool::new(false));` clone for
       the closure; `ctrlc::set_handler(...)` via `.map_err(...).expect(...)`-shaped pattern.
    3. one frame write (`write_frame(&[0u8; 2304])`) proves the pipe is live.
    4. parse pgid of SELF from `/proc/self/stat`; `expect("the container must expose/proc")`.
    5. spawn the sender (`/bin/kill ...` D8) and wait for its success status.
    6. poll: `handler_flag` goes true within a fresh bounded wait; then run the survival
       window: loop `PARENT_GROUP_SIGINT_PROOF_ATTEMPTS` times: attempt a CHILD-side aliveness
       probe — via `std::fs::metadata` on… NO: the aliveness probe uses the same stdout/stderr
       drain shape… CONCRETE mechanism: the fake script writes a heartbeat marker file
       (`echo hi >> marker`)… SIMPLIFY: drop marker-file complexity — the SURVIVAL proof = the
       child state read: the roster access `process.child.try_wait()` (pub(super) field) must
       return `Ok(None)` at each of the ~20 polls — the pre-fix bare spawn would have the sh
       fake DEAD (signal-killed → `Some(signal status)`), so both directions are pinned
       by the same assert: `assert!(process.child.try_wait().expect(..).is_none(), "..")`.
       One `let start = std::time::Instant::now();` bounds the loop honestly.
    7. `let completed = process.finish().expect("stdin EOF must finalize the isolated child");
       `assert `CompletedOutput::Stream(ref exit)` matches and `exit.exit_code() == Some(0)`.
  - Test 2 flow: spawn via `start_stream_fake`; `let pid = process.child.id();`; read
    `/proc/{pid}/stat`; parse pgrp; assert pgrp == pid; then `finish()` cleanly (leave no
    child behind).
- S5 (collateral immunity): apply D10 to the three files (read each fully first). Each edit =
  one `#[cfg(unix)]`-gated `use` + one chained `.process_group(ISOLATED_PROCESS_GROUP)`-shape
  call (test files may reuse a local `const` or the numeric `0` with the same meaning — USE
  the numeric literal `0` per site with a one-line comment pointing at D10 for review
  traceability; no new cross-module constant sharing).
- S6 (interrupt doc): `src/app/pipeline/interrupt.rs` — append limitation item 6 VERBATIM from
  D11 into the module docs (doc comment only; no code line changes).
- S7 (fmt): standard command `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust cargo fmt --check` — pass required.
- S8 (targeted tests): `… rust cargo test --locked streaming` and `… rust cargo test --locked -- isolation`
  — spot-verify: all streaming rosters green COUNT = previous streaming roster count + 2 (the
  exact current streaming count: run and record; the pins: 6 (ffmpeg_tests) + 4 (stream_tests)
  + 1 (real) + 1 (real_audio) + 2 NEW; record the OBSERVED number in the log).
- S9 (interrupt targeted): `cargo test --locked interrupt` — green, unchanged count.
- S10 (clippy): `cargo clippy --locked -- -D warnings` — zero warnings (this also compiles the
  TEST crates on Linux, so the unix-gated edits compile-check globally).
- S11 (FULL gate): `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust sh scripts/dev-checks.sh`
  — exit 0, `ALL CHECKS PASSED`, log path recorded (gitignored `logs/checks/<UTC>.log`),
  observed test count recorded (expect 389). If ≠ 389: re-run + inspect BEFORE continuing.
- S12 (builds + cross-check): `… rust sh scripts/build-linux.sh` and
  `… rust sh scripts/build-windows.sh` — both exit 0; `ls -l` both artifacts (`dist/`-side
  sizes printed by the scripts MUST match the `ls -l` observation; record both byte sizes).
- S13 (hygiene/secret audit): `git status --porcelain` = ONLY the 5 task paths
  (`spawn.rs`, `ffmpeg.isolation roster`, `ffmpeg.rs` decl line, `interrupt.rs`, `console_tests.rs`,
  `ffmpeg_real_tests.rs`, `ffmpeg_real_audio_tests.rs`); `git check-ignore -v dist/…` still
  `.gitignore:34:dist/`; `git grep -n "allow(" -- src/` = the pre-existing 8 attribute lines
  + 1 comment mention (ZERO new); secret greps (`ya29`, `AIza`, `key=`, `rtmps://`-with-… keys
  in NEW text) — nothing; the FIXTURE_URL in the new roster is the SAME `…example.test…`
  fake the sibling roster already uses.
- S14 (marker removal): `rm .agent/STOP`; `git status` clean of the marker.

## §5 Pinned / Byte-Stability Verification Checklist (4.3 reviewer's matrix)

| Surface | Locked shape |
| --- | --- |
| demo/local MP4 argv | `args.rs` untouched; demo tests unchanged pass |
| streaming argv | `streaming_stage_tests.rs` argv contract untouched pass |
| `finish()`/`Drop`/exit mapping `StreamingError` | untouched; all exit-mapping tests pass unchanged |
| demo exit contract (Interrupt→1) | `DemoError` mapping tests pass unchanged |
| `RUN_STOP_MESSAGE` clean-stop path | `app/run/mod.rs` semantics untouched; `runtime_tests` pass unchanged |
| shutdown order (cancel-before-join) | `runtime_tests` shutdown-order proof passes unchanged |
| single `error:`-line contract | all existing tests; no new second `error:` |
| diagnostics constants | `streaming_diagnostics_tests` 3 pins pass unchanged |
| 200-line caps | spawn.rs ~95–100; `ffmpeg_isolation_tests.rs` ≤ 200; ffmpeg.rs ≤ ~158 |
| ctrlc handler count | EXACTLY one handler per process (ctrlc set_handler is a once-API) — the isolation roster must not double-install |

## §6 Command Inventory (all inside the Compose `rust` service via the Alpine VM MCP)

1. `cargo fmt --check`
2. `cargo test --locked streaming` (S8)
3. `cargo test --locked -- isolation` (S8)
4. `cargo test --locked interrupt` (S9)
5. `cargo clippy --locked -- -D warnings` (S10)
6. `sh scripts/dev-checks.sh` FULL gate (S11; log path + count recorded)
7. `sh scripts/build-linux.sh` + `ls -l dist/rust-youtube-streamer-service` (S12)
8. `sh scripts/build-windows.sh` + `ls -l dist/windows/rust-youtube-streamer-service.exe` (S12)
9. hygiene: `git status --porcelain`, `git check-ignore -v dist/...`, the grep audit (S13)

(Exact Compose prefixes: `docker compose -f /rust-youtube-streamer/docker-compose.yml run --rm rust <cmd>`.)

## §7 Commits (per cycle split)

- C1 `chore: track stop-path Task 1 isolation plan` — THIS plan file (4.1b).
- C2 `feat: isolate the FFmpeg child from the console Ctrl+C` — S2–S6 (spawn.rs + roster +
  ffmpeg.rs decl + interrupt doc line + the three test-only immunity patches).
- (Docs edits beyond `interrupt.rs`/code comments belong to the 4.4 docs pass, NOT this cycle.)

## §8 Stop-and-Report Conditions (auto-STOP to the caller)

- The `/proc/self/stat` pgrp parse fails structurally in the container (format surprise).
- `/bin/kill` rejects `kill -s INT -- -<pgid>` (probe result) → STOP; NOT an excuse for a libc dep.
- Test 1 flakiness across ≥ 3 gate runs (survival window races) → STOP; never ship unverified
  (global-plan risk ¶3).
- Any pinned-test/exit-mapping/document drift not byte-stable → STOP.
- Any scope expansion (Task 2 args, resilience hardening, allow attributes) → STOP.

## §9 Acceptance Alignment (TODO Task 1 carve-out; 4.5b architector verifies)

- TODO bullet 1 → D1–D4 (spawn-level isolation, both families, no new dep).
- TODO bullet 2 → D6 (the offline child-survives+EOF-exit-0 pin; Windows compile-checked, documented).
- TODO bullet 3 → §5 stability matrix (S7–S13 + the unchanged tests as the runners);
  `primary_shutdown_outcome`/'+RUN_STOP_MESSAGE' untouched proofs remain in
  `runtime_tests`/`error_tests` (they exercise the SEAMS; no run code edits occur).
- NOT this cycle: adherence (4.5b), DONE marks (4.6), metadata/context refreshes of OTHER
  agents, docs beyond D11, git merge/push.

## §10 Known-Truth Notes for the Implementer

- `ctrlc` multilevel: `ctrlc::set_handler` errors on a second install — no other roster
  installs; if a future roster does, STOP (§8).
- The container is Linux (POSIX cfg active); the windows-gated helper's const compiles ONLY in
  the `x86_64-pc-windows-gnu` build (S12 cross-checks) — the standard gate NEVER compiles that
  helper; if the cross build fails on the const type, STOP with the compiler text.
- The 4.2 executor: ALL decisions are pre-made; deviate ONLY via the §8 STOP procedure.

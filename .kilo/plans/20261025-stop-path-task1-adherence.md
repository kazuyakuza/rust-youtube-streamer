# Task 1 Stop-Path Isolation — 4.5b Overall Plan Adherence Report

Plan date: 2026-10-25 · Author: Architector (4.5b) · Verification scope: the Task-1 chain on
branch `fix/stop-path-and-stream-health` (`7c6cd5d` plan → `aedfaa8` feat → `0cc59df` docs →
`4eb9788` refactor → `10eeefe` plan tracking). Read-only verification; this report file is the
only artifact written by this step.

## 1. Verdict

**ADHERENT WITH ACCEPTABLE DEVIATIONS — zero NON-ADHERENT items.** All 5 caller rulings and all
adjudicated deviations are verified against the tree, the binding plan (`D1–D12`), the review-fixes
plan (§R1) and the simplification plan (§S1). Every TODO Task-1 bullet and every Task-1-scoped
acceptance criterion is mapped to observed evidence below. Clear to proceed to 4.6.

## 2. Code-Level Verification (D1–D12 vs the tree at HEAD `10eeefe`)

| Decision | Evidence (observed in the working tree / commits) | Status |
| --- | --- | --- |
| D1 — per-family private helpers | `src/streaming/spawn.rs` L86–98: two `#[cfg(target_family = …)] fn isolate_from_console_control(&mut Command)` bodies, each one statement (windows `creation_flags(…)`; unix `process_group(…)`), `use` INSIDE each fn | ADHERENT |
| D2 — named constants where used | `spawn.rs` L104–110: `CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200` (windows-gated, doc comment quoting the Microsoft effect sentence) + `ISOLATED_PROCESS_GROUP: i32 = 0` (unix-gated, setpgid(0,0) doc). Roster consts: `STOP_SIGNAL_SURVIVAL_WINDOW_MS/ATTEMPTS` (post-R1 names), `GROUP_SIGINT_IMMUNE_STDIN_READER_BODY`, `HANDLER_FLAG_WAIT_*` (the caller-authorized redesign's bounded poll), + the ETXTBSY trio | ADHERENT |
| D3 — no generic abstraction / no new dep / no `allow(` | One helper per family (cfg alternatives; simplifier D-4 recorded "kept, canonical"); no Cargo.toml/lockfile change in `aedfaa8`/`4eb9788`; `allow(` grep recounts 8 attribute lines + 1 comment mention — ZERO new | ADHERENT |
| D4 — builder's last statement | `spawn.rs` L75–82: `prepare_piped_command` ends with `isolate_from_console_control(command);` after the pipe chain | ADHERENT |
| D5 — roster decl placement | `src/streaming/ffmpeg.rs`: `#[cfg(all(test, unix))] #[path = "ffmpeg_isolation_tests.rs"] mod ffmpeg_isolation_tests;` immediately after the `ffmpeg_tests` decl (diff `aedfaa8`, hunk at L143) | ADHERENT |
| D6 — two pinned tests (post-R1 names) | `ffmpeg_isolation_tests.rs` L136 `stream_child_survives_direct_stop_signal_and_finalizes_on_stdin_eof` + L179 `stream_child_leads_its_own_process_group`; both green in the final gate log (`20261025T035920Z.log` L370/L569: `…ffmpeg_isolation_tests::stream_child_leads_its_own_process_group … ok`, `…stream_child_survives_direct_stop_signal_and_finalizes_on…` ok) | ADHERENT |
| D7 — direct `ctrlc::set_handler` before spawn | L142–148: local `Arc<AtomicBool>` flag via `ctrlc::set_handler`, installed BEFORE `start_stream_fake`; no `crate::app` import (module stays std+config-sealed) | ADHERENT |
| D8 (post-ruling) — direct-pid sender | `send_stop_signal_to_test_process` L89–100: `/bin/kill -s INT <std::process::id()>`, NO group operand, NO `.process_group(0)` (removed per ruling 2 — verified absent), sender success asserted with captured stderr; a failing sender fails the test | ADHERENT |
| D9 — pgid parse helper | `process_group_from_stat` L80–85: `rsplit_once(')')` + whitespace fields (state/ppid/pgrp), `Option<i32>`, no panics | ADHERENT |
| D10 — collateral immunity | 4 patch sites in `aedfaa8`: `console_tests.rs` (`#[cfg(unix)]`-gated import + gated call, one-line D10 comment), `ffmpeg_real_audio_tests.rs` (`spawn_audio_smoke` + `probe_audio_smoke`), `ffmpeg_real_tests.rs` (`probe_recording`) — numeric `0` + one D10 comment each. Deviation vs D10's guessed sites: the plan's "ffmpeg_real_tests L130 second probe helper" does not exist in the tree (the roster has ONE ffprobe chain after the sweep) — the row was re-derived to the actual out-of-spawn site set; recorded, acceptable | ADHERENT (site re-derivation recorded) |
| D11 — interrupt item 6 | `src/app/pipeline/interrupt.rs`: item 6 appended VERBATIM from the plan text (doc-comment only; diff `aedfaa8` = 5 added doc lines); the list is 6 items | ADHERENT |
| D12 — byte-stability locks | `git show aedfaa8 --stat` = EXACTLY the 7 expected files, +272/−10; no edit to `args.rs`/`stream_args.rs`/error types/run exit mapping/`RUN_STOP_MESSAGE`/diagnostics constants/auth/YouTube surfaces; 4eb9788 touches ONLY the roster (test-only). Test arithmetic 387 → 389 verified in both logs | ADHERENT |

Recounted rule caps (fresh `Get-Content` counts): `spawn.rs` 110, `ffmpeg_isolation_tests.rs` 197,
`console_tests.rs` 88, `ffmpeg_real_tests.rs` 113, `ffmpeg_real_audio_tests.rs` 169 — all ≤ 200;
helper/function bodies ≤ 50 lines; ≤ 2 args; no max-depth breach; no commented-out code anywhere in
the diffs. The `.agent/STOP` marker is absent.

Byte-stability ledger re-verified: both gate logs (feature `20261025T034206Z.log` L573 and FINAL-tree
`20261025T035920Z.log` L572) read `test result: ok. 389 passed; 0 failed; 0 ignored; 0 measured` +
`ALL CHECKS PASSED` — the §5 matrix (demo Interrupt→1, single-`error:`-line, shutdown order,
diagnostics constants, argv pins) passed unchanged in both runs. Artifacts Linux 5227464 B /
Windows 8646799 B recorded from the 4.2 tree; `4eb9788` is test-only so those sizes govern.

## 3. The 4.4 Edits (uncommitted; belong to 4.6's commits)

- `README.md` L46: "The containerized test suite is green with 389 tests" — the single authorized
  integer edit 387 → 389; observed in the working tree (uncommitted). TRUTHFUL.
- `.agent/project-info/context.md`: the 2026-10-25 blocks (4.2 record + the 4.3/4.4 pass record) are
  present in the working tree (uncommitted). The 4.4 record truthfully states the
  `docs/production-run.md` NO-EDIT verdict — independently re-verified: a grep of ALL of `docs/` for
  `process group|process_group|CREATE_NEW_PROCESS_GROUP` returns ZERO hits, and the shutdown
  bullet's existing text never asserts the console signal reaches FFmpeg; no contradicting claim
  exists. TRUTHFUL.
- Both files are correctly OUTSIDE the tracked HEAD `10eeefe` (`git status --porcelain` shows only
  these two `M` entries) — flagged for 4.6: they must be committed with the 4.6 completion commit,
  per the established 4.4-until-4.6 convention.

## 4. Caller-Ruling Adjudication

| # | Ruling | Classification | Note |
| --- | --- | --- | --- |
| 1 | §8 STOP + test-1 direct-pid redesign + the honest 4-point proof structure | ADHERENT WITH ACCEPTABLE DEVIATION | The STOP fired exactly on a pre-encoded §8 condition (group operand rejected in the pgid-1 container); the redesign is documented in the roster's module doc + the honest-proof doc comment (a–d points present verbatim in effect); the reviewer file's §0 and the simplifier §0 both honor it. Acceptable: the deviation replaces the plan's own D6-test-1/D8 mechanism for a container-reality reason, and the honest consequence (the group-broadcast half of the proof is pinned statically instead of behaviorally) is recorded, never hidden. |
| 2 | Sender `process_group(0)` removal | ADHERENT | Verified absent in the tree; the success assert retained (D8's failure semantics hold). |
| 3 | The 7 recorded deviations | ADHERENT WITH ACCEPTABLE DEVIATION | The verifiable deviations are: (a) test-1 direct-pid redesign (+ honest proof structure); (b) sender wrapper removal; (c) D10 site re-derivation (4 actual sites vs the plan's 5 guessed lines); (d) the S1.2 `replaceAll` slot replaced by two per-test unique-anchor edits (S1-file helper comment placement forced per-site edits); (e) roster stays 197 lines, not the simplification plan's ~193 projection (the diffstat is 21+/21− net zero — the plan's arithmetic ignored the 12-line helper addition). Each is documented, behavior-neutral or caller-authorized, and none touches a §8 stop-forbidden surface. |
| 4 | Docs-commit derivation (`0cc59df`) | ADHERENT | Diff verified: ONLY `.agent/project-info/context.md` (+8) and `.agent/project-structure.md`; the structure map's streaming 23 files / spawn duty / roster entry / interrupt 6 limitations / v0.11.1 lines are all TRUE against the final tree; the README 387→389 drift was correctly left for 4.4. |
| 5 | The 2 mechanical deviations in the 4.3-fix (per-test anchors; roster 197) | ADHERENT WITH ACCEPTABLE DEVIATION | §R1 rename + §S1 fold applied VERBATIM in `4eb9788` (full diff inspected): the constants, the test name, and the helper are byte-for-byte the plans' prescribed text; the epilogue fold preserves both test semantics; the roster line count 197 ≤ 200 is the binding fact; the ~193 projection was arithmetic, not code. |

## 5. Criteria Matrix (TODO Task 1 bullets + Task-1-scoped criteria)

| Requirement (Task 1 scope) | Evidence | Verdict |
| --- | --- | --- |
| Bullet 1 — shared spawn plumbing isolates the child (`CREATE_NEW_PROCESS_GROUP` / own process group; platform-gated helpers; no new deps; no other-surface change) | `spawn.rs` L75–110 (D1/D2/D4); `aedfaa8` stat confirms zero other-surface edits; grep `allow(` recount = pre-existing only | ADHERENT |
| Bullet 2 — offline containment pin (child survives the stop signal, finalizes cleanly on stdin EOF; Windows compile-checked via cross-build + documented container-untestable) | The two green pins (D6 post-R1, both `ok` in `20261025T035920Z.log`); the structural `pgid == child pid` pin; Windows helper compile-checked by the `x86_64-pc-windows-gnu` cross-build (S12, exit 0); container-untestable honestly documented in the roster doc + honest-proof structure | ADHERENT |
| Bullet 3 — post-fix clean-stop expectations (FFmpeg exit 0, clean-stop outcome, shutdown-order tests hold, no pinned drift) | Byte-stability ledger D12: `runtime_tests` shutdown-order + clean-stop proofs and exit mappings pass unchanged in both 389/0 gates; the roster pins the reversed incident (survival + EOF exit 0); no pinned line drifted | ADHERENT (live operator behavior remains the pending manual E2E — never agent evidence) |
| AC1 (Task-1-scoped) — Ctrl+C no longer reaches the child (platform-isolated spawn) + the pinned offline test | = bullets 1+2 above | SATISFIED (Task-1 scope; AC may only flip at 4.6) |
| AC2 (Task-1-scoped) — user stop → clean-stop outcome (exit 0 + stop message); fatal mappings + single-`error:`-line byte-stable | `RUN_STOP_MESSAGE` path untouched; `error_tests`/`runtime_tests`/`demo` mapping tests pass unchanged in the gates | SATISFIED (Task-1 scope; the real operator Ctrl+C behavior stays operator-observed) |
| AC5 (Task-1-scoped portions) — gate green on final tree w/ recorded log + count; artifacts rebuilt + `ls -l`; hygiene/secret audit | Gate 389/0 on the FINAL tree (`20261025T035920Z.log`, observed locally); artifacts 5227464/8646799 B recorded and diff-stat-stable (4eb9788 test-only); hygiene: `git status --porcelain` = the 2 known uncommitted 4.4 files only; `.agent/STOP` absent; secret greps clean (only the sibling-shared `…ingest.example.test…` fixture shape) | SATISFIED for Task 1's cycle (whole-TODO closure at 4.6 after Task 2) |
| AC6 (Task-1-scoped portion) — truthful docs/structure/context; no real-Google verification claims | `0cc59df` verified truthful; the uncommitted README/context 4.4 edits truthful; §2/§3 above; zero real-Google claims anywhere in the diffs (live stop-path behavior stays operator-observed) | SATISFIED (Task-1 scope) |
| AC3/AC4 | Task 2 scope — NOT adjudicated here | n/a (out of scope) |

## 6. Unverifiable / Environment-Notes

- The gate logs and `dist/` artifacts live VM-side; this report re-observed the two 2026-10-25 gate
  logs directly (both present on the shared mount and inspected: 389/0 + `ALL CHECKS PASSED`) and
  verified the refactor's test-only diffstat ensures the recorded artifact sizes govern. No fresh
  build re-run was performed by this read-only step — not required (task is adjudication, not
  re-execution).
- The container pgid-1 reality and the `/bin/kill` rejection probe are caller-recorded facts
  (exact probe output quoted in the caller context); the resulting test redesign is structurally
  verified in the tree — the runtime probe itself is not re-observable from this step.
- Plan arithmetic note: the implementation plan's §1.3 items 2–3 (helper-collision) and the
  simplification plan's §1 (~193) contain stale projections superseded by the recorded deviations;
  the binding facts (caps, counts, semantics) hold.

## 7. Undefined-Outcome Items

None. No corrective proposal required. For 4.6: commit the two outstanding uncommitted 4.4 files
(`README.md`, `.agent/project-info/context.md`) with the completion marks, then the workflow step 5
(rename → merge → push `origin` ONLY).

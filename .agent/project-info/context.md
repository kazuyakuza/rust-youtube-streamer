# Context

> Source of truth: [`brief.md`](brief.md). Living status log — read at the start of every task and update it before finishing, per [`instructions.md`](instructions.md).

## Current Work Focus

Phase 00 TODO has been created in `.agent/todos/20261008/20261008-todo-2.md`. It defines the initial Cargo/build baseline only; no runtime service, Windows service registration, FFmpeg installation, or application feature implementation is part of this phase.

## Recent Changes (2026-10-08)

* `brief.md` defined as the complete technical brief and source of truth for the Rust YouTube Streamer Service.
* Project-info template customized to this project (`instructions.md` core-file structure in place).
* `README.md` replaced with initial project info for the Rust YouTube Streamer Service (detailed README deferred to future sessions).
* Initial structure scaffolded: `config/`, `credentials/`, `fonts/`, `logs/` and `src/{app,config,youtube,chat,renderer,streaming}` folders with `.gitkeep` placeholders; `.gitignore` logs rule adjusted; `project-structure.md` updated.
* Verified current repository contents and agent workflow conventions before creating the next TODO. Existing completed TODO is `.agent/todos/20261008/20261008-todo-1-DONE.md`; the new Phase 00 task is numbered 2.
* Created `.agent/todos/20261008/20261008-todo-2.md` to define Phase 00 — Repository Foundation and Build Baseline.

## Immediate Next Steps

1. Execute `.agent/todos/20261008/20261008-todo-2.md` using the repository's critical workflow.
2. In later phases, re-read the current repository and project info before drafting each new TODO, as requested.
3. Add the separate runtime prerequisites/permissions document and link it from README in the appropriate later documentation phase; it must state FFmpeg is a preinstalled external prerequisite, the executable runs as a normal user from a writable/readable location, and the app is not installed or registered as a Windows service.

## Scope Decisions

* The application is a normal executable, not a Windows service. Do not implement service registration, service wrappers, elevation, or installer behavior.
* FFmpeg is an external prerequisite that the user installs independently. The application must use the configured executable path and report configuration/process errors clearly; this project does not install FFmpeg.
* Documentation and automated tests should be included in phases where appropriate, not deferred exclusively to the final phase.

## Notes

`brief.md` is the source of truth; if any project-info file conflicts with it, `brief.md` wins.

# Task 3 Plan — Game-Foundation Handoff Documentation (docs-only)

Source TODO: `.agent/todos/20261031/20261031-todo-1.md` Task 3 (pp. 103–159; global constraints pp. 13–23 + Global Acceptance Criteria pp. 161–168 bind).
Global plan: `.kilo/plans/20261101-observability-game-handoff.md` (Task-3 pre-analysis binds).
Branch: `feat/observability-game-handoff` (HEAD `27586ba`, v0.14.0, gate OBSERVED 550/0). No branch creation/switch.
Front-end determination: **NOT front-end** (no 4.1a/4.5a; Rust CLI/service, no UI).
Writer mapping (BINDING — markdown-generation rule): the Planner/Architector plans; the **docs-specialist WRITES** all markdown in 4.2 (the 50%-restricted junior implementer may not create/modify `.md` files); the **code-reviewer reviews** in 4.3 (factual/rationality review of every claim against the §6 ledger); any required fixes are applied by the **docs-specialist in the 4.4 alignment pass** (Tasks 1–2 repo precedent); the 4.6 implementer **only commits**.

---

## 1. Pre-Analysis

### 1.1 Documentation-gap audit (read files; real gaps)

Existing docs and what each already covers (link targets, NOT duplication candidates):

| File | Covers today | Relevant gap for Task 3 |
| --- | --- | --- |
| `README.md` | Modes/commands, exit codes, RUST_LOG, demo quick start, OAuth, production run summary, observability + latency-test-map pointers (line 127) | No handoff/extension guide link; no "where game code belongs" pointer. Gains ONE AI-Agents bullet. |
| `docs/production-run.md` (incl. the Task-1 `## Pipeline Observability (Opt-In Diagnostics)` H2 + the Task-2 cross-link to `docs/latency-test-map.md`, line 378) | Operator startup stages, log lines, shutdown/fate, Live Chat section, contained-panic, ingest probe, stream-health note, diagnostics operator guide, manual verification. The ~35 s figure appears at line 375 as an honest external-operator observation. | No architecture/extension content — correct for scope. Gains only a See-Also row (do NOT touch operator procedures). |
| `docs/latency-test-map.md` (142 lines, Task 2) | Boundary-by-boundary metric map, capacity math, offline scenario tables, harness mechanics, operator live procedure, results template, interpretation matrix, threshold stance. Frames the ~35 s as "a measurement to investigate" (line 23). | Already the canonical measurement page — the guide LINKS it for timing facts; gains only a See-Also row. |
| `docs/configuration.md` | Field reference incl. the full `diagnostics` section; 27 semantic rules; CLI/exit codes | No extension-point framing. Gains only a See-Also row. |
| `docs/authentication.md` | `auth` flow, one-grant-one-channel, credential files, recovery | No change (guide links it for the OAuth boundary sentence). |
| `docs/google-cloud-prerequisites.md` | Google Cloud setup, quota model, channel live access | No change. |
| `docs/demo.md` | Demo timeline/prereqs/output/Ctrl+C/limits | No change (the guide links it as the offline path's operator doc). |
| `docs/build.md` / `docs/build-windows.md` | Release builds | No change. |
| `.agent/project-info/brief.md` | Source of truth: intended architecture (§2.2/§2.3 flow diagrams, §4 structure, boundary rules) | Describes INTENDED architecture only — the guide's job is the ACTUAL tree; brief stays untouched. |
| `.agent/project-info/architecture.md` (80 lines) | STALE: reproduces brief §2.2 verbatim, says "For current repo state, see `context.md`"; module table mirrors brief; NO diagnostics/`run/`/`demo/` module rows; no actual data flow | Real gap: add a current-implementation snapshot pointer + diagnostics row + the independence bullet. |
| `.agent/project-info/tech.md` (60 lines) | Stack/runtime/confined-Tokio facts, diagnostics Task-1/Task-2 bullets (lines 41–43) — already accurate | Lasting Task-3 fact: the handoff guide exists + the five separation lines summary. One bullet. |
| `.agent/project-info/context.md` | Living status log; Tasks 1–2 (observability) have landed blocks | Needs the Task-3 dated block + next-step. |
| `.agent/project-structure.md` | Accurate current per-file map | NO change (tree unchanged). |
| `AGENTS.md` | Project-info link hub | NO change (no new project-info files are added). |

Top findings (the real gaps the guide must fill — none of them exists anywhere today):

1. No document maps the ACTUAL runtime data flow to real source-module paths (the README describes behavior, not modules; project-info architecture.md still shows the brief's conceptual diagram and defers to `context.md`).
2. No document separates reusable-unchanged components from extension points (`MessageSource` seam at `src/app/pipeline/source.rs`, the `Renderer` trait at `src/renderer/renderer.rs`, the demo source schedule at `src/app/demo/schedule.rs` are never presented as seams for future work).
3. No document states the independence rule ("game logic must not depend on YouTube API types, OAuth, broadcast IDs, RTMPS URLs, or FFmpeg command details") outside brief §2.2/§9.1 phrasing — and nothing maps it onto the actual module boundaries.
4. No document derives game-orientation guidance from the timing facts (latency-tolerant mechanics as DEFAULT; viewers ≠ throughput; bounded queues; 1080p/4K affects memory/encoding cost, not chat timing semantics).
5. The 10-item agent guidance (TODO pp. 129–139) does not exist anywhere.
6. No decision checklist and no "what not to change" list exists for future agents.
7. No guide tells a future agent how to develop game behavior fully offline (demo + offline harness) without OAuth/Google contact.

### 1.2 Verified current-state code facts (guide inputs, with citations)

Verified by direct file inspection on HEAD `27586ba`; every claim in §5's ledger cites one of these files.

Modes and startup funnel:
- `src/main.rs:5–15` — process entry delegates to `app::run`; module roots `app`, `chat`, `config`, `renderer`, `streaming`, `youtube`.
- `src/app/cli.rs` + `.agent/project-structure.md` line 6 — `auth | demo | run | --config <path> (before/after) | --help`; `DEFAULT_CONFIG_PATH = config/config.json`.
- `src/app/mod.rs:40–133` — startup funnel parse → init logging → load+validate config → dispatch; 3 whitelisted-field info events (`mode selected`, `configuration loaded and validated`, `mode finished`); `report_failure` = ONE stderr `error:` line; exit contract 2 (usage) / 1 (config+log-init and mode-origin fatals).
- `src/app/auth/mod.rs` + `flow.rs` + `listener.rs` + `console.rs` — interactive OAuth flow (10-variant `AuthError`, every failure exit 1, redaction contract; loopback-only listener; URL brochure; no auto browser launch).
- `src/app/run/mod.rs:1–7` — run-mode module docs: the 10-stage startup; resource fate (D-T3-6 rule 8): NEVER delete/transition, `enableAutoStop=true` auto-completes, post-O1/O2 failures leave `ready` resources.
- `src/app/run/mod.rs:107–161` — `run_runtime` order: credential service → scope gate → transport bundle (`GoogleLiveApiTransport` + `GoogleLiveChatStreamTransport`) → broadcast preparation → the ONE destination reveal (`RtmpsDestination::new(prepared.into_ingestion().into_url())`, line 129) → `DiagnosticsCollector::from_config` (line 130) → chat stage build → gate spawn → streaming runner.
- `src/app/run/gate.rs` + `gate_diagnostics.rs` — post-start confirmation thread (`streamStatus == active`, 120 s budget), `GateFinish { Confirmed, Cancelled, Fatal, Panicked }`, chat worker spawned ONLY at `Confirmed`; watch-link info event.
- `src/app/run/stop_pair.rs` — the app's single Ctrl+C handler install, flipping the frame-loop `StopFlag` and the youtube `WaitCanceller`.
- `src/app/demo/mod.rs` + `schedule.rs` + `source.rs` + `.agent/project-structure.md` lines 37–41 — local-only demo: fixed timeline (28 messages, 30-frame lead-in, ≤1 message per 15 frames, 60-frame tail → 496 frames ≈ 16.5 s at 30 FPS), ids `demo-NN` hidden, FIFO eviction exercised, exit contract (0 only after verified finalization; failures exit 1), never contacts YouTube.

Chat retrieval → frame loop (the message path):
- `src/youtube/stream_proto.rs` / `stream_transport.rs` / `stream_client.rs` / `stream_concrete.rs` / `live_chat_stream_mapping.rs` — the published-proto wire types (hand-authored prost, no codegen), the mockable `ChatStreamTransport` seam, the tonic adapter (Bearer metadata only), the concrete establishment (`grpc.ready()` reserve → server-streaming call, `STREAM_READY_FAILURE_REASON` fixed detail), proto-batch → `ChatTextMessage` mapping (TEXT snippet type only, displayName→channelId fallback, RFC3339→epoch).
- `src/youtube/live_chat_resolution.rs` — O6 `liveChatId` resolution, bounded 6 × 5 s budget; `src/youtube/live_chat_dedup.rs` — 512-id seen window.
- `src/app/run/chat_worker.rs:1–60` — worker module docs + constants: confined current-thread Tokio runtime on the ONE worker thread; `CHAT_CHANNEL_CAPACITY = 64`; `CHAT_SLEEP_SLICE_MS = 1_000`; `CHAT_BACKOFF_MAX_MS = 60_000`; `CHAT_MAX_CONSECUTIVE_TRANSIENT_FAILURES = 10`; D-13 redaction (message text/authors/user-message ids NEVER logged; only the non-secret `live_chat_id`). v0.13.1 containment: `std::panic::catch_unwind(AssertUnwindSafe(...))` around `block_on` (imports at lines 32–33) → typed terminal with `CHAT_WORKER_PANIC_DETAIL`, never the payload.
- `src/app/run/chat_stream_worker.rs` / `chat_stream_delivery.rs` / `chat_worker_helpers.rs` — resumable loop (establish → 1 s stop-ticked select → consume; per-connection state), delivery (history-FIRST baseline discard; `try_send`-only; bounded re-establishment with the retained `nextPageToken`), the worker-side diagnostics hooks (`report_batch_received`, `note_worker_receipt`, `report_enqueue_*`, `report_reconnect`).
- `src/app/run/chat_channel.rs` — the bounded `std::sync::mpsc::sync_channel(64)` + terminal cell + the confined runtime builder.
- `src/app/run/source.rs` — `ChatSource`: non-blocking `try_recv` draining; yields `Ok(None)` while the channel is open and empty (the frame loop never blocks on chat).
- `src/app/pipeline/runner.rs:54–66` — the frame loop accepts AT MOST ONE message per frame iteration (nominal consumer capacity = the configured FPS messages/second); never names `FfmpegProcess` (the sink arrives as `S: FrameSink`).
- `src/app/pipeline/{clock,pacing,interrupt,error,source,source_error}.rs` + `runner_diagnostics.rs` + `runner_prepare.rs` — monotonic clock, FPS-paced schedule, `StopFlag`, typed errors, the `MessageSource` seam + `ChatSourceError { Chat, Retrieval }`, the diagnostics hooks (`Option`-guarded), the production constructors.

Frame path and streaming:
- `src/chat/mod.rs:1–22` + `model.rs` + `store.rs` — app-level message model (Display exactly `author: text`; metadata off-screen) and the bounded visible FIFO `ChatStore` (log-append-first; capacity derived from geometry).
- `src/renderer/mod.rs:1–37` — boundary: "a pure local component: no YouTube API types, chat state, network, or FFmpeg dependency"; `#[path = "renderer.rs"] mod contract` holds `RenderInput` + the `Renderer` trait; `frame.rs` (checked `width × height × 3` RGB24), `text.rs` (ab_glyph, parse-once font, clipping, no wrapping), `settings.rs` (construction-validated).
- `src/streaming/mod.rs:1–50` — sealed boundary: depends only on `std` + `crate::config::model`; frames arrive as opaque `&[u8]` via `FrameSink`; `args.rs` (locked local vector) / `stream_args.rs` (locked streaming vector with the derived CBR block + optional libmp3lame silent track) / `ffmpeg.rs` (spawn/build-over-one-engine, capture-time `finish()` redaction) / `spawn.rs` (no shell; console-Ctrl+C isolation) / `pipe_drain.rs` / `sink.rs` / `local_finalize.rs` / `rtmps_destination.rs` (redacting wrapper, one reveal) / `video_output.rs` / `error.rs`.

Config:
- `src/config/mod.rs:1–34` + `model.rs` + `loader.rs` + `validation.rs` + `diagnostics_config.rs` — typed serde schema mirroring `config/config.example.json`, `deny_unknown_fields`, 27 semantic validation rules (10 required strings, dims/FPS, exact `rgb24`, privacy enum, OAuth paths + distinctness, 6 diagnostics rules), the optional `DiagnosticsConfig` sibling.

YouTube boundary (module docs):
- `src/youtube/mod.rs:1–50` — boundary rules: owns all OAuth/token material behind redacting wrappers; must never depend on `chat`/`renderer`/`streaming`/demo; the Phase-07 async seam is the sanctioned exception with its runtime confined to the worker thread; every test boundary is a fake or `127.0.0.1` loopback (no test reaches Google). Lifecycle family: `api_transport.rs` (the five-operation seam), `google_live_api_transport.rs`, `live_ingestion.rs` (O1→O2→O3 bind→readiness→destination), `readiness_wait/{mod,confirmation_wait,observation}.rs` (180 s/5 s, 120 s/5 s), `rtmps_ingestion.rs` (redacting destination wrapper, `into_url()` = the ONE reveal), `wait_cancellation.rs`, `live_error/*`. Credential family: `credential_service.rs` (load→verify-first→refresh-once→persist→re-verify), `file_credential_store.rs` (atomic save, POSIX 0600), `persisted_credentials.rs` (frozen 4-key schema), `google_authorization_verifier.rs`, `authorization.rs` (`REQUIRED_SCOPE` = `youtube.force-ssl`).

Diagnostics (Task 1) + harness (Task 2):
- `src/app/diagnostics/mod.rs:26–54` — the ONE construction point `DiagnosticsCollector::from_config` → `None` unless any capability is enabled (disabled config allocates nothing) + `frame_interval_nanos(fps)`; `collector.rs`/`collector_stages.rs` (hooks + queue rows, `Option`-guarded per site), `state.rs` (fixed counters + 6 histograms + 128-entry capped in-flight map), `histogram.rs` (9 fixed buckets), `events.rs` (~`CLOCK_DOMAIN_NOTE` verbatim in every summary), `writer.rs` (lazy-open JSONL, per-row flush, size-cap rotation ≤ `max_file_bytes × max_generations`).
- `src/app/run/latency_load_fixtures.rs` + `latency_load_source.rs` + the rosters in `.agent/project-structure.md` line 30 — the `#[cfg(test)]`-only offline harness driving the REAL channel + `ChatSource` + `FramePipeline::run` on fake clocks (`RateProfile { numerator, denominator }` integer credit arithmetic; every rate derived from configured FPS; capacity from `CHAT_CHANNEL_CAPACITY`).

### 1.3 Verified doc conventions (the guide MUST follow them)

- H1 title; a short intro paragraph WITHOUT an H2; then `## Table of Contents` with `- [Title](#slug)` bullets (lowercase, hyphens, punctuation stripped — matches every existing doc).
- H2 `## Sections`; H3 `### Subsections`; final `## See Also` with bulleted relative links (from `docs/`: `production-run.md`, `../README.md`, etc. — the `docs/latency-test-map.md` pattern).
- Attribution honesty: "operator-run" / "NOT RUN by the agent" / "container-only" phrasing preserved wherever live-vs-offline claims are made; the ~35 s observer-baseline is cited as the operator-observed measurement (TODO p. 5 wording: "operator-observed baseline of approximately 35 seconds") — never as a platform constant and never as agent evidence.
- Real newline characters everywhere (newline-prevention rule); no literal `\n` in any written file.
- Focused pages with a TOC when > 100 lines (TODO p. 150); operator procedures are LINKED, never duplicated (latency-test-map precedent, its intro sentence p. 3).

### 1.4 Project-info update policy (from `.agent/project-info/instructions.md`)

- Updates occur when the implementation reveals lasting facts/next steps; review ALL project files before updating; `context.md` MUST be updated before finishing (Critical Closing Step); `AGENTS.md` must link all project-info files, so it changes ONLY if a new project-info file is added — **this task adds NONE**, so `AGENTS.md` stays untouched.
- `brief.md` stays the source of truth and is NEVER edited by this task.
- Context-window rule: the `context.md` Task-3 block is authored in 4.2 wording (draft) and committed at 4.6 — the same author-then-commit pattern as Tasks 1–2.

---

## 2. FINAL Guide Design

### 2.1 File name (exactly ONE decision)

**`docs/game-foundation.md`** (the docs/ folder has no collision; the name encodes the TODO's "foundation" language).

### 2.2 Guide title + code-fact version label

H1: `# Game Foundation Handoff` — first sentence anchors it to the ACTUAL tree: "This guide describes this repository as it exists at v0.14.0 (branch state at implementation time) — the actual code paths and contracts, not the brief's intended architecture — and tells a future implementation agent (human or AI) how to build a chat-controlled game on top of it without breaking the streaming infrastructure."

### 2.3 Section tree (H2 count = 10, H3 count = 15)

```
docs/game-foundation.md
├── ## Table of Contents
├── ## Purpose and Contract
├── ## Executable Modes and Their Responsibilities
│   ├── ### auth (Interactive, Once)
│   ├── ### run (Production Broadcast Lifecycle)
│   └── ### demo (Local-Only Offline Recording)
├── ## Current Runtime Data Flow (Actual Source Modules)
│   ├── ### Module Responsibility Map
│   ├── ### The Message Path (YouTube Chat → Frame Loop)
│   ├── ### The Frame Path (Frame Loop → Renderer → FFmpeg → YouTube)
│   ├── ### Startup Orchestration (the run 10-Stage Funnel)
│   └── ### Opt-In Diagnostics Overlay
├── ## Reusable Components vs Extension Points
│   ├── ### Reuse Unchanged
│   ├── ### Extension Points
│   └── ### The Independence Rule
├── ## Recommended Future Game Flow (NOT Implemented)
│   ├── ### The Flow
│   └── ### The Five Separation Lines
├── ## Timing and Scale Constraints
│   ├── ### The Latency Warning
│   ├── ### Viewers Are Not Message Throughput
│   ├── ### Bounded Queues and Overload
│   └── ### Resolution Does Not Change Timing Semantics
├── ## Guidance for the First Game Prototype (10 items)
├── ## Decision Checklist
├── ## What Not to Change
└── ## See Also
```

### 2.4 Per-section content outlines with the key factual sentences

**(A) `## Purpose and Contract`** (~6 lines)
- Audience: the future agent implementing the first game prototype; what the guide is (actual current-state map + extension instructions) and is NOT (a design, a latency analysis, an operator procedure).
- Three link sentences: architecture source of truth = `.agent/project-info/brief.md`; operator/mode docs = README + docs/*; measurement facts = `docs/latency-test-map.md`.
- Verbatim claim: "This document contains NO game implementation and does not change any runtime behavior — it is documentation only."

**(B) `## Executable Modes and Their Responsibilities`**

H3 `### auth (Interactive, Once)` — key sentences:
- "`auth` performs the Google OAuth 2.0 installed-application authorization flow: it loads the client file at `oauth.client_config_path`, binds a loopback-only callback listener, prints the authorization URL brochure (no browser launch, no interactive input), exchanges the code with PKCE S256, validates offline that the granted scope is `https://www.googleapis.com/auth/youtube.force-ssl` and that a refresh token was issued, and persists credentials atomically to `oauth.credentials_path`." (cites `src/app/auth/flow.rs`, `listener.rs`, `src/youtube/authorization.rs`, `src/youtube/file_credential_store.rs`)
- "Every failure is one redacted stderr `error:` line and exit 1; a verified success prints one stdout line and exits 0." (cites `src/app/auth/mod.rs`)
- "A game deployment uses the SAME grant: no re-auth is needed for chat (`youtube.force-ssl` is documented-sufficient for both streamList and the chat-id lookup — see the Live Chat section of `docs/production-run.md`)." (cites `docs/production-run.md` Live Chat Prerequisites)

H3 `### run (Production Broadcast Lifecycle)` — key sentences:
- "One `run` invocation: installs the Ctrl+C stop pair, runs the credential service non-interactively (load → verify-first → at most one refresh → atomic persist → re-verify), gates the required scope BEFORE any resource creation, creates ONE new broadcast + ONE new live-stream pair, binds them, readiness-waits (≤180 s at 5 s), acquires the RTMPS destination (kept secret, never logged), starts the configured FFmpeg toward RTMPS, and confirms `streamStatus == active` on a thread parallel with the frame loop (≤120 s budget) before the chat-backed frame loop runs." (cites `src/app/run/mod.rs:107–161`, `broadcast.rs`, `gate.rs`, `src/youtube/readiness_wait/*`)
- "The chat worker spawns ONLY after the confirmation gate observes `streamStatus == active`; its FIRST act resolves the broadcast's live chat id (bounded 6 × 5 s budget), then it establishes the `streamList` server-streamed connection and never re-connected on a schedule — it re-establishes only after a drop, carrying the accumulated `nextPageToken`." (cites `src/app/run/gate.rs`, `chat_worker.rs`, `chat_stream_delivery.rs`, `src/youtube/live_chat_resolution.rs`)
- "Resource fate: the application NEVER deletes or transitions broadcast/stream resources — `enableAutoStop=true` auto-completes the broadcast when ingestion ends; failed startups leave `ready` resources that the operator deletes in YouTube Studio." (cites `src/app/run/mod.rs:1–7` module docs; `docs/production-run.md` Broadcast/Stream Fate)
- Mode exit contract: clean/pre-streaming stop 0; any fatal one redacted `error:` line + 1. (cites `src/app/run/mod.rs:39–63`)

H3 `### demo (Local-Only Offline Recording)` — key sentences:
- "`demo` runs a fixed, deterministic message timeline (28 messages, 30-frame lead-in, at most one message per 15 frames, 60-frame tail → 496 frames ≈ 16.5 s at the example 30 FPS) through the SAME chat → renderer → frame → FFmpeg pipeline as `run`, records an MP4 under `videos/`, and NEVER contacts YouTube/OAuth/the chat transport." (cites `src/app/demo/schedule.rs`, `mod.rs`, `docs/demo.md`)
- "It is the template for offline game-behavior development: swap the message source, keep the pipeline."

**(C) `## Current Runtime Data Flow (Actual Source Modules)`**

Intro sentence: "The chain below is the ACTUAL implementation (every arrow names real files at implementation time); the conceptual target stays in brief §2.2/§2.3 and `.agent/project-info/architecture.md`."

H3 `### Module Responsibility Map` — the table (the guide's core reference):

| Module | Responsibility | Depends on (boundary) |
| --- | --- | --- |
| `src/app/` | Startup funnel, CLI modes, the frame pipeline, the `auth`/`demo`/`run` orchestration, the chat worker, the opt-in diagnostics collector | all other modules (composition only) |
| `src/app/run/` + `src/app/pipeline/` | run-mode orchestration + the paced frame loop (ONE message per frame) | config, youtube (through seams), chat, renderer, streaming |
| `src/config/` | Typed JSON schema (`deny_unknown_fields`), 27 semantic validation rules, loading | none (leaf; typed errors, no panics) |
| `src/youtube/` | OAuth credential layer, broadcast/stream lifecycle (5-operation `LiveApiTransport`), RTMPS destination wrapper, Live Chat retrieval (the O6 lookup + the `streamList` seam family) | nothing app-side; MUST never depend on `chat`, `renderer`, `streaming`, or `demo` (`src/youtube/mod.rs`) |
| `src/chat/` | `ChatMessage { author, text }` (Display exactly `author: text`; optional ids/timestamps stay off-screen) + bounded visible FIFO `ChatStore` + the append-only chat log | no YouTube API types, no network, no streaming, no FFmpeg (`src/chat/mod.rs`) |
| `src/renderer/` | `RenderInput` + `Renderer` trait + `TextRenderer` (ab_glyph) producing checked RGB24 `Frame`s | a pure local component: no YouTube/chat/network/FFmpeg (`src/renderer/mod.rs`) |
| `src/streaming/` | FFmpeg spawn/supervision, the locked argument vectors, frame-sink delivery as opaque bytes | only std + typed config; never chat/renderer/YouTube types (`src/streaming/mod.rs`) |
| `src/app/diagnostics/` | Opt-in read-only observability collector (bounded counters/histograms/in-flight map; JSONL writer with bounded rotation) | config + std only; never carries message text/authors/tokens/RTMPS URLs |

H3 `### The Message Path (YouTube Chat → Frame Loop)` — flow diagram text (verbatim in the guide):

```text
YouTube liveChatMessages.streamList (gRPC server streaming, Bearer-only)
  → src/youtube/stream_{proto,transport,client,concrete}.rs + live_chat_stream_mapping.rs
        (published-proto wire types; readiness reserve before the streaming call; TEXT-message-only mapping)
  → src/app/run/chat_worker.rs  (+ chat_stream_worker.rs / chat_stream_delivery.rs / chat_worker_helpers.rs)
        (ONE dedicated worker thread; confined current-thread Tokio runtime; catch_unwind containment;
         history-first baseline discard; try_send-only delivery; bounded reconnect with the retained token)
  → bounded channel: std::sync::mpsc::sync_channel(64)  (src/app/run/chat_channel.rs)  + terminal cell
  → src/app/run/source.rs :: ChatSource  (non-blocking try_recv; Ok(None) while open and empty)
  → src/app/pipeline/runner.rs  (at most ONE message per frame iteration into src/chat/store.rs :: ChatStore)
```

Body sentences: the one-per-frame consumer rule ⇒ nominal capacity = `fps` messages/second (30 at the example config); the 64-slot channel is the ONLY queue between worker and frame loop; up to 64 queued-but-undelivered messages can be lost at shutdown (they are never logged — log/video never diverge; cites `docs/production-run.md` Honest boundedness facts). The worker-side diagnostics hooks are listed with their metric names (`received`, `enqueued`, `full_retry`, `worker_retained`, `reconnect_count` — cites `src/app/run/chat_stream_delivery.rs` + `docs/latency-test-map.md`).

H3 `### The Frame Path (Frame Loop → Renderer → FFmpeg → YouTube)` — flow diagram text:

```text
src/app/pipeline/runner.rs (FPS-paced loop, StopFlag-polled)
  → dequeue (ChatSource) → node_message_dequeued diagnostics hook
  → src/chat/store.rs :: ChatStore (bounded visible FIFO; append-only chat log first)
  → render input = ChatStore.visible_text → src/renderer (TextRenderer → checked RGB24 Frame)
  → frame write to FrameSink (opaque &[u8]; src/app/pipeline/runner.rs never names FfmpegProcess)
  → src/streaming/ffmpeg.rs :: FfmpegProcess.stdin  (locked streaming vector: libx264 veryfast,
     derived CBR video block, optional libmp3lame 128k silent audio; process-group console isolation)
  → RTMPS (the redacting RtmpsDestination wrapper; the URL exists ONLY at the single reveal in src/app/run)
  → YouTube Live
```

H3 `### Startup Orchestration (the run 10-Stage Funnel)` — ordered one-sentence-per-stage list condensed from `src/app/run/mod.rs:107–161` + `gate.rs` + `docs/production-run.md` "What Happens at Startup", ending with the shutdown order sentence: "On Ctrl+C the shutdown is ordered: frame-loop stop flag → chat worker stop → YouTube wait cancellation → FFmpeg stdin EOF → drain + wait → the bounded chat-worker join at the gate finish." (cites `stop_pair.rs`, `gate.rs`; `docs/production-run.md` Shutdown Behavior).

H3 `### Opt-In Diagnostics Overlay` — 4 sentences: `DiagnosticsCollector::from_config` returns `None` unless a capability is enabled (a disabled config allocates/costs nothing beyond one `Option` check per site, per docs); rows are per-message but never text/authors/secrets (provider message id is the single correlation value); two clock domains (wall-clock `publishedAt` vs local monotonic `*_ns`) with the embedded `CLOCK_DOMAIN_NOTE`; rotation bounds total disk ≤ `max_file_bytes × max_generations`. LINK `docs/production-run.md#pipeline-observability-opt-in-diagnostics` + `docs/configuration.md#diagnostics` — no field table duplication.

**(D) `## Reusable Components vs Extension Points`**

H3 `### Reuse Unchanged` — itemized: the frame pipeline + pacing (`src/app/pipeline/runner.rs`, `clock.rs`, `pacing.rs`), `ChatSource`, the renderer family, the stream family (FFmpeg spawn, `FrameSink`, argv builders, redaction), the config layer, the diagnostics collector, the credential service. Sentence: "Do not rewrite these to 'make room' for a game — they are the tested infrastructure; a game bug must not be able to take down the stream."

H3 `### Extension Points` — the four named seams with file citations:
1. `src/app/pipeline/source.rs :: MessageSource` — already widened to `Result<Option<ChatMessage>, ChatSourceError>`; a game command adapter is a NEW `MessageSource` implementation, not an edit of the chat worker.
2. `src/renderer.rs :: Renderer` trait + `RenderInput` — a game renderer implements the trait; games map their render state to `RenderInput` at the composition site.
3. The demo timeline (`src/app/demo/schedule.rs` + `source.rs`) — the pattern for a deterministic game-side test source.
4. Config (`src/config/model.rs` + `validation.rs` precedent) — new game settings join the validated config with named validation rules and an example-template update.

H3 `### The Independence Rule` — the verbatim TODO sentence reworded honestly: "Game logic must not depend on YouTube API types, OAuth, broadcast IDs, RTMPS URLs, or FFmpeg command details. A game module may reach only `crate::chat::ChatMessage` (or its own command types), `crate::config`, and the renderer/pipeline seams. All YouTube specifics (`src/youtube/*`), all credential material, and all FFmpeg argument construction (`src/streaming/args.rs`, `stream_args.rs`) stay out of game modules — the app layer is the only place those meet."

**(E) `## Recommended Future Game Flow (NOT Implemented)`**

H3 `### The Flow` — the chain (verbatim TODO p. 122, restyled as the guide's text):

```text
YouTube Chat Adapter  ->  ChatMessage  ->  Command Parsing/Validation
  ->  Game Input/Events  ->  Game State Update  ->  Render State
  ->  Renderer  ->  RGB24 Frames  ->  FFmpeg  ->  YouTube Live
```

plus the note: "This file implements none of it; a separated game TODO owns it. The adapter starts at the EXISTING `MessageSource` seam — the chat worker, channel, and frame loop above it are intentionally unchanged."

H3 `### The Five Separation Lines` — each as one bullet with the technical reason (all derive from the verified facts):
1. Transport messages vs validated game commands — `ChatMessage` is `author` + `text` + metadata off-screen (`src/chat/model.rs`); a chat message is NOT a command until a game-side parser has validated it.
2. Command parsing/authorization/rate-limiting vs game rules — input governance lives in its own module, so rule changes never touch transport parsing.
3. Authoritative game state vs render-only state — only a projection of state becomes `RenderInput` (brief §9.6 precedent: the store owns visibility, the renderer draws what it is given).
4. Deterministic game update/tick logic vs frame rendering — tick functions are pure and testable with fake clocks (house rule: zero sleeps in tests); rendering happens at the FPS cadence.
5. Game simulation cadence vs output FPS — the frame loop is paced at `video.fps` (`src/app/pipeline/pacing.rs`); simulation ticks are independent (e.g. sub-frame or multi-frame rates), deliberately not coupled to the encode cadence.

**(F) `## Timing and Scale Constraints`**

H3 `### The Latency Warning` — key sentences:
- "YouTube Live Chat IS NOT a real-time game networking transport: a message reaches the app only after upstream delivery, and the viewer sees the frame only after encoding, ingest, and player buffering." (cites `docs/latency-test-map.md` boundary table)
- "The approximately 35 seconds from send to first-visible is the CURRENT OPERATOR-OBSERVED end-to-end baseline (TODO-dated 2026-10-31, motivating the observability work) — a measurement to investigate stage by stage, never a platform guarantee or constant, and never attributable to any single stage without measurements."
- DEFAULT recommendation: "Design latency-tolerant mechanics FIRST — turn-based moves, queued actions, voting, prediction-style prompts — and justify any real-time mechanic only after operator measurements (the procedure + template in `docs/latency-test-map.md`) show the model is wrong."

H3 `### Viewers Are Not Message Throughput` — "Load is driven by the MESSAGE rate and burst size, and by processing cost — not by viewer count. The consumer ceiling is one message per frame (nominally `fps`/second at the configured FPS), and upstream bursts larger than the 64-slot channel retention are retained by the worker, not dropped."

H3 `### Bounded Queues and Overload` — "The bounded-memory/backpressure behavior is a project invariant: no unbounded queues, no silent message drops, no weakened shutdown guarantees (TODO constraint, p. 19). Overload behavior must be DEFINED explicitly by the game design before implementation (retention-and-retry is the existing precedent)."

H3 `### Resolution Does Not Change Timing Semantics` — "Output resolution (1080p/4K) affects frame memory (`width × height × 3` bytes per frame), rendering cost, and encoding bandwidth — NOT the timing semantics of chat delivery; the chat → frame timing math is resolution-independent."

**(G) `## Guidance for the First Game Prototype (10 items)`** — the TODO pp. 129–139 list REWORDED into the guide's voice (each item keeps the TODO's intent; the guide adds the file-pointer parentheses):

1. First inspect `AGENTS.md`, the `.agent/project-info/` files (`brief.md` is the source of truth; `context.md` is the status log), the workflows (`.agent/WORKFLOWS.md`), the rules (`.agent/RULES.md`), and `.agent/project-structure.md` before adding any file.
2. Before changing any module, inspect that module's contracts AND its test rosters — the pinned tests are the executable form of the contract (for example `src/app/run/*_tests.rs`, `src/streaming/*_tests.rs`, `src/app/pipeline/*_tests.rs`).
3. Preserve the `auth`, `demo`, and `run` contracts: CLI usage/exit-code table, the single-`error:`-line rule, the demo timeline and exit contract, the ordered shutdown.
4. Add game-specific modules behind application-level interfaces (as a new `src/app/` submodule of the implementer's choosing, planned in a future TODO) rather than embedding game rules in the `src/youtube/` or `src/streaming/` modules.
5. Keep game state independent of the renderer implementation: game state lives in its own module and only a projection is handed to `RenderInput`.
6. Make update logic deterministic and unit-testable: pure tick functions, injected clocks, zero sleeps in tests (the existing house rule).
7. Define explicit policies for command validation, permissions, spam/rate limiting, queue overflow, and simultaneous inputs BEFORE implementing the prototype.
8. Add a deterministic local demo/test source so game behavior can be developed without a live broadcast (the `src/app/demo/` schedule pattern).
9. Avoid introducing a new async runtime or changing the chat worker's runtime ownership without an architectural reason recorded in the plan: the only Tokio in the tree is the confined current-thread runtime inside the ONE chat worker thread (`src/app/run/chat_worker.rs`).
10. When the architecture changes, update `.agent/project-structure.md`, `config/config.example.json` + `docs/configuration.md`, this guide's facts, and the project-info files per the update policy in `.agent/project-info/instructions.md`.

**(H) `## Decision Checklist`** — the pre-implementation questions (bullets, each with a one-line default answer):
- Where does the adapter attach? → default: a new `MessageSource` implementation fed by game commands, leaving `chat_stream_delivery.rs` byte-stable.
- Which mechanics first? → default: latency-tolerant (turn/vote/queue) until measurements justify otherwise.
- What is measured first? → default: enable diagnostics offline + the `latency_load` harness; live figures ONLY via the operator procedure.
- What state is authoritative vs rendered? → default: game state authoritative; render state a derived projection.
- What is the input policy (grammar/permissions/rate limit/overload/simultaneous inputs)? → default: none is invented by this repository — the game TODO must own it.
- Cadence: tick rate vs FPS? → default: independent; do not couple to `video.fps`.
- Offline dev path? → default: demo-pattern source + offline tests; no OAuth, no broadcast.
- Which contracts are frozen? → see `## What Not to Change` below.

**(I) `## What Not to Change`** — the frozen list (one sentence each, all cited):
1. The locked FFmpeg argument vectors and their CBR block (`src/streaming/args.rs`, `stream_args.rs` — the byte-pinned 33/42-token vectors).
2. The bounded channel and its delivery rules (`CHAT_CHANNEL_CAPACITY = 64`, `try_send`-only, retain-retry, non-blocking `try_recv`) — no unbounded queue, no silent drop.
3. The shutdown order and the single Ctrl+C handler pair (`src/app/run/stop_pair.rs`, `src/app/pipeline/interrupt.rs`).
4. The exit-code contract: usage 2 / startup+mode-fatal 1 / success+clean-stop 0, and the ONE `error:` line rule everywhere.
5. The resource-fate policy: never delete/transition YouTube resources; `enableAutoStop` owns completion.
6. The secret-redaction rules: OAuth/token material inside redacting wrappers (`src/youtube/token.rs`, `rtmps_ingestion.rs`, `rtmps_destination.rs`), capture-time diagnostics redaction (`src/streaming/ffmpeg.rs`, `pipe_drain.rs`), worker D-13 (message text/authors never logged), diagnostics never carrying text/authors/secrets.
7. The credential record schema (`src/youtube/persisted_credentials.rs`, frozen 4-key) and the required scope.
8. The `demo` local-only contract: no YouTube contact of any kind.
9. The one-per-frame consumer and the continuous-FPS frame loop (a change here is an architecture decision, not a refactor).
10. The confined-runtime ruling: the chat worker's runtime stays confined to the worker thread.

**(J) `## See Also`** — bullets: `../README.md`; `docs/production-run.md` (incl. the Pipeline Observability anchor); `docs/latency-test-map.md`; `docs/configuration.md`; `docs/demo.md`; `docs/authentication.md`; `.agent/project-info/architecture.md`; `.agent/project-structure.md`.

### 2.5 Size + TOC decision

- Target length: **≤ 230 lines** (projected ~200–215; comparable-but-larger than Task-2's 142-line `latency-test-map.md` because Task 3 carries 6 content areas). No repo-wide 100-line docs rule applies, but TOC is REQUIRED (>100 lines) and every table stays ≤ 10 rows to keep the page scannable.
- CRITICAL: no field-by-field duplication of `docs/configuration.md`, no operator-procedure duplication of `docs/production-run.md`/`latency-test-map.md` (link, never copy — TODO p. 148).

### 2.6 Cross-link list (the ONLY other doc edits)

| File | Edit | Content (exact placement) |
| --- | --- | --- |
| `README.md` | +1 bullet | In `## AI Agents`, after the `architecture.md` bullet: `- [Game foundation handoff](docs/game-foundation.md) — how to reuse this repository (auth/run/demo, chat→frame→FFmpeg pipeline, seams) as the base for a future chat-controlled game.` |
| `docs/production-run.md` | +1 See-Also row | `- [docs/game-foundation.md](game-foundation.md) — how this mode's pipeline becomes the base for a future chat-controlled game (architecture map, extension points, frozen contracts).` (See Also section; NO other edit) |
| `docs/latency-test-map.md` | +1 See-Also row | `- [docs/game-foundation.md](game-foundation.md) — the game-foundation handoff consuming this page's timing facts.` |
| `docs/configuration.md` | +1 See-Also row | `- [docs/game-foundation.md](game-foundation.md) — the game-foundation handoff guide (when and why to extend the validated configuration).` |

---

## 3. Project-Info Update Plan

| File | Change (authored by docs-specialist in 4.2, committed at 4.6) |
| --- | --- |
| `architecture.md` | (1) In the Module Map table, add one row: `src/app/diagnostics/` — "Opt-in read-only pipeline diagnostics collector (disabled by default)."; (2) after the Module Map intro sentence "For current repo state, see `context.md`.", append: "The implemented layout and the actual chat→frame→FFmpeg data-flow map (v0.14.0 tree) live in `docs/game-foundation.md`."; (3) in Boundary Rules, add: "Game logic must not depend on YouTube API types, OAuth, broadcast IDs, RTMPS URLs, or FFmpeg command details — the extension-point map and frozen contracts are in `docs/game-foundation.md`." NOTHING else changes. |
| `tech.md` | Add ONE bullet after the Task-2 harness bullet (end of Dependencies section): "The game-foundation handoff guide (`docs/game-foundation.md`, v0.14.0 Task 3) records the reusable-vs-extension-point map, the recommended future flow (chat adapter → commands → game state → render state → renderer → FFmpeg), and the frozen contracts; it is documentation only — no code, dependency, or behavior change." |
| `context.md` | Add a dated block `## Game-Foundation Handoff Documentation (2026-11-01)` after the current top block: states Task 3 is docs-only (the 8 edited `.md` paths), that the gate OBSERVED 550/0 evidence at HEAD `27586ba` remains valid (no source change), that `.agent/project-structure.md` and `AGENTS.md` are untouched, and lists OPERATOR/integrator next steps: a future game TODO consumes the guide. The block is WRITTEN in 4.2 and committed at 4.6 (Tasks 1–2 pattern). |
| `AGENTS.md` | NO change (no new project-info file is added). |
| `brief.md` / `product.md` | NO change (source of truth and product docs are out of Task-3 scope). |
| `.agent/project-structure.md` | NO change (the source tree is unchanged — TODO K). |

---

## 4. Step Breakdown (docs-specialist, 4.2) + Git Boundaries

All steps are markdown-only; every step uses the write/edit tools (never shell file ops); all content is inline-provided by this plan (the junior docs implementer handles wording polish only within the given tables/sentences).

- D1: Create `docs/game-foundation.md` exactly per §2 (title, TOC, 10 H2 / 15 H3, the two code-block flow diagrams, the 8-row module table, the 10-item guidance, the 8-bullet decision checklist, the 10-item frozen list, See Also; length ≤ 230 lines; real newlines).
- D2: Edit `README.md` — the single AI-Agents bullet (§2.6).
- D3: Edit `docs/production-run.md` — the single See-Also row (importance: do NOT touch any other line; the file is insertion-only).
- D4: Edit `docs/latency-test-map.md` — the single See-Also row.
- D5: Edit `docs/configuration.md` — the single See-Also row.
- D6: Edit `.agent/project-info/architecture.md` — the three §3 deltas.
- D7: Edit `.agent/project-info/tech.md` — the single bullet.
- D8: Edit `.agent/project-info/context.md` — the dated Task-3 block (draft wording, uncommitted).
- D9 (self-check, no VM): verify TOC anchors match section slugs; links resolve to real files; no literal `\n`; length count ≤ 230; `git status --porcelain` shows EXACTLY the 8 expected paths (1 new + 7 modified) and nothing else; secret-pattern sweep (`ya29`, `AIza`, `key=`, `rtmps://`) counts identical to HEAD (the guide mentions no secrets; the RTMPS mentions are the frozen-list boundary sentences naming the wrapper modules, not URLs).

Git boundaries:
- 4.2 leaves ALL edits UNCOMMITTED in the working tree (the caller's binding pattern; attribution precedents: hash unknowable from inside the record, captured by 4.3/4.4 review passes).
- 4.3: code-reviewer performs the factual/rationality review of the guide against §5's ledger; no markdown write rights for the reviewer — findings return to the caller.
- 4.4: docs-specialist applies any required factual/wording fixes ONLY within the touched files (repo precedent: payload corrections without new scope).
- 4.5b: architector adjudicates adherence against this plan.
- 4.6: the junior implementer makes ONE commit, e.g. `docs: game-foundation handoff guide, cross-links and project-info refresh (Task 3)` — staging ONLY the 8 markdown paths; verification: `git status` clean of stray paths; NO version bump, NO gate re-run (§6); then records the commit hash in context.md notes if required by the caller.

---

## 5. Factual-Accuracy Ledger (every code claim → citation; the 4.3 reviewer verifies row by row)

| # | Claim in the guide | Citation |
| --- | --- | --- |
| L1 | Process entry, module roots | `src/main.rs:5–15` |
| L2 | CLI modes/flags/default path | `src/app/cli.rs`; `.agent/project-structure.md` line 6 |
| L3 | Startup funnel, 3 info events, single `error:`, exit codes 2/1 | `src/app/mod.rs:40–133` |
| L4 | auth flow grades (client file, loopback listener, brochure, PKCE, scope check, atomic persist, exit 1/0, redaction) | `src/app/auth/{mod,flow,listener,console}.rs`; `.agent/project-structure.md` lines 31–36 |
| L5 | run_runtime order (credentials → scope gate → transports → prepare → destination reveal → diagnostics → chat stage → gate → runner) | `src/app/run/mod.rs:107–161` |
| L6 | The ONE destination reveal | `src/app/run/mod.rs:129` + `src/youtube/rtmps_ingestion.rs` (`into_url()` ONE reveal) |
| L7 | Resource fate (never delete/transition; enableAutoStop; `ready` leftovers) | `src/app/run/mod.rs:1–7`; `docs/production-run.md` Broadcast/Stream Fate |
| L8 | Gate budgets (180 s/5 s readiness; 120 s/5 s confirmation) + chat worker spawned ONLY at Confirmed | `src/youtube/readiness_wait/{mod,confirmation_wait}.rs`; `src/app/run/gate.rs` |
| L9 | Watch-link info event (`https://www.youtube.com/watch?v=<broadcast_id>`, non-secret) | `src/app/run/broadcast.rs`; `.agent/project-structure.md` line 16 |
| L10 | Worker constants (64; 1 s slices; 60 s backoff; 10-transient) | `src/app/run/chat_worker.rs:46–49` |
| L11 | Confined current-thread Tokio runtime on the ONE worker thread; block_on funnel | `src/app/run/chat_worker.rs:1–17`; `.agent/project-info/tech.md` Stack bullet |
| L12 | Panic containment (`catch_unwind` → typed terminal with the fixed detail; backtrace stays on stderr) | `src/app/run/chat_worker.rs:32–33`; `docs/production-run.md` contained-panic paragraph; `.agent/project-info/tech.md` Lifecycle bullet |
| L13 | streamList wire family (published proto, hand-authored prost; Bearer-only; readiness reserve `grpc.ready()` with the fixed reason; TEXT-only mapping; displayName→channelId fallback; RFC3339→epoch) | `src/youtube/stream_{proto,transport,client,concrete}.rs`, `stream_concrete_tests.rs`, `live_chat_stream_mapping.rs`, `live_chat_rfc3339.rs` |
| L14 | O6 chat-id resolution 6 × 5 s budget; 512-id dedup window | `src/youtube/live_chat_resolution.rs`; `live_chat_dedup.rs` |
| L15 | History-first baseline discard; resumable reconnect with retained token; never re-called on schedule | `src/app/run/chat_stream_delivery.rs` (`process_batch`); `docs/production-run.md` Live Chat Expected Behavior |
| L16 | Bounded `sync_channel(64)` + terminal cell; `ChatSource` try_recv/Ok(None) | `src/app/run/chat_channel.rs`; `src/app/run/source.rs` |
| L17 | One message per frame; FPS-paced loop; sink is `FrameSink` opaque bytes; float-locked streaming vector w/ CBR | `src/app/pipeline/runner.rs:54–66`; `src/streaming/stream_args.rs` |
| L18 | Chat model (Display exactly `author: text`; metadata off-screen; log-append-first store; derived visible capacity) | `src/chat/{mod,model,store}.rs` |
| L19 | Renderer boundary (pure local; no YouTube/chat/network/FFmpeg) + trait/settings/frame/text facts | `src/renderer/mod.rs:1–12`, `renderer.rs`, `settings.rs`, `frame.rs`, `text.rs` |
| L20 | Streaming boundary (std+config only; opaque bytes; capture-time redaction; spawn isolation) | `src/streaming/mod.rs:1–24`, `ffmpeg.rs`, `spawn.rs`, `pipe_drain.rs`, `rtmps_destination.rs` |
| L21 | Config (typed serde, deny_unknown_fields, 27 rules, diagnostics section) | `src/config/{mod,model,loader,validation,diagnostics_config}.rs` |
| L22 | Credential service chain + frozen 4-key schema + atomic 0600 store + REQUIRED_SCOPE | `src/youtube/{credential_service,persisted_credentials,file_credential_store,authorization}.rs` |
| L23 | Diagnostics construction (None unless capability); bounded state (counters, 6 histograms, 128-entry in-flight cap); JSONL writer rotation ≤ max_file_bytes × max_generations; CLOCK_DOMAIN_NOTE | `src/app/diagnostics/{mod,collector,state,histogram,events,writer}.rs` |
| L24 | Offline latency harness (test-only; real seams; RateProfile integer credit; FPS-derived pacing; capacity from `CHAT_CHANNEL_CAPACITY`) | `src/app/run/latency_load_{fixtures,source}.rs`; `docs/latency-test-map.md` Capacity Math |
| L25 | Demo timeline facts + exit contract + never contacts YouTube | `src/app/demo/{mod,schedule,source}.rs`; `docs/demo.md` |
| L26 | Shutdown order (stop flag → wait cancellation → stdin EOF → drain/wait → bounded worker join at gate finish) | `src/app/run/{stop_pair,gate}.rs`; `docs/production-run.md` Shutdown Behavior |
| L27 | The ~35 s observation attribution (operator-observed baseline, measurement to investigate) | TODO p. 5; `docs/production-run.md` line 375; `docs/latency-test-map.md` lines 23–25 |
| L28 | Viewers ≠ throughput; one-per-frame ceiling; worker retention on full channel | TODO pp. 144–146; `src/app/pipeline/runner.rs`; `src/app/run/chat_stream_delivery.rs` retain arm |
| L29 | Resolution affects memory/cost/encoding, not chat timing semantics | `width × height × 3` (`src/renderer/frame.rs`); TODO p. 146 |
| L30 | No new dependency for diagnostics/harness; only Tokio confined to the chat worker | `Cargo.toml` (v0.14.0, structure map line 145); `docs/latency-test-map.md` Offline Verification Reminder |

---

## 6. Verification — No Gate Needed (explicit reasoning)

- **ZERO code/test/config/dependency/version changes are expected** — this task touches only the 8 markdown paths (D1–D8). No gate re-run is required before 4.6.
- Reasoning recorded explicitly: the OBSERVED gate result **550 passed / 0 failed** (branch state at the implementation-time HEAD) remains VALID because (a) cargo gates observe only Rust sources, `Cargo.toml`/`Cargo.lock`, and test fixtures; (b) `.md` files are NOT inputs to `cargo fmt/check/test/clippy`; (c) the 4.6 commit adds only markdown, so no gate-relevant input changes between the gated HEAD and the committed tree. A re-run would re-observe the same 550/0 and is NOT required by the TODO (Task-3 acceptance criteria are all documentation/consistency criteria; Global AC "existing checks pass" is satisfied by the unchanged tree).
- The ONLY required verifications are the D9 no-VM checks (status/paths/anchors/secret-pattern counts) plus the 4.3 factual review against §5. If any step ever drifts outside the 8 paths (forbidden), STOP and escalate to the caller.

## 7. Line/Scope Guards (BINDING)

1. NO new `src/` file; NO edit to any file under `src/`, `config/`, `scripts/`, `Dockerfile`, `docker-compose.yml`.
2. NO `Cargo.toml`/`Cargo.lock` change; NO dependency addition; NO version bump (stays 0.14.0).
3. `.agent/project-structure.md` UNTOUCHED (the tree does not change); `AGENTS.md` UNTOUCHED (no new project-info file); `brief.md`/`product.md` UNTOUCHED.
4. Guide length cap ≤ 230 lines with TOC; every other touched doc grows by EXACTLY one row/bullet/paragraph as specified.
5. No operator procedure duplication; no diagnostics field-table duplication (link).
6. NO game mechanics, NO runtime behavior change, NO test addition (TODO Task-3 final AC: "No game mechanics or game-specific runtime behavior are implemented as part of this task").
7. No secret material in any edit (no real tokens/keys/RTMPS URLs; frozen-list words name module files, never URLs).
8. Boolean condition style/`single-section`/`max-depth` rules are irrelevant here (no code), but ALL markdown files use REAL newlines.

## 8. Front-End Determination

NOT front-end — no UI change exists; sub-steps 4.1a/4.5a are NOT executed for Task 3.

---

## 9. Traceability Table (TODO Task-3 requirement → plan section)

| TODO text (line pp.) | Binding requirement | Plan coverage |
| --- | --- | --- |
| pp. 109–112 | executable modes + responsibilities (A) | §2.4(B) modes section |
| p. 113–119 | runtime data flow with actual source-module locations (B) | §2.4(C) data-flow section + §5 ledger rows L1–L26 |
| p. 120–121 | reusable vs extension points; independence rule (C) | §2.4(D); §3 architecture.md bullet 3 |
| pp. 121–128 | future flow + five separation lines (D) | §2.4(E) |
| pp. 129–139 | the 10-item agent guidance, verbatim coverage (E) | §2.4(G) — all 10 items present, voice-consistent |
| pp. 140–146 | timing/scale constraints incl. honest ~35 s attribution (F) | §2.4(F); ledger row L27–L29 |
| p. 147 | decision checklist + what-not-to-change (G) | §2.4(H), §2.4(I) |
| p. 148 | cross-links from README + relevant docs; no duplication (H, final J clause) | §2.6 |
| p. 149 | project-info updates; AGENTS.md accuracy only if files added (I) | §3; AGENTS.md = NO change (no new file) |
| p. 150 | focused, sized files, TOC for >100 lines (J) | §2.5 |
| pp. 152–159 | Task-3 acceptance criteria | §9 rows map 1:1: future-agent identification (§2.4 D+G), actual code paths (§2.4 C + §5), latency warning + latency-tolerant default (§2.4 F), offline path (§2.4 B demo H3), links + project-info consistency (§2.6 + §3), NO game mechanics (§7.6) |
| pp. 161–165 (Global AC) | pipeline/UML/shutdown/redaction/exit contracts untouched | §6 + §7 (unchanged tree; gate validity reasoning) |
| p. 166 (Global AC) | ~35 s = measurement, not bottleneck/guarantee | §2.4(F) Latency-Warning wording; §7.7 |
| pp. 167–168 (Global AC) | no game implementation; docs-only TODO | §1.4(A last sentence) + §7.6 |

---

## Approval

Presented to the caller for approval; upon approval the docs-specialist executes §4 D1–D9. The implementer never writes markdown; 4.6 commits only.

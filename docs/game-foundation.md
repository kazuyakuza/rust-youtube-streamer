# Game Foundation Handoff

This guide describes this repository as it exists at v0.14.0 (branch state at implementation time) — the actual code paths and contracts, not the brief's intended architecture — and tells a future implementation agent (human or AI) how to build a chat-controlled game on top of it without breaking the streaming infrastructure.

## Table of Contents

- [Purpose and Contract](#purpose-and-contract)
- [Executable Modes and Their Responsibilities](#executable-modes-and-their-responsibilities)
- [Current Runtime Data Flow (Actual Source Modules)](#current-runtime-data-flow-actual-source-modules)
- [Reusable Components vs Extension Points](#reusable-components-vs-extension-points)
- [Recommended Future Game Flow (NOT Implemented)](#recommended-future-game-flow-not-implemented)
- [Timing and Scale Constraints](#timing-and-scale-constraints)
- [Guidance for the First Game Prototype (10 items)](#guidance-for-the-first-game-prototype-10-items)
- [Decision Checklist](#decision-checklist)
- [What Not to Change](#what-not-to-change)
- [See Also](#see-also)

## Purpose and Contract

Audience: the future agent that will implement the first chat-controlled game prototype. This document is an actual current-state map plus extension instructions. It is NOT a game design, NOT a latency analysis, and NOT an operator procedure — those live in the documents it links. The architecture source of truth is [`.agent/project-info/brief.md`](../.agent/project-info/brief.md); operator/mode documentation lives in [`../README.md`](../README.md) and this `docs/` folder; timing measurement facts are canonical in [`latency-test-map.md`](latency-test-map.md). This document contains NO game implementation and does not change any runtime behavior — it is documentation only.

## Executable Modes and Their Responsibilities

### auth (Interactive, Once)

`auth` performs the Google OAuth 2.0 installed-application authorization flow: it loads the client file at `oauth.client_config_path`, binds a loopback-only callback listener, prints the authorization-URL brochure (no browser launch, no interactive input), exchanges the code with PKCE S256, validates offline that the granted scope is `https://www.googleapis.com/auth/youtube.force-ssl` and that a refresh token was issued, and persists credentials atomically to `oauth.credentials_path` (`src/app/auth/flow.rs`, `src/app/auth/listener.rs`, `src/youtube/authorization.rs`, `src/youtube/file_credential_store.rs`). Every failure is one redacted stderr `error:` line and exit 1; a verified success prints one stdout line and exits 0 (`src/app/auth/mod.rs`). A game deployment uses the SAME grant: no re-auth is needed for chat — `youtube.force-ssl` is documented-sufficient for both `streamList` and the chat-id lookup (the Live Chat prerequisites in [`production-run.md`](production-run.md)).

### run (Production Broadcast Lifecycle)

One `run` invocation installs the Ctrl+C stop pair, runs the credential service non-interactively (load → verify-first → at most one refresh → atomic persist → re-verify), gates the required scope BEFORE any resource creation, creates ONE new broadcast + ONE new live-stream pair, binds them, readiness-waits (≤180 s at 5 s), acquires the RTMPS destination (kept secret, never logged), starts the configured FFmpeg toward RTMPS, and confirms `streamStatus == active` on a thread parallel with the frame loop (≤120 s budget) before the chat-backed frame loop runs (`src/app/run/mod.rs:107–161`, `src/app/run/gate.rs`, `src/youtube/readiness_wait/`). The chat worker spawns ONLY after the confirmation gate observes `streamStatus == active`; its FIRST act resolves the broadcast's live chat id (bounded 6 × 5 s budget), then it establishes the `streamList` server-streamed connection and never re-connects on a schedule — it re-establishes only after a drop, carrying the accumulated `nextPageToken` (`src/app/run/chat_worker.rs`, `src/app/run/chat_stream_delivery.rs`, `src/youtube/live_chat_resolution.rs`). Resource fate: the application NEVER deletes or transitions broadcast/stream resources — `enableAutoStop=true` auto-completes the broadcast when ingestion ends; failed startups leave `ready` resources that the operator deletes in YouTube Studio (`src/app/run/mod.rs:1–7`). Exit contract: a clean stop and a pre-streaming stop exit 0; any fatal is one redacted `error:` line and exit 1.

### demo (Local-Only Offline Recording)

`demo` runs a fixed, deterministic message timeline (28 messages, 30-frame lead-in, at most one message per 15 frames, 60-frame tail → 496 frames ≈ 16.5 s at the example 30 FPS) through the SAME chat → renderer → frame → FFmpeg pipeline as `run`, records an MP4 under `videos/`, and NEVER contacts YouTube/OAuth/the chat transport (`src/app/demo/schedule.rs`, `src/app/demo/mod.rs`, [`demo.md`](demo.md)). It is the template for offline game-behavior development: swap the message source, keep the pipeline.

## Current Runtime Data Flow (Actual Source Modules)

The chain below is the ACTUAL implementation (every arrow names real files at implementation time); the conceptual target stays in brief §2.2/§2.3 and [`.agent/project-info/architecture.md`](../.agent/project-info/architecture.md).

### Module Responsibility Map

| Module | Responsibility | Depends on (boundary) |
| --- | --- | --- |
| `src/app/` | Startup funnel, CLI modes, the frame pipeline, the `auth`/`demo`/`run` orchestration, the chat worker, the opt-in diagnostics collector | all other modules (composition only) |
| `src/app/run/` + `src/app/pipeline/` | run-mode orchestration + the paced frame loop (ONE message per frame) | config, youtube (through seams), chat, renderer, streaming |
| `src/config/` | Typed JSON schema (`deny_unknown_fields`), 27 semantic validation rules, loading | none (leaf; typed errors, no panics) |
| `src/youtube/` | OAuth credential layer, broadcast/stream lifecycle (5-operation `LiveApiTransport`), RTMPS destination wrapper, Live Chat retrieval (the O6 live-chat-id lookup + the `streamList` seam family) | nothing app-side; MUST never depend on `chat`, `renderer`, `streaming`, or `demo` (`src/youtube/mod.rs`) |
| `src/chat/` | `ChatMessage { author, text }` (Display exactly `author: text`; optional ids/timestamps stay off-screen) + bounded visible FIFO `ChatStore` + append-only chat log | no YouTube API types, no network, no streaming, no FFmpeg (`src/chat/mod.rs`) |
| `src/renderer/` | `RenderInput` + `Renderer` trait + `TextRenderer` (ab_glyph) producing checked RGB24 `Frame`s | a pure local component: no YouTube/chat/network/FFmpeg (`src/renderer/mod.rs`) |
| `src/streaming/` | FFmpeg spawn/supervision, the locked argument vectors, frame-sink delivery as opaque bytes | only std + typed config; never chat/renderer/YouTube types (`src/streaming/mod.rs`) |
| `src/app/diagnostics/` | Opt-in read-only observability collector (bounded counters/histograms/in-flight map; JSONL writer with bounded rotation) | config + std only; never carries message text/authors/tokens/RTMPS URLs |

### The Message Path (YouTube Chat to Frame Loop)

```text
YouTube liveChatMessages.streamList (gRPC server streaming, Bearer-only)
  -> src/youtube/stream_{proto,transport,client,concrete}.rs + live_chat_stream_mapping.rs
        (published-proto wire types; readiness reserve before the streaming call; TEXT-message-only mapping)
  -> src/app/run/chat_worker.rs  (+ chat_stream_worker.rs / chat_stream_delivery.rs / chat_worker_helpers.rs)
        (ONE dedicated worker thread; confined current-thread Tokio runtime; catch_unwind containment;
         history-first baseline discard; try_send-only delivery; bounded reconnect with the retained token)
  -> bounded channel: std::sync::mpsc::sync_channel(64)  (src/app/run/chat_channel.rs)  + terminal cell
  -> src/app/run/source.rs :: ChatSource  (non-blocking try_recv; Ok(None) while open and empty)
  -> src/app/pipeline/runner.rs  (at most ONE message per frame iteration into src/chat/store.rs :: ChatStore)
```

The one-per-frame consumer rule gives a nominal capacity of `fps` messages/second (30 at the example config). The 64-slot channel is the ONLY queue between worker and frame loop; up to 64 queued-but-undelivered messages can be lost at shutdown and they are never logged, so log and video never diverge (the honest boundedness facts in [`production-run.md`](production-run.md)). The worker-side diagnostics hooks are `received`, `enqueued`, `full_retry`, `worker_retained`, and `reconnect_count` (`src/app/run/chat_stream_delivery.rs`; metric names in [`latency-test-map.md`](latency-test-map.md)).

### The Frame Path (Frame Loop to Renderer to FFmpeg to YouTube)

```text
src/app/pipeline/runner.rs (FPS-paced loop, StopFlag-polled)
  -> dequeue (ChatSource) -> node_message_dequeued diagnostics hook
  -> src/chat/store.rs :: ChatStore (bounded visible FIFO; append-only chat log first)
  -> render input = ChatStore.visible_text -> src/renderer (TextRenderer -> checked RGB24 Frame)
  -> frame write to FrameSink (opaque &[u8]; runner.rs never names FfmpegProcess)
  -> src/streaming/ffmpeg.rs :: FfmpegProcess.stdin  (locked streaming vector: libx264 veryfast,
     derived CBR video block, optional libmp3lame silent track; process-group console isolation)
  -> RTMPS (the redacting RtmpsDestination wrapper; the URL exists ONLY at the single reveal in src/app/run)
  -> YouTube Live
```

### Startup Orchestration (the run 10-Stage Funnel)

One sentence per stage, condensed from `src/app/run/mod.rs:107–161` and the startup section of [`production-run.md`](production-run.md): (1) install the single Ctrl+C stop pair; (2) run the credential service (load → verify-first → at most one refresh → atomic persist → re-verify); (3) gate the required scope before any resource creation; (4) build the transport bundle (`GoogleLiveApiTransport` + `GoogleLiveChatStreamTransport`); (5) prepare the broadcast (create → bind → readiness-wait); (6) perform the ONE RTMPS destination reveal; (7) construct diagnostics (`DiagnosticsCollector::from_config`, `None` unless enabled); (8) build the chat stage; (9) spawn the confirmation gate; (10) run the streaming frame loop. On Ctrl+C the shutdown is ordered: frame-loop stop flag → chat worker stop → YouTube wait cancellation → FFmpeg stdin EOF → drain + wait → the bounded chat-worker join at the gate finish (`src/app/run/stop_pair.rs`, `src/app/run/gate.rs`).

### Opt-In Diagnostics Overlay

`DiagnosticsCollector::from_config` returns `None` unless a capability is enabled, so a disabled config allocates nothing beyond one `Option` check per site. Rows are per-message but never carry text, authors, or secrets — the provider message id is the single correlation value. Two clock domains coexist deliberately (wall-clock `publishedAt` vs local monotonic `*_ns`) with the embedded `CLOCK_DOMAIN_NOTE`. Rotation bounds total disk to at most `max_file_bytes × max_generations` bytes. Field and operator detail lives in [`production-run.md#pipeline-observability-opt-in-diagnostics`](production-run.md#pipeline-observability-opt-in-diagnostics) and [`configuration.md#diagnostics`](configuration.md#diagnostics) — not duplicated here.

## Reusable Components vs Extension Points

### Reuse Unchanged

Reuse as-is: the frame pipeline and pacing (`src/app/pipeline/runner.rs`, `clock.rs`, `pacing.rs`), `ChatSource`, the renderer family, the streaming family (FFmpeg spawn, `FrameSink`, argv builders, redaction), the config layer, the diagnostics collector, and the credential service. Do not rewrite these to "make room" for a game — they are the tested infrastructure; a game bug must not be able to take down the stream.

### Extension Points

1. `src/app/pipeline/source.rs :: MessageSource` — already widened to `Result<Option<ChatMessage>, ChatSourceError>`; a game command adapter is a NEW `MessageSource` implementation, not an edit of the chat worker.
2. `src/renderer/renderer.rs :: Renderer` trait + `RenderInput` — a game renderer implements the trait; games map their render state to `RenderInput` at the composition site.
3. The demo timeline (`src/app/demo/schedule.rs` + `source.rs`) — the pattern for a deterministic game-side test source.
4. Config (`src/config/model.rs` + `validation.rs` precedent) — new game settings join the validated config with named validation rules and an example-template update.

### The Independence Rule

Game logic must not depend on YouTube API types, OAuth, broadcast IDs, RTMPS URLs, or FFmpeg command details. A game module may reach only `crate::chat::ChatMessage` (or its own command types), `crate::config`, and the renderer/pipeline seams. All YouTube specifics (`src/youtube/*`), all credential material, and all FFmpeg argument construction (`src/streaming/args.rs`, `stream_args.rs`) stay out of game modules — the app layer is the only place those meet.

## Recommended Future Game Flow (NOT Implemented)

### The Flow

```text
YouTube Chat Adapter  ->  ChatMessage  ->  Command Parsing/Validation
  ->  Game Input/Events  ->  Game State Update  ->  Render State
  ->  Renderer  ->  RGB24 Frames  ->  FFmpeg  ->  YouTube Live
```

This file implements none of it; a separated game TODO owns it. The adapter starts at the EXISTING `MessageSource` seam — the chat worker, channel, and frame loop above it are intentionally unchanged.

### The Five Separation Lines

1. Transport messages vs validated game commands — `ChatMessage` is `author` + `text` + metadata off-screen (`src/chat/model.rs`); a chat message is NOT a command until a game-side parser has validated it.
2. Command parsing/authorization/rate-limiting vs game rules — input governance lives in its own module, so rule changes never touch transport parsing.
3. Authoritative game state vs render-only state — only a projection of state becomes `RenderInput` (brief §9.6 precedent: the store owns visibility, the renderer draws what it is given).
4. Deterministic game update/tick logic vs frame rendering — tick functions are pure and testable with fake clocks (house rule: zero sleeps in tests); rendering happens at the FPS cadence.
5. Game simulation cadence vs output FPS — the frame loop is paced at `video.fps` (`src/app/pipeline/pacing.rs`); simulation ticks are independent (sub-frame or multi-frame rates), deliberately not coupled to the encode cadence.

## Timing and Scale Constraints

### The Latency Warning

YouTube Live Chat IS NOT a real-time game networking transport: a message reaches the app only after upstream delivery, and the viewer sees the frame only after encoding, ingest, and player buffering (the boundary-by-boundary map in [`latency-test-map.md`](latency-test-map.md)). The approximately 35 seconds from send to first-visible is the CURRENT OPERATOR-OBSERVED end-to-end baseline (TODO-dated 2026-10-31, motivating the observability work) — a measurement to investigate stage by stage, never a platform guarantee or constant, and never attributable to any single stage without measurements. Design latency-tolerant mechanics FIRST — turn-based moves, queued actions, voting, prediction-style prompts — and justify any real-time mechanic only after operator measurements (the procedure and template in [`latency-test-map.md`](latency-test-map.md)) show the model is wrong.

### Viewers Are Not Message Throughput

Load is driven by the MESSAGE rate and burst size, and by processing cost — not by viewer count. The consumer ceiling is one message per frame (nominally `fps`/second at the configured FPS), and upstream bursts larger than the 64-slot channel retention are retained by the worker, not dropped.

### Bounded Queues and Overload

The bounded-memory/backpressure behavior is a project invariant: no unbounded queues, no silent message drops, no weakened shutdown guarantees. Overload behavior must be DEFINED explicitly by the game design before implementation (retention-and-retry is the existing precedent in `src/app/run/chat_stream_delivery.rs`).

### Resolution Does Not Change Timing Semantics

Output resolution (1080p/4K) affects frame memory (`width × height × 3` bytes per frame, `src/renderer/frame.rs`), rendering cost, and encoding bandwidth — NOT the timing semantics of chat delivery; the chat → frame timing math is resolution-independent.

## Guidance for the First Game Prototype (10 items)

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

## Decision Checklist

- Where does the adapter attach? → default: a new `MessageSource` implementation fed by game commands, leaving `chat_stream_delivery.rs` byte-stable.
- Which mechanics first? → default: latency-tolerant (turn/vote/queue) until measurements justify otherwise.
- What is measured first? → default: enable diagnostics offline + the `latency_load` harness; live figures ONLY via the operator procedure.
- What state is authoritative vs rendered? → default: game state authoritative; render state a derived projection.
- What is the input policy (grammar/permissions/rate limit/overload/simultaneous inputs)? → default: none is invented by this repository — the game TODO must own it.
- Cadence: tick rate vs FPS? → default: independent; do not couple to `video.fps`.
- Offline dev path? → default: demo-pattern source + offline tests; no OAuth, no broadcast.
- Which contracts are frozen? → see [What Not to Change](#what-not-to-change) below.

## What Not to Change

1. The locked FFmpeg argument vectors and their CBR block (`src/streaming/args.rs`, `stream_args.rs` — the byte-pinned 33/42-token vectors).
2. The bounded channel and its delivery rules (`CHAT_CHANNEL_CAPACITY = 64`, `try_send`-only, retain-retry, non-blocking `try_recv`) — no unbounded queue, no silent drop.
3. The shutdown order and the single Ctrl+C handler pair (`src/app/run/stop_pair.rs`, `src/app/pipeline/interrupt.rs`).
4. The exit-code contract: usage 2 / startup+mode-fatal 1 / success+clean-stop 0, and the ONE `error:` line rule everywhere.
5. The resource-fate policy: never delete/transition YouTube resources; `enableAutoStop` owns completion.
6. The secret-redaction rules: OAuth/token material inside redacting wrappers (`src/youtube/token.rs`, `rtmps_ingestion.rs`, `rtmps_destination.rs`), capture-time diagnostics redaction (`src/streaming/ffmpeg.rs`, `pipe_drain.rs`), worker D-13 redaction (message text/authors never logged), diagnostics never carrying text/authors/secrets.
7. The credential record schema (`src/youtube/persisted_credentials.rs`, frozen 4-key) and the required scope.
8. The `demo` local-only contract: no YouTube contact of any kind.
9. The one-per-frame consumer and the continuous-FPS frame loop (a change here is an architecture decision, not a refactor).
10. The confined-runtime ruling: the chat worker's runtime stays confined to the worker thread.

## See Also

- [`../README.md`](../README.md) — project overview and the production-run summary.
- [`production-run.md`](production-run.md#pipeline-observability-opt-in-diagnostics) — operator startup stages, log lines, shutdown/fate, and the diagnostics operator guide.
- [`configuration.md`](configuration.md) — field reference, the 27 validation rules, CLI/exit codes.
- [`latency-test-map.md`](latency-test-map.md) — the canonical measurement procedure and the operator-run viewer-latency template.
- [`demo.md`](demo.md) — the offline local-only recording path.
- [`authentication.md`](authentication.md) — the OAuth `auth` flow and credential contract.
- [`.agent/project-info/architecture.md`](../.agent/project-info/architecture.md) — module map, data flow, boundary rules.
- [`.agent/project-structure.md`](../.agent/project-structure.md) — the per-file current map.

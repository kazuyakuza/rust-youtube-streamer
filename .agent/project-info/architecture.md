# Architecture

> Source of truth: [`brief.md`](brief.md) — if this summary conflicts with the brief, the brief wins; see [`instructions.md`](instructions.md) for project-info rules.

## Style

Modular monolith (brief §2.1): a single Rust executable organized into well-defined internal modules with explicit responsibilities, communicating through types, interfaces, and controlled dependencies. Microservices are not required and must not be introduced.

Principles:

* Separation of concerns.
* Dependency inversion at important boundaries.
* Explicit configuration.
* Strong typing.
* Testable business logic.
* Structured error handling.
* Minimal shared mutable state.
* Asynchronous execution where it provides a clear benefit.
* No unnecessary abstractions or frameworks.

## Module Map

Target layout (brief §4). For current repo state, see `context.md`.

| Path | Responsibility |
| --- | --- |
| `src/app/` | Orchestrates startup, task execution, error handling, and shutdown. |
| `src/config/` | Loads, validates, and exposes the JSON application configuration. |
| `src/youtube/` | Owns authentication (OAuth), broadcast lifecycle, stream configuration, and chat integration. |
| `src/chat/` | Application-level chat message types; bounded FIFO queue (`VecDeque<ChatMessage>`) of visible messages. |
| `src/renderer/` | Converts `RenderInput` into raw RGB24 video frames; trait contract `Renderer` / `TextRenderer` (brief §9.2). Must NOT depend on the YouTube API or broadcast lifecycle. |
| `src/streaming/` | Starts and supervises FFmpeg; manages the raw frame input pipeline. |

## Data Flow

The authoritative diagram lives in brief §2.2 and is reproduced here verbatim:

```text
                    Application
                         |
             +-----------+-----------+
             |                       |
      YouTube API                FFmpeg Process
             |                       ^
             v                       |
       YouTube Chat                  |
             |                       |
             v                       |
          Chat Module                 |
             |                       |
             v                       |
           ChatStore                  |
             |                       |
             v                       |
         RenderInput                  |
             |                       |
             v                       |
           Renderer                   |
             |                       |
             v                       |
        Raw Video Frames ------------+
                                     |
                                     v
                                  YouTube
```

Summary: YouTube → Chat → ChatStore → RenderInput → Renderer → raw frames → FFmpeg stdin → YouTube.

## Boundary Rules

* The renderer must not depend on YouTube-specific types or API behavior (brief §2.2, §9.1).
* FFmpeg knows nothing about chat messages, users, or game mechanics; it receives raw video frames via its standard input only (brief §2.2, §10.1–10.2). Rust never implements H.264 encoding or RTMPS transport itself.
* The chat store is transient in-memory state, never persisted or restored across restarts; the chat log is independent of the visible queue (brief §8.4, §11.2).
* Secrets — access/refresh tokens, stream keys, client secrets — are never logged or committed (brief §6.4, §7.4).

## Application Lifecycle

State chain (brief §14.1): Starting → ConfigurationValidated → Authenticated → BroadcastPrepared → Streaming → Stopping → Stopped. State transitions must be logged; additional intermediate states may be required by YouTube's broadcast and ingestion lifecycle.

Concurrent tasks coordinated after startup (brief §14.2): the video render loop, the chat listener, the FFmpeg process, and the application shutdown mechanism.

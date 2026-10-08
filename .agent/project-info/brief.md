# Project Brief — Rust YouTube Streamer Service

Document type: Technical Project Brief and Implementation Specification
Project name: Rust YouTube Streamer Service
Primary language: Rust
Initial targets: Windows and Linux
Production target: Ubuntu Server

## 1. Project Overview

### 1.1 Purpose

Build a Rust-based service that autonomously creates and manages a YouTube Live broadcast, generates video frames, and streams them to YouTube.

The initial prototype will display a black screen with incoming YouTube Live Chat messages rendered as white text. Its purpose is to validate the complete technical pipeline before introducing game logic, graphics, or interactive mechanics.

This repository must also serve as the architectural foundation for future YouTube chat-controlled games.

The MVP is intentionally simple in functionality, but its internal architecture must be modular, maintainable, testable, and extensible.

### 1.2 Core objectives

The service must:

1. Authenticate with YouTube using OAuth 2.0.
2. Create and manage the resources required for a new YouTube Live broadcast.
3. Produce a valid video stream from startup, even when no chat messages have arrived.
4. Render incoming chat messages onto a black background.
5. Continuously encode and transmit video using FFmpeg.
6. Maintain a bounded in-memory queue containing the messages currently visible on screen.
7. Write received messages to a separate log file.
8. Run on Windows and Linux without requiring architectural changes between platforms.
9. Handle operational errors, shutdown, and process failures predictably.
10. Expose clean interfaces that allow future renderers, game state, sprites, audio, and command processing to be introduced without rewriting the streaming infrastructure.

### 1.3 Non-goals

The first prototype will not implement:

* Game mechanics or game state.
* Chat commands, moderation, filtering, or authorization rules.
* Sprites, animations, complex scenes, or graphical user interfaces.
* Audio generation or mixing.
* Persistent storage for game state or chat history.
* Multiple concurrent broadcasts.
* A web dashboard.
* A distributed architecture or microservices.
* Automatic recovery from every possible crash scenario.

The goal is to prove the streaming architecture end to end, not to build the game itself.

## 2. Architectural Principles

### 2.1 Modular monolith

The application must be implemented as a single Rust executable organized into well-defined internal modules.

Modules should have explicit responsibilities and communicate through types, interfaces, and controlled dependencies.

Microservices are not required and must not be introduced.

The architecture should follow these principles:

* Separation of concerns.
* Dependency inversion at important boundaries.
* Explicit configuration.
* Strong typing.
* Testable business logic.
* Structured error handling.
* Minimal shared mutable state.
* Asynchronous execution where it provides a clear benefit.
* No unnecessary abstractions or frameworks.

The codebase should be structured like a maintainable production service, even though the first prototype has limited functionality.

### 2.2 High-level data flow

The target architecture is:

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

The diagram is conceptual. The implementation must account for asynchronous tasks and the fact that YouTube API operations and video-frame transmission have separate lifecycles.

The renderer must not depend on YouTube-specific types or API behavior.

FFmpeg must not know about chat messages, users, or game mechanics. It receives encoded-input-compatible raw video frames through its standard input.

### 2.3 Future architecture

The architecture should permit the following evolution without replacing the streaming infrastructure:

```text
YouTube Chat
     |
     v
Chat Processing
     |
     v
Command Processing
     |
     v
Game State
     |
     v
Render State / RenderInput
     |
     v
Renderer
     |
     v
Raw Video Frames
     |
     v
FFmpeg
     |
     v
YouTube Live
```

The first prototype may connect the chat store directly to the renderer input. The intermediate command-processing and game-state modules are future responsibilities and need not be implemented now.

## 3. Technology Choices

### 3.1 Rust

Rust is the primary implementation language.

Use idiomatic Rust rather than reproducing object-oriented class hierarchies from languages such as Java or C#.

Rust modules, structs, enums, traits, ownership, and explicit error types should form the core of the design.

### 3.2 Asynchronous runtime

Use Tokio for asynchronous operations where appropriate, particularly:

* YouTube API requests.
* Chat streaming.
* Coordination between application tasks.
* Process management and standard-input writes.
* Graceful shutdown and task supervision.

Do not introduce asynchronous complexity into components that can remain simple and synchronous.

For example, rendering a single frame can remain a synchronous operation if its cost and execution time are appropriate.

### 3.3 YouTube API integration

Do not use the `google-youtube3` crate as the foundation of this project because its maintenance status is unsuitable for this requirement.

Instead, evaluate maintained Rust libraries for:

* HTTP requests and response handling.
* JSON serialization and deserialization.
* OAuth 2.0 authorization and token refresh.
* gRPC and Protocol Buffers, if required for the selected chat API.

Use a direct YouTube API client implementation where necessary.

The YouTube integration must be isolated behind the application's own module interfaces so that the underlying HTTP, OAuth, or gRPC implementation can be replaced without changing the renderer or application logic.

Do not implement the entire Google API protocol manually if suitable maintained libraries exist.

### 3.4 FFmpeg

FFmpeg must run as an external process.

The Rust application is responsible for launching and supervising the executable, configuring its arguments, and writing raw frames to its standard input.

FFmpeg is responsible for video encoding and transmission to YouTube.

Do not require an FFmpeg Rust library or embed FFmpeg into the Rust executable.

The FFmpeg executable path must be configurable.

Examples:

* Windows: `C:\ffmpeg\bin\ffmpeg.exe`
* Linux: `/usr/bin/ffmpeg`

These are examples, not hardcoded paths.

### 3.5 Configuration and logging libraries

Select maintained libraries for:

* JSON configuration loading and validation.
* Structured application logging.
* Error context and propagation.
* OAuth token persistence.
* Graceful process shutdown.

Dependency selection must favor actively maintained, widely used crates with appropriate licenses.

Pin dependencies through the normal Cargo dependency and lockfile workflow.

Avoid adding a dependency when the standard library or an existing project dependency provides a sufficient solution.

## 4. Project Structure

Use a modular directory structure similar to the following:

```text
rust-youtube-streamer/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .gitignore
├── config/
│   ├── config.example.json
│   └── config.json
├── credentials/
│   └── README.md
├── fonts/
│   └── README.md
├── logs/
│   └── .gitkeep
├── docs/
│   ├── architecture.md
│   ├── authentication.md
│   ├── streaming-recovery.md
│   └── development.md
└── src/
    ├── main.rs
    ├── app/
    │   ├── mod.rs
    │   └── application.rs
    ├── config/
    │   ├── mod.rs
    │   └── model.rs
    ├── youtube/
    │   ├── mod.rs
    │   ├── auth.rs
    │   ├── broadcast.rs
    │   ├── stream.rs
    │   └── chat.rs
    ├── chat/
    │   ├── mod.rs
    │   ├── model.rs
    │   └── store.rs
    ├── renderer/
    │   ├── mod.rs
    │   ├── renderer.rs
    │   ├── text.rs
    │   └── frame.rs
    └── streaming/
        ├── mod.rs
        └── ffmpeg.rs
```

This structure is a starting point, not a requirement to create empty files for every future responsibility.

The implementation agent may refine the layout when justified, provided the module boundaries and responsibilities remain clear.

### Module responsibilities

`app`

Orchestrates startup, task execution, error handling, and shutdown.

`config`

Loads, validates, and exposes the application configuration.

`youtube`

Owns authentication, broadcast lifecycle operations, stream configuration, and chat integration.

`chat`

Defines application-level chat message types and manages the bounded queue of visible messages.

`renderer`

Converts renderer input into raw video frames. It must not depend on the YouTube API or broadcast lifecycle.

`streaming`

Starts and supervises FFmpeg and manages the raw frame input pipeline.

## 5. Configuration

### 5.1 General requirements

All relevant runtime settings must be externalized into JSON configuration.

The application must load configuration at startup and validate it before starting the broadcast.

Invalid configuration must result in a clear error and a non-zero exit code.

The repository must include `config/config.example.json`. A real `config/config.json` must be excluded from version control.

Secrets, OAuth credentials, and tokens must never be committed to the repository.

### 5.2 Example configuration

The following illustrates the intended configuration structure. The implementation agent may refine property names and add validated fields when required.

```json
{
  "youtube": {
    "broadcast": {
      "title": "Rust YouTube Streamer Prototype",
      "description": "YouTube Live streaming prototype",
      "privacy_status": "unlisted"
    }
  },
  "video": {
    "width": 1920,
    "height": 1080,
    "fps": 30,
    "pixel_format": "rgb24"
  },
  "renderer": {
    "font": "fonts/console.ttf",
    "font_size": 32,
    "line_height": 40,
    "left_margin": 20,
    "top_margin": 20,
    "right_margin": 20,
    "bottom_margin": 20,
    "text_color": "#FFFFFF",
    "background_color": "#000000"
  },
  "chat": {
    "log_file": "logs/chat.log"
  },
  "ffmpeg": {
    "executable": "ffmpeg",
    "video_codec": "libx264",
    "preset": "veryfast",
    "bitrate": "6000k"
  }
}
```

Important notes:

* The actual FFmpeg executable must be configured explicitly for the target environment; `"ffmpeg"` is only an illustrative value.
* The font file must be supplied or otherwise documented. The implementation must not silently depend on an unspecified system font.
* `max_messages` must not be an arbitrary fixed configuration value. The visible message capacity is derived from video height, margins, and line height.
* Any required additional fields, including FFmpeg input format, stream URL construction, OAuth paths, timeout settings, or API options, must be added as appropriate.
* Configuration must distinguish application settings from secrets and credential material.

### 5.3 Configuration validation

Validate all:

* Positive video dimensions and frame rate.
* Supported pixel format.
* Valid renderer dimensions and margins.
* Positive font size and line height.
* A usable font path.
* A non-empty FFmpeg executable path.
* Valid codec, preset, and bitrate settings.
* Supported YouTube privacy values.
* Valid file and directory paths.
* etc.

The renderer must reject or clearly report invalid geometry instead of producing malformed frames.

## 6. YouTube Authentication

### 6.1 OAuth 2.0

Authentication must use Google's supported OAuth 2.0 mechanisms.

A static API key is not a substitute for OAuth authorization for account operations that require the user's authenticated identity and permissions.

The application must support two execution modes:

```text
rust-youtube-streamer-service auth
rust-youtube-streamer-service run
```

Equivalent subcommand naming is acceptable if documented.

### 6.2 Authentication mode

The `auth` mode must:

1. Load the OAuth client configuration.
2. Start the supported installed-application authorization flow.
3. Display the authorization URL in the console.
4. Allow the user to authorize the application in a browser.
5. Receive the authorization result through a local callback when supported.
6. Exchange the authorization code for tokens.
7. Verify that the resulting credentials can access the required YouTube resources.
8. Persist the required token data securely.
9. Exit after successful authorization and verification.

Do not request that the user manually copy an access token if a supported authorization-code flow can handle the process.

### 6.3 Normal execution mode

The `run` mode must:

1. Load the persisted credentials.
2. Refresh expired access tokens when possible.
3. Verify that the required scopes and permissions are available.
4. Fail clearly if authorization is missing, revoked, or insufficient.
5. Avoid requiring interactive authorization during normal execution when valid refresh credentials are available.

OAuth client configuration and token storage paths must be configurable.

The first authorization may require a user interaction. Subsequent operation should be suitable for unattended execution on Ubuntu Server, subject to Google's OAuth policies and the application's authorization status.

### 6.4 Credential security

* Exclude credentials and tokens from version control.
* Do not log access tokens, refresh tokens, client secrets, or stream keys.
* Restrict token-file permissions where supported.
* Document the procedure for initial authorization and secure deployment.
* Do not assume that a token created on one machine can be copied to another without considering file permissions, OAuth configuration, and deployment security.

## 7. YouTube Broadcast Lifecycle

### 7.1 Ownership

The application must manage the broadcast lifecycle through the YouTube API rather than requiring the user to manually create each broadcast in YouTube Studio.

Each normal service execution should create a new stream and broadcast rather than attempting to reuse existing resources.

The initial broadcast privacy must default to `unlisted`, and its title and description must be configurable.

### 7.2 Startup sequence

The application should orchestrate the following sequence:

1. Load and validate configuration.
2. Initialize logging.
3. Load OAuth credentials and validate access.
4. Create a new YouTube live stream resource.
5. Create a new broadcast with the configured metadata and privacy.
6. Associate the broadcast with the stream.
7. Obtain and validate the ingestion information.
8. Initialize the renderer and calculate the visible message capacity.
9. Initialize the chat store and log file.
10. Start FFmpeg with the correct video input and output parameters.
11. Begin producing and transmitting video frames.
12. Establish the YouTube Live Chat connection and discard its initial historical messages.
13. Complete the required broadcast readiness and transition operations according to the current YouTube API lifecycle.
14. Continue running until shutdown or a fatal error occurs.

The exact order of resource creation, broadcast readiness, FFmpeg startup, and state transitions must be verified against the current YouTube Live Streaming API documentation. The list above defines the responsibilities, not permission to ignore API prerequisites.

The application must not announce a successful live state merely because an HTTP request succeeded or FFmpeg started.

### 7.3 API integration

Use maintained HTTP and JSON libraries for YouTube Data API operations.

The implementation must support the necessary operations for:

* Creating live stream resources.
* Creating broadcasts.
* Binding streams to broadcasts.
* Inspecting stream and broadcast states.
* Performing required lifecycle transitions.
* Retrieving ingestion information.
* Retrieving chat connection information.

Do not hardcode undocumented API behavior.

The implementation must respect API scopes, quota limits, request errors, and the lifecycle constraints imposed by YouTube.

### 7.4 Ingestion protocol

Prefer RTMPS whenever the supplied ingestion endpoint supports it.

RTMPS provides TLS protection for the connection between the service and YouTube.

The stream key and ingestion URL must be treated as sensitive operational data. They must not appear in ordinary application logs.

## 8. YouTube Live Chat Integration

### 8.1 Connection mechanism

Evaluate YouTube's low-latency server-streaming chat API, `streamList`, as the preferred approach.

This API uses gRPC server streaming rather than WebSocket.

The implementation must evaluate maintained Rust gRPC and Protocol Buffers libraries and the current availability of the required YouTube protocol definitions.

If `streamList` cannot be implemented reliably with suitable dependencies, document the limitation and evaluate the supported polling API as a fallback. The preferred design should not silently substitute a less suitable approach.

### 8.2 Message handling

Only ordinary text chat messages are required for the MVP.

The application must ignore other event types, including Super Chats, polls, membership events, and other non-ordinary-message events.

No moderation, filtering, command interpretation, or game-related processing will be implemented.

For each supported message, extract:

* Author display name.
* Message text.
* Message identifier and timestamp when supplied by the API, for internal correctness and duplicate prevention.

The visible format must be:

```text
username: message
```

The display name is the preferred author identifier. If unavailable, use a suitable stable channel identifier when available.

Internal message metadata does not need to be printed on screen.

### 8.3 Initial history

The stream must start with an empty chat display.

If the chat API sends recent messages when the connection is established, those historical messages must be discarded.

Only messages received after the initial connection baseline has been established may enter the visible queue and chat log.

The implementation must handle reconnects carefully:

* Do not replay old chat history into the display.
* Do not duplicate messages already processed.
* Do not treat historical messages returned during reconnection as new messages.
* Use message identifiers, timestamps, or the API's supported continuation mechanism as appropriate.

### 8.4 Message storage

Maintain an in-memory FIFO queue, such as `VecDeque<ChatMessage>`, containing only messages currently eligible to be displayed.

When a new message arrives:

1. Normalize the message into the application's internal message model.
2. Append the message to the log.
3. Add the message to the visible queue.
4. Remove the oldest message if the queue exceeds its calculated capacity.

The queue is transient. It must not be persisted or restored across application restarts.

The log is independent of the visible queue and is not a source of game state.

## 9. Renderer Architecture

### 9.1 Design goal

The renderer must be an independent component that receives render input and produces a raw video frame.

It must not contain YouTube API calls, chat connection logic, or FFmpeg process-management logic.

### 9.2 Rust abstraction

Use a trait to define the renderer contract, with a concrete struct providing the MVP implementation.

A conceptual interface is:

```rust
pub trait Renderer {
    fn render(&mut self, input: &RenderInput) -> Frame;
}
```

The actual return type and error behavior may be refined during implementation. For example, rendering can return a typed error if a frame cannot be produced.

The initial render input is:

```rust
pub struct RenderInput {
    pub text: String,
}
```

The initial implementation may be named `TextRenderer`.

The trait defines the contract; `TextRenderer` implements it using Rust's trait implementation mechanism:

```rust
impl Renderer for TextRenderer {
    // Rendering implementation
}
```

Do not introduce class inheritance or an unnecessary generic rendering framework.

The abstraction must make it possible to add other renderer implementations later without changing YouTube integration.

### 9.3 Rendering behavior

The renderer must implement the following behavior:

Empty input

`render("")` produces a completely black frame.

Non-empty input

The renderer produces a black frame with the supplied multiline text drawn in white.

For example:

```text
alice: Hello!
bob: Testing the stream.
charlie: This is the third message.
```

The string represents the complete visible text, not an individual chat message.

The renderer must draw the supplied lines in order, using the configured font, font size, line height, and margins.

It must not add message history, perform scrolling, or decide which messages should remain visible. Those responsibilities belong to the chat store and application layer.

### 9.4 Frame format

For the MVP, the renderer should produce raw `RGB24` frames.

Each frame must contain exactly the pixel data required for the configured width and height.

The expected byte count per frame is:

`width × height × 3`

The background must be black, and text must be white.

The renderer must not depend on FFmpeg to draw the text. Text rasterization occurs in Rust, and FFmpeg receives completed raw frames.

### 9.5 Horizontal overflow

Text that exceeds the available horizontal rendering area must be clipped at the frame boundary.

The implementation must not wrap text, dynamically resize fonts, or implement truncation logic for the MVP.

Characters beyond the right edge must not appear outside the video frame.

### 9.6 Vertical layout

The visible line capacity must be calculated from the configured video height, vertical margins, and line height.

Conceptually:

```text
available_height = height - top_margin - bottom_margin

visible_lines = floor(available_height / line_height)
```

The implementation must validate the inputs and ensure that at least one line can be displayed.

The visible queue capacity must be derived from this value, not independently configured as a fixed number.

For example, with a height of 1080 pixels, top and bottom margins of 20 pixels, and a line height of 40 pixels, the capacity is:

```text
floor((1080 - 20 - 20) / 40) = 26 lines
```

This is an illustrative result for those specific values.

When the queue reaches capacity, adding a new message removes the oldest message. The newest message appears at the bottom of the displayed list.

### 9.7 Rendering frequency

Rendering must run continuously at the configured FPS, independently of chat activity.

The renderer must not be invoked only when a message arrives.

If the chat is empty, it continues to generate black frames.

If no new messages arrive, the currently visible messages remain on screen and are rendered repeatedly.

The application must avoid uncontrolled frame accumulation if FFmpeg or the underlying pipe cannot keep up with the configured frame rate.

## 10. FFmpeg and Video Streaming

### 10.1 Responsibility boundary

Rust produces raw frames. FFmpeg encodes and transmits them.

The pipeline is:

```text
TextRenderer
    |
    v
RGB24 Raw Frames
    |
    v
FFmpeg Standard Input
    |
    v
H.264 Encoding
    |
    v
RTMPS Ingestion
    |
    v
YouTube Live
```

Rust must not implement H.264 encoding or RTMPS transport itself.

### 10.2 FFmpeg input

FFmpeg must be launched as an external process using the configured executable path.

The application must configure the raw-video input with the correct:

* Input format.
* Video width and height.
* Frame rate.
* Pixel format.
* Standard-input source.

The exact FFmpeg arguments must be generated from validated configuration and tested against the selected FFmpeg version.

Raw frame data must be written to the process's standard input in the exact format and order FFmpeg expects.

### 10.3 Video encoding

The initial video encoding defaults are:

* Codec: `libx264`.
* Preset: `veryfast`.
* Bitrate: `6000k`.
* Frame rate: `30 FPS`.
* Resolution: `1920×1080`.
* Renderer output format: `RGB24`.

These are defaults and must remain configurable.

The implementation should select appropriate output pixel-format and encoding parameters for YouTube ingestion, using current documented compatibility requirements.

### 10.4 Audio

Audio is outside the scope of the MVP.

The initial stream must be video-only if YouTube accepts the selected ingestion configuration.

The implementation agent must verify that the chosen configuration is accepted by YouTube and must not fabricate silent audio unless the API or ingestion requirements make it necessary.

If audio becomes necessary, document the requirement and implement it as an explicit design change.

The architecture should permit audio to be added later without requiring the renderer to become responsible for YouTube transport.

### 10.5 Process supervision

The streaming module must:

* Start FFmpeg.
* Capture and process diagnostic output.
* Detect unexpected process termination.
* Detect broken standard input and pipe errors.
* Avoid writing to a process that has already exited.
* Wait for the process during shutdown.
* Report exit codes and useful diagnostics.
* Prevent uncontrolled restart loops.

FFmpeg standard output and standard error must be handled so that the process cannot deadlock because an output pipe fills.

The application must not claim that the stream is healthy solely because the FFmpeg process exists. Where practical, health checks should distinguish process health, successful frame delivery, and YouTube broadcast state.

## 11. Logging and Observability

### 11.1 Application logging

Use structured application logging.

Log at least:

* Application startup and selected mode.
* Configuration loading and validation.
* OAuth initialization and refresh outcomes.
* Broadcast and stream creation.
* YouTube API errors.
* Chat connection, disconnection, and reconnection.
* Number of messages processed where useful.
* FFmpeg startup, exit, and diagnostic errors.
* Frame-pipeline failures.
* Shutdown sequence and exit reason.

Logs must include sufficient context for troubleshooting without exposing secrets.

Avoid logging every video frame under normal operation.

### 11.2 Chat log

Every newly accepted ordinary chat message must be written to a separate append-only log file.

Each record must contain:

* A timestamp.
* The author's display name or fallback identifier.
* The message text.

The logical record format is:

```text
2026-10-08T12:34:56Z alice: Hello!
```

The exact timestamp format may be adjusted, but it must be documented and unambiguous.

Use UTF-8.

The chat log is a redundant record of received messages and is not part of the game state or a persistence mechanism for the visible queue.

The application must handle log-file creation and write failures explicitly.

### 11.3 Operational documentation

Document:

* How to inspect application logs.
* How to identify FFmpeg failures.
* How to determine whether YouTube has accepted the broadcast.
* How to stop a broadcast manually if the process crashes.
* How to recover from revoked OAuth credentials.
* How to troubleshoot common ingestion and API errors.

## 12. Error Handling and Recovery

### 12.1 General policy

Not all failures should trigger retries.

The application must classify failures into recoverable and fatal categories.

Examples of potentially recoverable failures:

* Temporary network errors.
* Transient API rate limits.
* Temporary chat connection loss.
* Token refresh requests failing due to temporary connectivity problems.

Examples of potentially fatal failures:

* Invalid configuration.
* Missing required OAuth credentials.
* Insufficient OAuth scopes.
* Unsupported encoder configuration.
* Failure to create required broadcast resources.
* FFmpeg startup failure.
* Unrecoverable frame-pipeline errors.

The exact classification must be implemented according to the operation and the API response, not solely according to a generic error message.

### 12.2 Retry behavior

Implement bounded retries with appropriate delays for operations that are safe to retry.

Use exponential backoff with jitter where appropriate.

Do not retry invalid requests indefinitely.

Do not create duplicate broadcasts or streams merely because an API request timed out after the server may already have processed it. Where possible, reconcile the result or report the ambiguous state for operator intervention.

### 12.3 Fatal failures

When a fatal failure occurs:

1. Log the error with context.
2. Stop accepting new work.
3. Attempt orderly cleanup of owned resources and processes.
4. Terminate FFmpeg when appropriate.
5. Attempt to finalize the broadcast through the API when the application is still operational and the state allows it.
6. Return a non-zero exit code.

Cleanup must use bounded waits and must not prevent the process from terminating indefinitely.

### 12.4 Abrupt crashes

An abrupt process crash cannot execute the application's own cleanup code.

The MVP does not require a separate self-restarting supervisor.

Document the manual recovery procedure in `docs/streaming-recovery.md`, including how to inspect and end an active broadcast in YouTube Studio.

YouTube may detect a missing ingestion signal, but the application must not assume that the broadcast will always terminate immediately or automatically.

A future Ubuntu deployment may use systemd or another external supervisor. A supervisor can restart the service, but it does not by itself guarantee correct cleanup of a previously created broadcast.

## 13. Cross-Platform Requirements

### 13.1 Supported environments

The architecture must support:

* Windows development and execution.
* Linux development and execution.
* Ubuntu Server deployment.

The first manual validation may occur on Windows.

### 13.2 Portability

Avoid unnecessary operating-system-specific assumptions.

In particular:

* Use configurable executable paths.
* Use platform-appropriate path handling.
* Avoid hardcoded path separators.
* Handle child processes and shutdown behavior appropriately for each platform.
* Use portable Rust libraries where practical.
* Keep OS-specific behavior behind small implementation boundaries.

Differences in process signaling, permissions, executable naming, and file locations must be handled explicitly where necessary.

### 13.3 Docker

Docker is optional.

It may be used for reproducible builds, CI, or deployment, but the service must not require Docker to run natively on Windows or Linux.

Do not bundle FFmpeg into the Rust executable.

The README must document how to install or obtain a compatible FFmpeg binary and configure its location.

## 14. Application Lifecycle

### 14.1 Normal startup

The application must not enter its continuous streaming loop until required configuration, authentication, and initial resource creation have succeeded.

The implementation should define explicit lifecycle states, such as:

```text
Starting
  |
  v
ConfigurationValidated
  |
  v
Authenticated
  |
  v
BroadcastPrepared
  |
  v
Streaming
  |
  v
Stopping
  |
  v
Stopped
```

Additional intermediate states may be required by YouTube's broadcast and ingestion lifecycle.

State transitions must be logged.

### 14.2 Concurrent tasks

Once startup is complete, the application must coordinate at least:

* The video render loop.
* The chat listener.
* The FFmpeg process.
* The application shutdown mechanism.

The chat listener updates the chat store independently of the rendering schedule.

The render loop reads the current visible messages and produces frames continuously.

Shared state should use the simplest synchronization mechanism appropriate for the selected runtime. Do not introduce shared mutable state unnecessarily.

### 14.3 Graceful shutdown

The application must support graceful shutdown using the appropriate operating-system signal or mechanism.

On shutdown:

1. Stop accepting new chat messages.
2. Stop the render loop.
3. Stop sending additional frames.
4. Close FFmpeg's frame input.
5. Allow FFmpeg a bounded opportunity to exit.
6. Terminate the process if it does not exit within the configured timeout.
7. Finalize the YouTube broadcast where possible and appropriate.
8. Flush and close the chat log.
9. Finish logging and exit cleanly.

The exact order must respect process dependencies and the broadcast lifecycle.

## 15. Testing Strategy

Testing must distinguish unit tests from integration tests requiring real external services.

### 15.1 Unit tests

At minimum, test:

Configuration

* Valid configuration loads correctly.
* Invalid values produce useful errors.
* Missing required settings are detected.

Renderer

* Empty input produces a black frame.
* Non-empty input produces a frame with the expected dimensions.
* Output byte length is correct for RGB24.
* Text layout respects margins and line height.
* Horizontal clipping is applied.
* Invalid geometry is rejected.

Chat store

* Messages are added in the correct order.
* The oldest message is removed when capacity is exceeded.
* The queue never exceeds its capacity.
* The visible text is constructed in the expected format.

Message processing

* Ordinary text messages are accepted.
* Unsupported event types are ignored.
* Historical messages are discarded during initial connection.
* Duplicate prevention works according to the selected chat API mechanism.

Streaming process

* FFmpeg arguments are generated correctly from configuration.
* Process-start failures are reported.
* Process termination is detected.
* Pipe failures are handled without hanging the application.

### 15.2 Integration tests

Where practical, use test doubles for YouTube API and process boundaries.

Tests must not accidentally create public broadcasts.

Any live integration test must be explicitly configured and documented.

The initial end-to-end validation will require a real authorized YouTube account and a test broadcast with unlisted privacy.

### 15.3 Manual end-to-end validation

The initial validation procedure must demonstrate that:

1. OAuth authorization succeeds.
2. Normal startup creates the required YouTube resources.
3. A valid live video stream begins.
4. The screen is black before chat messages arrive.
5. Incoming ordinary chat messages appear on screen.
6. Text is rendered as white characters on a black background.
7. Messages remain visible when no new messages arrive.
8. The oldest visible message disappears when the queue fills.
9. Horizontal overflow is clipped.
10. The chat log contains timestamps and message text.
11. Unsupported event types are not displayed.
12. Shutdown attempts to finalize the stream correctly.
13. A failed FFmpeg process is detected and reported.

The implementation agent is not required to validate the resulting executable on every supported operating system during the initial implementation phase. The supported build targets and the actually tested environments must be documented accurately.

## 16. Implementation Guidelines for the Coding Agent

The implementation agent must:

1. Read this brief before modifying the repository.
2. Follow the stated architecture and avoid unnecessary redesign.
3. Verify current YouTube API behavior and OAuth requirements against authoritative documentation.
4. Evaluate crate maintenance and compatibility before introducing dependencies.
5. Keep YouTube-specific code isolated from the renderer and chat store.
6. Avoid adding game mechanics or other features outside the MVP.
7. Implement and test components incrementally.
8. Document assumptions when external API behavior cannot be verified.
9. Never report a successful live broadcast without evidence that the relevant operations succeeded.
10. Never expose credentials, tokens, or stream keys in logs.
11. Keep the build and runtime workflow documented for both Windows and Linux.
12. Ensure the architecture supports future extensions without implementing those extensions prematurely.

If a requirement conflicts with a current YouTube API limitation, the agent must document the conflict and propose the smallest necessary adjustment rather than silently changing the project's intended behavior.

## 17. Final Definition of the MVP

The Rust YouTube Streamer Service MVP is a cross-platform Rust application that authorizes access to YouTube, creates a new unlisted live broadcast, generates continuous black video frames, displays newly received ordinary chat messages as white text, encodes the video through an external FFmpeg process, and sends the resulting stream to YouTube over RTMPS when supported.

It maintains only the currently visible messages in memory, writes received messages to an independent timestamped log, and handles startup failures, runtime errors, and graceful shutdown through explicit application lifecycle management.

The result is not yet a game. It is the reusable streaming foundation upon which future chat-controlled games can be built.

<!-- DO NOT DELETE NEXT SECTION -->

## Important Note for AI Agents

All agents working on this project MUST adhere to the workflows and rules outlined in [AI Agent Onboarding document](../../AGENTS.md).

Before starting any task:

1. Review `AGENTS.md`: is the primary source of instructions for agents.
2. Follow Workflows: follow the procedures defined in `.agent/WORKFLOWS.md`, especially the `.kilo/commands/critical-workflow.md`.

<!-- END DO NOT DELETE -->

# Product — Rust YouTube Streamer Service

> Source of truth: [`brief.md`](brief.md) — if this summary conflicts with the brief, the brief wins; see [`instructions.md`](instructions.md) for project-info rules.

## Problem

There is no validated pipeline for autonomously creating and managing a YouTube Live broadcast from Rust and streaming generated frames to it. The existing `google-youtube3` crate is unsuitable for this requirement (brief §3.3). The MVP must validate the complete technical pipeline end to end before any game logic, graphics, or interactive mechanics are introduced.

## Product Description

The Rust YouTube Streamer Service is a cross-platform Rust service that authorizes access to YouTube via OAuth 2.0, creates a new unlisted YouTube Live broadcast, generates continuous video frames from Rust (MVP: a black screen with incoming chat messages rendered as white text), encodes the frames through an external FFmpeg process, and transmits the resulting stream to YouTube over RTMPS when supported.

## Core Objectives

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

## Non-Goals (MVP)

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

## Future Direction

The repository must serve as the architectural foundation for future YouTube chat-controlled games: the target evolution inserts Chat Processing, Command Processing, and Game State between the chat store and the renderer input (brief §2.3). The streaming infrastructure must not need replacement when those modules are introduced.

//! Typed YouTube Live API resource models (plan F1, D-T1-2): serde views
//! of the documented `liveBroadcasts`/`liveStreams` response fields plus
//! the typed insert-request bodies. Responses IGNORE unknown fields (the
//! deliberate opposite of the config module's `deny_unknown_fields` —
//! Google adds resource fields freely); missing parts stay `None` here
//! and surface as typed failures at the consumer boundary (Task 2).
//! Request bodies lock the exact documented JSON shapes offline before
//! any transport exists. Only config dependency: `VideoConfig` (D-T1-6).
//!
//! Grounding: the field spellings mirror the liveBroadcasts reference
//! (https://developers.google.com/youtube/v3/live/docs/liveBroadcasts) and
//! the liveStreams reference
//! (https://developers.google.com/youtube/v3/live/docs/liveStreams); the
//! FFmpeg destination source field is `cdn.ingestionInfo.rtmpsIngestionAddress`
//! per https://developers.google.com/youtube/v3/live/guides/rtmps-ingestion.
//!
//! Staging: consumed by Task 2's concrete transport
//! (`google_live_api_transport`) and orchestrator (`live_ingestion`);
//! truly live when Task 3's run wiring roots the chain.
//!
//! Design plan:
//! `.kilo/plans/20261013-phase05-task1-official-api-research-design.md`
//! (§1.1/§1.3, F1); TODO `.agent/todos/20261013/20261013-todo-1.md` Task 1.

pub(crate) use broadcast_resource::{LiveBroadcast, LiveBroadcastList};
pub(crate) use commands::{BindCommand, CreateBroadcastCommand, CreateStreamCommand};
pub(crate) use stream_cdn::StreamCreationCdn;
pub(crate) use stream_resource::{LiveStream, LiveStreamList};

// Re-exported for Task 2's concrete transport and for offline model tests.
pub(crate) use request_bodies::{BroadcastRequestBody, StreamRequestBody};

mod broadcast_resource;
mod commands;
mod request_bodies;
mod stream_cdn;
mod stream_resource;

#[cfg(test)]
#[path = "../live_models_tests.rs"]
mod live_models_tests;

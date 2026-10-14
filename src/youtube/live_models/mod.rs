//! Typed YouTube Live API resource models (plan F1, D-T1-2): serde views
//! of the documented `liveBroadcasts`/`liveStreams` response fields plus
//! the typed insert-request bodies. Responses IGNORE unknown fields (the
//! deliberate opposite of the config module's `deny_unknown_fields` —
//! Google adds resource fields freely); missing parts stay `None` here
//! and surface as typed failures at the consumer boundary (Task 2).
//! Request bodies lock the exact documented JSON shapes offline before
//! any transport exists. Only config dependency: `VideoConfig` (D-T1-6).

pub(crate) use broadcast_resource::LiveBroadcast;
pub(crate) use commands::{BindCommand, CreateBroadcastCommand, CreateStreamCommand};
pub(crate) use stream_resource::LiveStream;

// Test-only re-exports: the offline model tests deserialize the list
// pages and serialize the insert bodies, which requires naming these
// types; production code never uses them until Task 2's transport.
#[cfg(test)]
pub(crate) use broadcast_resource::LiveBroadcastList;
#[cfg(test)]
pub(crate) use request_bodies::{BroadcastRequestBody, StreamRequestBody};
#[cfg(test)]
pub(crate) use stream_cdn::StreamCreationCdn;
#[cfg(test)]
pub(crate) use stream_resource::LiveStreamList;

mod broadcast_resource;
mod commands;
mod request_bodies;
mod stream_cdn;
mod stream_resource;

#[cfg(test)]
#[path = "../live_models_tests.rs"]
mod live_models_tests;

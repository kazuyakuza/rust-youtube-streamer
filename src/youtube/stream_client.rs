//! The concrete tonic adapter for the streamList gRPC call (record P7-12):
//! endpoint + service-path constants pinned by test; the Bearer metadata is
//! the ONLY authentication form (never an API key — record §3-C4); the
//! endpoint construction is offline-instantiable (pinned without dialing);
//! the REAL `open()` path dials Google only in production runs (the operator
//! manual E2E tests it; the offline rosters stop at the seam).
//!
//! tonic surfaces verified 2026-10-29 (docs.rs/tonic/0.13.1): `Grpc::
//! server_streaming(request, path, codec) -> Result<Response<Streaming<M2>>,
//! Status>`; `Streaming::message() -> Result<Option<T>, Status>`; `Endpoint::
//! from_shared/tls_config/connect_timeout/connect`; `ClientTlsConfig::
//! with_enabled_roots`; the codegen feature re-exports `tonic::async_trait`
//! and `tonic::codegen::http`.

use std::time::Duration;

use tonic::metadata::{Ascii, MetadataValue};

use super::live_chat_stream_mapping::map_stream_batch;
use super::live_error::{classify_stream_status, LiveApiError};
use super::stream_proto::{LiveChatMessageListRequest, LiveChatMessageListResponse};
use super::stream_transport::{LiveChatStreamSession, YieldedChatStreamBatch};
use super::token::AccessToken;

pub(crate) const STREAM_ENDPOINT_URL: &str = "https://youtube.googleapis.com";
pub(crate) const STREAM_LIST_PATH: &str = "/youtube.api.v3.V3DataLiveChatMessageService/StreamList";
pub(crate) const AUTHORIZATION_METADATA_KEY: &str = "authorization";
/// Fixed stream family failure reasons (secret-safe: no server text).
pub(super) const STREAM_CONNECT_TIMEOUT_REASON: &str =
    "the stream endpoint connect attempt timed out";
pub(super) const STREAM_CONNECT_FAILURE_REASON: &str = "the stream endpoint connection failed";
pub(super) const STREAM_READY_FAILURE_REASON: &str = "the stream endpoint readiness reserve failed";
const STREAM_ENDPOINT_BUILD_REASON: &str = "the stream endpoint could not be built";

/// P7-6 stream family connect bound (Task 1 record §A.3): 10 s.
pub(super) const CHAT_STREAM_CONNECT_TIMEOUT_MS: u64 = 10_000;

pub(crate) fn bearer_token_metadata(
    access_token: &AccessToken,
) -> Result<MetadataValue<Ascii>, LiveApiError> {
    if !access_token.secret().is_ascii() {
        return Err(LiveApiError::AuthorizationRejected);
    }
    MetadataValue::try_from(format!("Bearer {}", access_token.secret()))
        .map_err(|_| LiveApiError::AuthorizationRejected)
}

pub(crate) fn stream_request(
    request_proto: LiveChatMessageListRequest,
    bearer: MetadataValue<Ascii>,
) -> tonic::Request<LiveChatMessageListRequest> {
    let mut request = tonic::Request::new(request_proto);
    request
        .metadata_mut()
        .insert(AUTHORIZATION_METADATA_KEY, bearer);
    request
}

pub(super) fn stream_endpoint() -> Result<tonic::transport::Endpoint, LiveApiError> {
    let endpoint = tonic::transport::Endpoint::from_shared(STREAM_ENDPOINT_URL.to_string())
        .map_err(|_| endpoint_build_failure())?;
    let endpoint = endpoint.connect_timeout(Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS));
    endpoint
        .tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
        .map_err(|_| endpoint_build_failure())
}

fn endpoint_build_failure() -> LiveApiError {
    LiveApiError::Network {
        reason: STREAM_ENDPOINT_BUILD_REASON.to_string(),
    }
}

pub(crate) struct TonicChatStreamSession {
    inner: tonic::Streaming<LiveChatMessageListResponse>,
}

impl TonicChatStreamSession {
    pub(crate) fn new(inner: tonic::Streaming<LiveChatMessageListResponse>) -> Self {
        Self { inner }
    }
}

#[tonic::async_trait]
impl LiveChatStreamSession for TonicChatStreamSession {
    async fn next_batch(&mut self) -> Option<Result<YieldedChatStreamBatch, LiveApiError>> {
        match self.inner.message().await {
            Ok(Some(response)) => Some(Ok(map_stream_batch(&response))),
            Err(status) => Some(Err(classify_stream_status(&status))),
            Ok(None) => None, // clean server close = the seam's None (record §A.6)
        }
    }
}

#[cfg(test)]
#[path = "stream_client_tests.rs"]
mod stream_client_tests;

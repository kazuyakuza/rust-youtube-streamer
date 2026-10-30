//! The concrete `ChatStreamTransport` over the tonic adapter (record
//! P7-12/A.3): a FRESH `Endpoint` per open (the recorded channel-reuse
//! alternative rejected — each open is a new quota-billed call and a shared
//! lazy channel would void the bounded 10 s establishment), the tokio
//! timeout around the connect future, the generic-client server-streaming
//! call, and per-status classification. Errors are exactly the §A.5 table.
//! The establishment shape is pinned offline against a dead plaintext loopback endpoint (the readiness reserve anchor and fix-shape pins); the real-Google TLS/handshake behavior remains the operator manual E2E (record P7-12 limitation, amended 2026-10-30).

use std::time::Duration;

use tonic::codegen::http::uri::PathAndQuery;

use super::live_error::{classify_stream_status, LiveApiError};
use super::stream_client::{
    bearer_token_metadata, stream_endpoint, stream_request, TonicChatStreamSession,
    CHAT_STREAM_CONNECT_TIMEOUT_MS, STREAM_CONNECT_FAILURE_REASON, STREAM_CONNECT_TIMEOUT_REASON,
    STREAM_LIST_PATH, STREAM_READY_FAILURE_REASON,
};
use super::stream_proto::stream_list_request;
use super::stream_transport::{ChatStreamRequest, ChatStreamTransport, LiveChatStreamSession};
use super::token::AccessToken;

pub(crate) struct GoogleLiveChatStreamTransport {
    access_token: AccessToken,
}

impl GoogleLiveChatStreamTransport {
    pub(crate) fn new(access_token: AccessToken) -> Self {
        Self { access_token }
    }

    /// The one establishment composition (the test-constructed endpoint
    /// pins it offline): bounded connect → readiness reserve → the
    /// server-streaming call.
    async fn open_with_endpoint(
        &mut self,
        endpoint: tonic::transport::Endpoint,
        command: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        let bearer = bearer_token_metadata(&self.access_token)?;
        let channel = tokio::time::timeout(
            Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS),
            endpoint.connect(),
        )
        .await
        .map_err(|_| stream_connect_timeout_error())?
        .map_err(|_| stream_connect_failure_error())?;
        let mut grpc = tonic::client::Grpc::new(channel);
        grpc.ready()
            .await
            .map_err(|_| stream_ready_failure_error())?;
        let response = grpc
            .server_streaming(
                stream_request(stream_list_request(&command), bearer),
                PathAndQuery::from_static(STREAM_LIST_PATH),
                tonic::codec::ProstCodec::default(),
            )
            .await
            .map_err(|status| classify_stream_status(&status))?;
        Ok(Box::new(TonicChatStreamSession::new(response.into_inner())))
    }
}

fn stream_connect_timeout_error() -> LiveApiError {
    LiveApiError::Network {
        reason: STREAM_CONNECT_TIMEOUT_REASON.to_string(),
    }
}

fn stream_connect_failure_error() -> LiveApiError {
    LiveApiError::Network {
        reason: STREAM_CONNECT_FAILURE_REASON.to_string(),
    }
}

fn stream_ready_failure_error() -> LiveApiError {
    LiveApiError::Network {
        reason: STREAM_READY_FAILURE_REASON.to_string(),
    }
}

#[tonic::async_trait]
impl ChatStreamTransport for GoogleLiveChatStreamTransport {
    async fn open(
        &mut self,
        command: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        let endpoint = stream_endpoint()?;
        self.open_with_endpoint(endpoint, command).await
    }
}

#[cfg(test)]
#[path = "stream_concrete_tests.rs"]
mod stream_concrete_tests;

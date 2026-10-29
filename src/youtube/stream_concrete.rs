//! The concrete `ChatStreamTransport` over the tonic adapter (record
//! P7-12/A.3): a FRESH `Endpoint` per open (the recorded channel-reuse
//! alternative rejected — each open is a new quota-billed call and a shared
//! lazy channel would void the bounded 10 s establishment), the tokio
//! timeout around the connect future, the generic-client server-streaming
//! call, and per-status classification. Errors are exactly the §A.5 table.
//! The handshake stays untested offline; the operator manual E2E is the
//! only real-Google validation (record P7-12 limitation).

use std::time::Duration;

use tonic::codegen::http::uri::PathAndQuery;

use super::live_error::{classify_stream_status, LiveApiError};
use super::stream_client::{
    bearer_token_metadata, stream_endpoint, stream_request, TonicChatStreamSession,
    CHAT_STREAM_CONNECT_TIMEOUT_MS, STREAM_CONNECT_FAILURE_REASON, STREAM_CONNECT_TIMEOUT_REASON,
    STREAM_LIST_PATH,
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

#[tonic::async_trait]
impl ChatStreamTransport for GoogleLiveChatStreamTransport {
    async fn open(
        &mut self,
        command: ChatStreamRequest,
    ) -> Result<Box<dyn LiveChatStreamSession>, LiveApiError> {
        let endpoint = stream_endpoint()?;
        let bearer = bearer_token_metadata(&self.access_token)?;
        let channel = tokio::time::timeout(
            Duration::from_millis(CHAT_STREAM_CONNECT_TIMEOUT_MS),
            endpoint.connect(),
        )
        .await
        .map_err(|_| stream_connect_timeout_error())?
        .map_err(|_| stream_connect_failure_error())?;
        let mut grpc = tonic::client::Grpc::new(channel);
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

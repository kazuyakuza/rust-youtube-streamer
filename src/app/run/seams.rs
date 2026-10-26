//! The testability seams of the run runtime (plan D-T3-9; split from
//! `mod.rs` per the D-T3-14 line cap): the parameter objects that keep
//! every startup collaborator injectable within the ≤2-args rule.

use std::sync::Arc;

use crate::app::pipeline::StopFlag;
use crate::config::AppConfig;
use crate::youtube::{
    AccessToken, AuthorizationVerifier, CredentialStore, LiveApiTransport, LiveChatTransport,
    TokenEndpointTransport, WaitCanceller,
};

/// The Ctrl+C pair handed to the startup (one handler, both flags).
pub(super) struct RunStops {
    pub(super) stop_flag: StopFlag,
    pub(super) canceller: WaitCanceller,
}

/// Stage 2's four injected service seams (D-T3-9).
pub(super) struct ServiceStops<'a> {
    pub(super) client_config: &'a crate::youtube::ClientConfig,
    pub(super) store: &'a dyn CredentialStore,
    pub(super) endpoint: &'a dyn TokenEndpointTransport,
    pub(super) verifier: &'a dyn AuthorizationVerifier,
}

/// The shared transport handle type: `Arc` so the confirmation gate
/// thread can hold a clone after `prepare` (the ONE sanctioned
/// `Send + Sync` wrapper form; the compiler confirmed the concrete ureq
/// transport satisfies it directly — no Mutex wrapper was needed).
pub(super) type BroadcastTransport = Arc<dyn LiveApiTransport + Send + Sync>;

/// The chat-side shared transport handle (same Send+Sync wrapper form).
pub(super) type ChatTransport = Arc<dyn LiveChatTransport + Send + Sync>;

/// The transport factory's result: BOTH handles coerce from the ONE
/// concrete `Arc<GoogleLiveApiTransport>` (one token, one instance).
pub(super) struct BroadcastTransports {
    pub(super) live: BroadcastTransport,
    pub(super) chat: ChatTransport,
}

/// Tests substitute the fakes here; production builds the concrete ureq
/// transport from the verified access token (both handles from one Arc).
pub(super) type TransportFactory<'a> = Box<dyn FnOnce(AccessToken) -> BroadcastTransports + 'a>;

/// The whole startup after the Ctrl+C pair install, parameterized for
/// offline testing (≤2-args rule via parameter object).
pub(super) struct RuntimeStart<'a> {
    pub(super) config: &'a AppConfig,
    pub(super) stops: RunStops,
    pub(super) service: ServiceStops<'a>,
    pub(super) build_transport: TransportFactory<'a>,
}

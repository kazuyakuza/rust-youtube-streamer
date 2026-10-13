//! Typed failures for the credential service boundary: one enum for the
//! load/refresh/verify pipeline, with one operator-facing `Display` line
//! per failure class. Payloads carry only non-secret material (io causes,
//! paths, Google error code strings, inner redacted enums) — never token
//! values or response bodies. Class mapping and Google semantics:
//! `.kilo/plans/20261012-phase04-task4-credential-service.md` D3/D5.

use std::error::Error;
use std::fmt;

use super::error::{CredentialStoreError, TokenEndpointError, VerificationError};

/// Every distinct failure class of [`super::credential_service`].
/// Exit-code/actionable-class ownership happens in the future run wiring;
/// this type defines the classes only.
#[derive(Debug)]
pub(crate) enum CredentialServiceError {
    /// No stored credentials exist; the operator must run the auth
    /// command first.
    Absent,
    /// Stored credentials exist but could not be read or parsed
    /// (the `FileOpen`/`ReadParse` store classes).
    Unreadable(CredentialStoreError),
    /// The refresh succeeded but persisting the refreshed data failed.
    Persistence(CredentialStoreError),
    /// The refresh attempt failed at the token endpoint (transport
    /// failure, unparsable response, or a server code other than
    /// invalid_grant) — the transient/recoverable failure class.
    Refresh(TokenEndpointError),
    /// The refresh endpoint answered with EXACTLY the `invalid_grant`
    /// code (Google's documented expired/invalidated/revoked signature);
    /// any other server code stays in the transient `Refresh` class.
    /// Only interactive re-authorization fixes this.
    Revoked { code: String },
    /// Stored credentials carry no refresh token, so a rejected access
    /// token cannot be refreshed; re-authorization is required.
    Unrefreshable,
    /// Verification of the effective authorization failed (insufficient
    /// scope, unauthorized-after-refresh, or transport); the inner error
    /// names the exact class.
    Verification(VerificationError),
}

impl fmt::Display for CredentialServiceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CredentialServiceError::Absent => formatter.write_str(
                "no stored credentials are available; run the authorization command first",
            ),
            CredentialServiceError::Unreadable(error) => write!(formatter, "{error}"),
            CredentialServiceError::Persistence(error) => write!(formatter, "{error}"),
            CredentialServiceError::Refresh(error) => {
                write!(formatter, "credential refresh failed: {error}")
            }
            CredentialServiceError::Revoked { code } => write!(
                formatter,
                "the stored refresh token is no longer valid (Google error code '{code}'); \
run the authorization command again to obtain new credentials"
            ),
            CredentialServiceError::Unrefreshable => formatter.write_str(
                "the stored credentials cannot be refreshed because no refresh token is \
stored; run the authorization command again",
            ),
            CredentialServiceError::Verification(error) => write!(formatter, "{error}"),
        }
    }
}

impl Error for CredentialServiceError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CredentialServiceError::Unreadable(inner) => inner.source(),
            CredentialServiceError::Persistence(inner) => inner.source(),
            _ => None,
        }
    }
}

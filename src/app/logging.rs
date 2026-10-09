//! Structured logging initialization for the startup pipeline. It is called
//! exactly once per process, before any configuration load or mode handler
//! runs; the binding is the process's one logging boundary. Call sites depend
//! only on this function's signature, never on logger internals, which keeps
//! logging machinery out of the pipeline and the mode handlers.
//!
//! The log filter is read from `RUST_LOG` once at startup: unset, empty, or
//! whitespace-only values resolve to [`DEFAULT_FILTER`]; a malformed
//! directive is a fatal [`LogInitError`], never a silent substitution.
//! Application logs go to standard error.
//!
//! Secrets rule: log events may only carry the whitelisted fields `command`
//! (`auth`/`run`), `config_path` (operator-supplied path text), and
//! `exit_code` (numeric constant), plus free-text messages with no field
//! values appended. Forbidden forever: secret or credential values, stream
//! keys, ingestion URLs, configuration field values of any kind, and the
//! full contents of the configuration file.

use super::error::LogInitError;
use tracing_subscriber::EnvFilter;

/// Filter directive applied when `RUST_LOG` is unset, empty, or
/// whitespace-only.
pub(super) const DEFAULT_FILTER: &str = "info";

const RUST_LOG: &str = "RUST_LOG";

/// Initializes structured application logging once.
///
/// Reads `RUST_LOG` once: unset, empty, or whitespace-only resolves to
/// [`DEFAULT_FILTER`]; a malformed directive fails fatally with
/// [`LogInitError::FilterInvalid`]. Output goes to standard error.
pub(super) fn init_logging() -> Result<(), LogInitError> {
    let raw = std::env::var(RUST_LOG).ok();
    let directive = resolved_directive(raw.as_deref())?;
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&directive))
        .with_writer(std::io::stderr)
        .finish();
    let install = tracing::subscriber::set_global_default(subscriber);
    install.map_err(|error| LogInitError::SubscriberInstall {
        reason: error.to_string(),
    })
}

/// Resolves the raw `RUST_LOG` value into the directive compiled into the
/// filter: unset, empty, or whitespace-only falls back to
/// [`DEFAULT_FILTER`]; anything else is trimmed and validated. Pure seam:
/// no environment access, so tests inject values directly.
fn resolved_directive(raw: Option<&str>) -> Result<String, LogInitError> {
    let value = raw.unwrap_or_default().trim();
    if value.is_empty() {
        return Ok(DEFAULT_FILTER.to_string());
    }
    validate(value)?;
    Ok(value.to_string())
}

/// Rejects a malformed filter directive as fatal instead of silently
/// substituting the default.
fn validate(directive: &str) -> Result<(), LogInitError> {
    EnvFilter::try_new(directive)
        .map(|_| ())
        .map_err(|_| LogInitError::FilterInvalid {
            directive: directive.to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filter_invalid_directive(result: &Result<String, LogInitError>) -> String {
        match result {
            Err(LogInitError::FilterInvalid { directive }) => directive.clone(),
            other => panic!("expected filter-invalid error, got {other:?}"),
        }
    }

    #[test]
    fn default_directive_constant_is_info() {
        assert_eq!(DEFAULT_FILTER, "info");
        assert_eq!(resolved_directive(None).unwrap(), "info");
    }

    #[test]
    fn unset_env_resolves_to_default_filter() {
        assert_eq!(resolved_directive(None).unwrap(), DEFAULT_FILTER);
    }

    #[test]
    fn empty_and_whitespace_env_resolves_to_default_filter() {
        assert_eq!(resolved_directive(Some("")).unwrap(), DEFAULT_FILTER);
        assert_eq!(resolved_directive(Some("  \t  ")).unwrap(), DEFAULT_FILTER);
        assert_eq!(resolved_directive(Some("\n")).unwrap(), DEFAULT_FILTER);
    }

    #[test]
    fn valid_level_directive_is_kept_verbatim() {
        assert_eq!(resolved_directive(Some("debug")).unwrap(), "debug");
    }

    #[test]
    fn combined_directive_is_kept_verbatim() {
        assert_eq!(
            resolved_directive(Some("trace,app=off")).unwrap(),
            "trace,app=off"
        );
    }

    #[test]
    fn whitespace_around_directive_is_trimmed() {
        assert_eq!(resolved_directive(Some(" debug ")).unwrap(), "debug");
    }

    #[test]
    fn malformed_directive_is_rejected_as_filter_invalid() {
        let result = resolved_directive(Some("app=banana"));
        assert_eq!(filter_invalid_directive(&result), "app=banana");
    }
}

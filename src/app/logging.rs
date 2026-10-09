//! Structured logging initialization for the startup pipeline. It is called
//! exactly once per process, before any configuration load or mode handler
//! runs; the binding is the process's one logging boundary. Call sites depend
//! only on this function's signature, never on logger internals, which keeps
//! logging machinery out of the pipeline and the mode handlers. The init-once
//! invariant is that single call site plus `set_global_default`, which refuses
//! a second subscriber and reports the clash as `SubscriberInstall` instead of
//! silently dropping logs.
//!
//! The stack is `tracing-subscriber`'s default `fmt` layout, writing to
//! standard error, with an `env-filter` (`EnvFilter`) supplying the level
//! filter. The directive comes from the environment variable named exactly
//! `RUST_LOG` (the name is case-sensitive), read once at startup and resolved
//! in order: unset or a non-UTF-8 value falls back to [`DEFAULT_FILTER`]
//! silently; an empty or whitespace-only value does the same; a directive that
//! fails to compile under `EnvFilter` emits exactly one `warning:` line on
//! standard error, then falls back to [`DEFAULT_FILTER`]; a valid directive is
//! used verbatim after trimming surrounding whitespace.
//!
//! [`DEFAULT_FILTER`] is `info`. Operator guidance for `RUST_LOG` lives in the
//! project README.
//!
//! Secrets rule: log events may only carry the whitelisted fields `command`
//! (`auth`/`run`), `config_path` (operator-supplied path text), and
//! `exit_code` (numeric constant), plus free-text messages with no field
//! values appended. Forbidden forever: secret or credential values, stream
//! keys, ingestion URLs, configuration field values of any kind, and the
//! full contents of the configuration file.

use super::error::LogInitError;
use tracing_subscriber::EnvFilter;

/// Filter directive applied when `RUST_LOG` is unset (including a
/// non-UTF-8 value), empty, whitespace-only, or malformed. The value is
/// `info`.
pub(super) const DEFAULT_FILTER: &str = "info";

const RUST_LOG: &str = "RUST_LOG";

/// Initializes structured application logging once at startup.
///
/// Resolves the `RUST_LOG` directive as described in the module header,
/// builds a `tracing-subscriber` `fmt` layer writing to standard error with an
/// `EnvFilter`, and installs it as the global default subscriber. A failed
/// installation returns [`LogInitError::SubscriberInstall`]; no other outcome
/// fails startup.
pub(super) fn init_logging() -> Result<(), LogInitError> {
    let raw = std::env::var(RUST_LOG).ok();
    warn_about_invalid_directive(raw.as_deref());
    let directive = resolved_directive(raw.as_deref());
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&directive))
        .with_writer(std::io::stderr)
        .finish();
    let install = tracing::subscriber::set_global_default(subscriber);
    install.map_err(|error| LogInitError::SubscriberInstall {
        reason: error.to_string(),
    })
}

/// Emits the single recovery warning when a non-empty `RUST_LOG` value is a
/// malformed directive that falls back to [`DEFAULT_FILTER`].
fn warn_about_invalid_directive(raw: Option<&str>) {
    let value = raw.unwrap_or_default().trim();
    if value.is_empty() {
        return;
    }
    if is_directive_supported(value) {
        return;
    }
    eprintln!("warning: invalid RUST_LOG value '{value}'; using default filter");
}

/// Resolves the raw `RUST_LOG` value into the directive compiled into the
/// filter: unset, empty, whitespace-only, or malformed values fall back to
/// [`DEFAULT_FILTER`]; anything else is trimmed and kept. Pure seam: no
/// environment access, so tests inject values directly.
fn resolved_directive(raw: Option<&str>) -> String {
    let value = raw.unwrap_or_default().trim();
    if value.is_empty() {
        return DEFAULT_FILTER.to_string();
    }
    if !is_directive_supported(value) {
        return DEFAULT_FILTER.to_string();
    }
    value.to_string()
}

/// Reports whether a filter directive compiles under `EnvFilter` semantics.
fn is_directive_supported(raw: &str) -> bool {
    EnvFilter::try_new(raw).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_directive_constant_is_info() {
        assert_eq!(DEFAULT_FILTER, "info");
        assert_eq!(resolved_directive(None), "info");
    }

    #[test]
    fn unset_env_resolves_to_default_filter() {
        assert_eq!(resolved_directive(None), DEFAULT_FILTER);
    }

    #[test]
    fn empty_and_whitespace_env_resolves_to_default_filter() {
        assert_eq!(resolved_directive(Some("")), DEFAULT_FILTER);
        assert_eq!(resolved_directive(Some("  \t  ")), DEFAULT_FILTER);
        assert_eq!(resolved_directive(Some("\n")), DEFAULT_FILTER);
    }

    #[test]
    fn valid_level_directive_is_kept_verbatim() {
        assert_eq!(resolved_directive(Some("debug")), "debug");
    }

    #[test]
    fn combined_directive_is_kept_verbatim() {
        assert_eq!(resolved_directive(Some("trace,app=off")), "trace,app=off");
    }

    #[test]
    fn whitespace_around_directive_is_trimmed() {
        assert_eq!(resolved_directive(Some(" debug ")), "debug");
    }

    #[test]
    fn malformed_directive_falls_back_to_default_filter() {
        assert_eq!(resolved_directive(Some("app=banana")), DEFAULT_FILTER);
    }
}

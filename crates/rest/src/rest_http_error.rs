use crate::RestResponseHeaders;
use http::StatusCode;
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

/// A REST exchange returned an unsuccessful HTTP status.
///
/// Typed receive methods preserve this error through caller context and response
/// artifact diagnostics. Inspect [`Self::status`] instead of parsing its message;
/// transport and response-decoding failures are different errors.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RestHttpError {
    pub status: StatusCode,
    pub reason_phrase: Option<String>,
    /// Response metadata used by callers for throttling and retry decisions.
    pub headers: RestResponseHeaders,
}

impl Display for RestHttpError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "REST call failed with status {}: {}",
            self.status.as_u16(),
            self.reason_phrase.as_deref().unwrap_or("Unknown error")
        )
    }
}

impl Error for RestHttpError {}

use cloud_terrastodon_auth_types::AzureCredentialResource;
use http::StatusCode;
use std::fmt;
use uuid::Uuid;

/// Safe diagnostics from a rejected Microsoft Entra browser token exchange.
///
/// Retains numeric AADSTS codes and validated request identifiers for diagnosis,
/// without retaining tokens, response bodies, or service-provided descriptions.
/// See [the token endpoint error response](https://learn.microsoft.com/en-us/entra/identity-platform/v2-oauth2-auth-code-flow#error-response-1)
/// and [the AADSTS error reference](https://learn.microsoft.com/en-us/entra/identity-platform/reference-error-codes).
#[derive(Debug)]
pub struct EntraTokenRequestError {
    pub resource: AzureCredentialResource,
    pub status: StatusCode,
    /// Known OAuth error vocabulary; other service strings become `unrecognized_error`.
    pub oauth_error: &'static str,
    pub error_codes: Vec<u64>,
    pub trace_id: Option<Uuid>,
    pub correlation_id: Option<Uuid>,
}

impl fmt::Display for EntraTokenRequestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Entra browser token request for {:?} failed with {} ({})",
            self.resource, self.status, self.oauth_error,
        )?;
        for code in &self.error_codes {
            write!(formatter, "; AADSTS{code}")?;
        }
        if let Some(trace_id) = self.trace_id {
            write!(formatter, "; trace ID: {trace_id}")?;
        }
        if let Some(correlation_id) = self.correlation_id {
            write!(formatter, "; correlation ID: {correlation_id}")?;
        }
        Ok(())
    }
}

impl std::error::Error for EntraTokenRequestError {}

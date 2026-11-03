use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use http::HeaderValue;
use std::fmt;

/// A sensitive HTTP Bearer authorization header constructed from a nonempty token.
///
/// See [Azure DevOps bearer authentication](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/entra-oauth?view=azure-devops).
/// Credential bytes are transport data only; this type has no serialization or
/// registry implementation.
#[derive(Clone)]
pub struct BearerAuthorization {
    // Private storage preserves the Bearer scheme and sensitive flag. Neither
    // arbitrary header construction nor mutable access may bypass those invariants.
    header: HeaderValue,
}

impl BearerAuthorization {
    /// Build a sensitive bearer header without parsing or logging token claims.
    /// Empty credentials are rejected as a library constraint.
    pub fn new(token: &str) -> Result<Self> {
        ensure!(!token.is_empty(), "bearer credential must not be empty");
        let mut header = HeaderValue::from_str(&format!("Bearer {token}"))
            .wrap_err("bearer credential cannot be represented as an authorization header")?;
        header.set_sensitive(true);
        Ok(Self { header })
    }

    /// Borrow the sensitive header for transmission; do not log or persist its bytes.
    pub fn header(&self) -> &HeaderValue {
        &self.header
    }
}

impl fmt::Debug for BearerAuthorization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("BearerAuthorization([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_credentials_and_injection_without_exposing_input() {
        let synthetic = "synthetic-secret\r\nInjected: value";
        let error = BearerAuthorization::new(synthetic).unwrap_err();
        assert!(!format!("{error:#}").contains(synthetic));
        assert!(BearerAuthorization::new("").is_err());
    }
}

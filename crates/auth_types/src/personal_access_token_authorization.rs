use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use http::HeaderValue;
use std::fmt;

/// A sensitive PAT HTTP Basic authorization header with an empty username.
///
/// See [Azure DevOps PAT authentication](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops#use-a-pat).
/// Credential bytes are transport data only; this type has no serialization or
/// registry implementation.
#[derive(Clone)]
pub struct PersonalAccessTokenAuthorization {
    // Private storage preserves Basic encoding, the empty username, and the
    // sensitive flag; arbitrary header construction or mutation could break them.
    header: HeaderValue,
}

impl PersonalAccessTokenAuthorization {
    /// Encode a PAT as HTTP Basic authentication with an empty username.
    /// Empty credentials are rejected as a library constraint.
    pub fn new(token: &str) -> Result<Self> {
        ensure!(!token.is_empty(), "personal access token must not be empty");
        let mut header =
            HeaderValue::from_str(&format!("Basic {}", STANDARD.encode(format!(":{token}"))))
                .wrap_err(
                    "personal access token cannot be represented as an authorization header",
                )?;
        header.set_sensitive(true);
        Ok(Self { header })
    }

    /// Borrow the sensitive header for transmission; do not log or persist its bytes.
    pub fn header(&self) -> &HeaderValue {
        &self.header
    }
}

impl fmt::Debug for PersonalAccessTokenAuthorization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PersonalAccessTokenAuthorization([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_an_empty_username_and_rejects_empty_tokens() -> Result<()> {
        assert!(PersonalAccessTokenAuthorization::new("").is_err());
        let authorization = PersonalAccessTokenAuthorization::new("synthetic-token")?;
        let encoded = authorization
            .header()
            .to_str()?
            .strip_prefix("Basic ")
            .unwrap();
        assert_eq!(STANDARD.decode(encoded)?, b":synthetic-token");
        Ok(())
    }
}

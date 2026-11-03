use crate::BearerAuthorization;
use crate::PersonalAccessTokenAuthorization;
use eyre::Result;
use http::HeaderValue;

/// Sensitive authorization acquired for one execution.
///
/// The header is marked sensitive and diagnostics are redacted. This type has
/// no serialization or registry implementation: credential bytes are not request
/// data, artifact inputs, or cache identity. Its borrowed header is exposed only
/// for the transport boundary; callers must not log or persist its bytes.
/// See [Azure DevOps bearer authentication](https://learn.microsoft.com/en-us/azure/devops/integrate/get-started/authentication/entra-oauth?view=azure-devops)
/// and [PAT HTTP Basic authentication](https://learn.microsoft.com/en-us/azure/devops/organizations/accounts/use-personal-access-tokens-to-authenticate?view=azure-devops#use-a-pat).
#[derive(Clone, Debug)]
pub enum Authorization {
    /// Execute without an authorization header or credential lookup.
    Anonymous,
    /// A validated, sensitive HTTP Bearer header.
    Bearer(BearerAuthorization),
    /// A validated, sensitive PAT HTTP Basic header with an empty username.
    PersonalAccessToken(PersonalAccessTokenAuthorization),
}

impl Authorization {
    /// Build a sensitive bearer header without parsing or logging token claims.
    /// Empty credentials are rejected as a library constraint.
    pub fn bearer(token: &str) -> Result<Self> {
        Ok(Self::Bearer(BearerAuthorization::new(token)?))
    }

    /// Encode a PAT as HTTP Basic authentication with an empty username.
    /// Empty credentials are rejected as a library constraint.
    pub fn personal_access_token(token: &str) -> Result<Self> {
        Ok(Self::PersonalAccessToken(
            PersonalAccessTokenAuthorization::new(token)?,
        ))
    }

    /// Borrow the sensitive header for transmission; anonymous execution returns None.
    pub fn header(&self) -> Option<&HeaderValue> {
        match self {
            Self::Anonymous => None,
            Self::Bearer(authorization) => Some(authorization.header()),
            Self::PersonalAccessToken(authorization) => Some(authorization.header()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;

    #[test]
    fn credential_diagnostics_and_cloned_headers_remain_redacted() -> Result<()> {
        let synthetic = "synthetic-credential-for-redaction";
        let encoded = STANDARD.encode(format!(":{synthetic}"));
        for authorization in [
            Authorization::bearer(synthetic)?,
            Authorization::personal_access_token(synthetic)?,
        ] {
            let cloned = authorization.clone();
            let header = cloned.header().unwrap();
            assert!(header.is_sensitive());
            for diagnostic in [
                format!("{authorization:?}"),
                format!("{cloned:#?}"),
                format!("{header:?}"),
            ] {
                assert!(!diagnostic.contains(synthetic));
                assert!(!diagnostic.contains(&encoded));
            }
        }
        assert!(Authorization::Anonymous.header().is_none());
        Ok(())
    }
}

use eyre::Result;
use eyre::ensure;

/// Sensitive [ACR access-token response](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/authentication/exchange-acr-refresh-token-for-acr-access-token?view=rest-registry-dataplane-2021-07-01#acraccesstoken).
#[derive(facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub(crate) struct ContainerRegistryAccessTokenResponse {
    #[facet(sensitive)]
    pub access_token: String,
}

impl ContainerRegistryAccessTokenResponse {
    pub fn decode(body: &str) -> Result<Self> {
        // Preserve the credential boundary instead of including a decoder's token-bearing excerpt.
        let value: Self = facet_json::from_str(body).map_err(|_| {
            eyre::eyre!("invalid ACR access-token response; sensitive body omitted")
        })?;
        ensure!(
            !value.access_token.is_empty(),
            "ACR returned an empty access token"
        );
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::container_registry_refresh_token_response::ContainerRegistryRefreshTokenResponse;

    #[test]
    fn malformed_token_responses_do_not_quote_secret_text_in_errors() {
        let secret = "synthetic-credential-never-log";
        let malformed = format!("{{\"access_token\":\"{secret}\"");
        let access = ContainerRegistryAccessTokenResponse::decode(&malformed)
            .err()
            .unwrap();
        let refresh = ContainerRegistryRefreshTokenResponse::decode(&malformed)
            .err()
            .unwrap();
        assert!(!format!("{access:?}").contains(secret));
        assert!(!format!("{refresh:?}").contains(secret));
    }
}

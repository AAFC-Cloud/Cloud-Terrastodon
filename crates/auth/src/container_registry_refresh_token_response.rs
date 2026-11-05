use eyre::Result;
use eyre::ensure;

/// Sensitive [ACR refresh-token response](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/authentication/exchange-aad-access-token-for-acr-refresh-token?view=rest-registry-dataplane-2021-07-01#acrrefreshtoken).
#[derive(facet::Facet)]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub(crate) struct ContainerRegistryRefreshTokenResponse {
    #[facet(sensitive)]
    pub refresh_token: String,
}
impl ContainerRegistryRefreshTokenResponse {
    pub fn decode(body: &str) -> Result<Self> {
        // Facet's parse diagnostic can quote the response: never retain token text in an error.
        let value: Self = facet_json::from_str(body).map_err(|_| {
            eyre::eyre!("invalid ACR refresh-token response; sensitive body omitted")
        })?;
        ensure!(
            !value.refresh_token.is_empty(),
            "ACR returned an empty refresh token"
        );
        Ok(value)
    }
}

use crate::KeyVaultSecret;
use url::Url;

/// [SecretListResult, API 2025-07-01](https://learn.microsoft.com/en-us/rest/api/keyvault/secrets/get-secrets/get-secrets?view=rest-keyvault-secrets-2025-07-01#secretlistresult).
#[derive(Debug, facet::Facet)]
#[facet(rename_all = "camelCase")]
#[cfg_attr(debug_assertions, facet(deny_unknown_fields))]
pub struct KeyVaultSecretListResponse {
    pub value: Vec<KeyVaultSecret>,
    pub next_link: Option<Url>,
}

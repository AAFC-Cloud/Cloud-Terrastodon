use crate::AuthContext;
use crate::container_registry_access_token_response::ContainerRegistryAccessTokenResponse;
use crate::container_registry_refresh_token_response::ContainerRegistryRefreshTokenResponse;
use crate::fetch_azure_bearer_access_token;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ContainerRegistryLoginServer;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use eyre::ensure;
use reqwest::Client;
use reqwest::Url;
use std::collections::HashMap;
use std::time::Duration;

/// A registry bearer challenge bound to the contacted registry, never a caller-supplied token endpoint.
/// [ACR authentication exchange](https://learn.microsoft.com/en-us/rest/api/registry-dataplane/authentication/exchange-aad-access-token-for-acr-refresh-token?view=rest-registry-dataplane-2021-07-01).
pub struct ContainerRegistryBearerChallenge {
    // Private fields keep the token endpoint and audience bound to one validated server.
    server: ContainerRegistryLoginServer,
    scope: String,
}

impl ContainerRegistryBearerChallenge {
    pub fn parse(endpoint: &Url, challenge: &str) -> Result<Self> {
        ensure!(
            endpoint.scheme() == "https"
                && endpoint.username().is_empty()
                && endpoint.password().is_none()
                && endpoint.port().is_none(),
            "registry authentication requires an HTTPS registry endpoint"
        );
        let server: ContainerRegistryLoginServer = endpoint
            .host_str()
            .ok_or_else(|| eyre::eyre!("registry endpoint has no hostname"))?
            .parse()?;
        let (scheme, fields) = challenge
            .split_once(' ')
            .ok_or_else(|| eyre::eyre!("registry returned a malformed authentication challenge"))?;
        ensure!(
            scheme.eq_ignore_ascii_case("Bearer"),
            "registry authentication requires a bearer challenge"
        );
        // Quoted commas occur in repository action lists, so split only outside quotes.
        let mut quoted = false;
        let mut start = 0;
        let mut parts = Vec::new();
        for (index, ch) in fields.char_indices() {
            match ch {
                '"' => quoted = !quoted,
                ',' if !quoted => {
                    parts.push(&fields[start..index]);
                    start = index + 1;
                }
                '\\' => {
                    eyre::bail!("escaped registry challenge parameters are unsupported");
                }
                _ => {}
            }
        }
        ensure!(
            !quoted,
            "registry challenge contains an unterminated quoted value"
        );
        parts.push(&fields[start..]);
        let mut values = HashMap::new();
        for part in parts {
            let (key, value) = part
                .trim()
                .split_once('=')
                .ok_or_else(|| eyre::eyre!("registry challenge parameter has no value"))?;
            let value = value.trim();
            let value = value
                .strip_prefix('"')
                .and_then(|value| value.strip_suffix('"'))
                .ok_or_else(|| eyre::eyre!("registry challenge parameter must be quoted"))?;
            ensure!(
                values
                    .insert(key.trim().to_ascii_lowercase(), value)
                    .is_none(),
                "registry challenge contains a duplicate parameter"
            );
        }
        ensure!(
            values
                .get("service")
                .is_some_and(|value| value.eq_ignore_ascii_case(server.as_str())),
            "registry challenge service differs from the selected registry"
        );
        let realm = Url::parse(
            values
                .get("realm")
                .ok_or_else(|| eyre::eyre!("registry challenge has no realm"))?,
        )?;
        ensure!(
            realm.origin() == endpoint.origin()
                && realm.path() == "/oauth2/token"
                && realm.username().is_empty()
                && realm.password().is_none()
                && realm.query().is_none()
                && realm.fragment().is_none(),
            "registry challenge token endpoint differs from the selected registry"
        );
        let scope = values
            .get("scope")
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| eyre::eyre!("registry challenge has no access scope"))?
            .to_string();
        Ok(Self { server, scope })
    }

    /// Exchange Entra -> ACR refresh -> scoped ACR access token without execution artifacts.
    /// Forms follow the [official SDK implementation](https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/containerregistry/azure-containerregistry/azure/containerregistry/_generated/operations/_patch.py).
    /// Redirects are disabled so credential forms cannot leave the bound registry.
    pub async fn authorize(
        &self,
        auth_context: &AuthContext,
        tenant: AzureTenantId,
        cancellation: &CancellationScope,
    ) -> Result<Authorization> {
        cancellation
            .cancellable_scope(|scope| async move {
                let token = fetch_azure_bearer_access_token(
                    auth_context,
                    tenant,
                    AzureCredentialResource::AzureContainerRegistry,
                    &scope,
                )
                .await?;
                let client = Client::builder()
                    .https_only(true)
                    .redirect(reqwest::redirect::Policy::none())
                    .connect_timeout(Duration::from_secs(30))
                    .timeout(Duration::from_secs(120))
                    .build()?;
                let origin = self.server.endpoint()?;
                let response = client
                    .post(origin.join("/oauth2/exchange?api-version=2021-07-01")?)
                    .form(&[
                        ("grant_type", "access_token"),
                        ("service", self.server.as_str()),
                        ("tenant", tenant.to_string().as_str()),
                        ("access_token", token.access_token.as_str()),
                    ])
                    .send()
                    .await?
                    .error_for_status()?;
                let refresh =
                    ContainerRegistryRefreshTokenResponse::decode(&response.text().await?)?;
                scope.bail_if_cancelled()?;
                let response = client
                    .post(origin.join("/oauth2/token?api-version=2021-07-01")?)
                    .form(&[
                        ("grant_type", "refresh_token"),
                        ("service", self.server.as_str()),
                        ("scope", self.scope.as_str()),
                        ("refresh_token", refresh.refresh_token.as_str()),
                    ])
                    .send()
                    .await?
                    .error_for_status()?;
                let access = ContainerRegistryAccessTokenResponse::decode(&response.text().await?)?;
                scope.bail_if_cancelled()?;
                Authorization::bearer(&access.access_token)
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn binds_challenge_credentials_to_registry_and_preserves_action_lists() {
        let url = Url::parse("https://fixture.azurecr.io/acr/v1/sample/_tags").unwrap();
        let challenge = r#"Bearer realm="https://fixture.azurecr.io/oauth2/token",service="fixture.azurecr.io",scope="repository:sample:metadata_read,pull""#;
        let parsed = ContainerRegistryBearerChallenge::parse(&url, challenge).unwrap();
        assert_eq!(parsed.scope, "repository:sample:metadata_read,pull");
        for altered in [
            challenge.replace(
                "realm=\"https://fixture.azurecr.io",
                "realm=\"https://other.azurecr.io",
            ),
            challenge.replace("service=\"fixture.azurecr.io", "service=\"other.azurecr.io"),
            format!("{challenge},service=\"fixture.azurecr.io\""),
            challenge.replace("/oauth2/token", "/oauth2/token?redirect=1"),
        ] {
            assert!(ContainerRegistryBearerChallenge::parse(&url, &altered).is_err());
        }
    }
}

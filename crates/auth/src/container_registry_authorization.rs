use crate::AuthContext;
use crate::container_registry_access_token_response::ContainerRegistryAccessTokenResponse;
use crate::container_registry_refresh_token_response::ContainerRegistryRefreshTokenResponse;
use crate::fetch_azure_bearer_access_token;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_auth_types::ContainerRegistryBearerChallenge;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use reqwest::Client;
use std::time::Duration;

/// Exchange Entra -> ACR refresh -> scoped ACR access token without execution artifacts.
/// Forms follow the [official SDK implementation](https://github.com/Azure/azure-sdk-for-python/blob/main/sdk/containerregistry/azure-containerregistry/azure/containerregistry/_generated/operations/_patch.py).
/// Redirects are disabled so credential forms cannot leave the bound registry.
pub(crate) async fn authorize_container_registry(
    challenge: &ContainerRegistryBearerChallenge,
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
            let origin = challenge.server().endpoint()?;
            let response = client
                .post(origin.join("/oauth2/exchange?api-version=2021-07-01")?)
                .form(&[
                    ("grant_type", "access_token"),
                    ("service", challenge.server().as_str()),
                    ("tenant", tenant.to_string().as_str()),
                    ("access_token", token.access_token.as_str()),
                ])
                .send()
                .await?
                .error_for_status()?;
            let refresh = ContainerRegistryRefreshTokenResponse::decode(&response.text().await?)?;
            scope.bail_if_cancelled()?;
            let response = client
                .post(origin.join("/oauth2/token?api-version=2021-07-01")?)
                .form(&[
                    ("grant_type", "refresh_token"),
                    ("service", challenge.server().as_str()),
                    ("scope", challenge.scope()),
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

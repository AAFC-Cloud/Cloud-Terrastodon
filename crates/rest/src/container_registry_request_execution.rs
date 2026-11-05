use crate::RequestHeaders;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_auth::ContainerRegistryBearerChallenge;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::AzureBearerAuthSource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::ContainerRegistryLoginServer;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use eyre::ensure;
use http::Method;
use http::StatusCode;
use reqwest::Client;
use reqwest::Response;
use reqwest::Url;
use reqwest::header::AUTHORIZATION;
use reqwest::header::WWW_AUTHENTICATE;
use std::time::Duration;

/// Metadata GETs follow the registry's scoped bearer challenge; token exchanges stay out of artifacts.
pub(crate) async fn execute_container_registry_request(
    auth_context: &AuthContext,
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    tenant: Option<AzureTenantId>,
    cancellation: &CancellationScope,
) -> Result<Response> {
    ensure!(
        method == Method::GET && body.is_none(),
        "registry metadata transport currently supports bodyless GET requests"
    );
    let tenant = tenant.ok_or_else(|| {
        eyre::eyre!("Container Registry authentication requires an explicit tenant")
    })?;
    let source = AzureBearerAuthSource::try_from(auth_context.source().ok_or_else(|| {
        eyre::eyre!("Container Registry authentication requires an explicit credential source")
    })?)?;
    auth_context.admit(&AuthenticationSelection::AzureBearer {
        auth_source: source,
        tenant_id: tenant,
    })?;
    ensure!(
        url.scheme() == "https"
            && url.username().is_empty()
            && url.password().is_none()
            && url.port().is_none(),
        "Container Registry transport requires a secure registry endpoint"
    );
    let _: ContainerRegistryLoginServer = url
        .host_str()
        .ok_or_else(|| eyre::eyre!("registry URL has no hostname"))?
        .parse()?;
    let headers = headers
        .map(|headers| headers.to_header_map())
        .transpose()?
        .unwrap_or_default();
    ensure!(
        !headers.contains_key(AUTHORIZATION),
        "registry authorization must come from its selected credential source"
    );
    cancellation
        .cancellable_scope(|scope| async move {
            let client = Client::builder()
                .https_only(true)
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(30))
                .timeout(Duration::from_secs(120))
                .build()?;
            let response = client
                .get(url.clone())
                .headers(headers.clone())
                .send()
                .await?;
            if response.status() != StatusCode::UNAUTHORIZED {
                return Ok(response);
            }
            let challenge = response
                .headers()
                .get(WWW_AUTHENTICATE)
                .ok_or_else(|| {
                    eyre::eyre!("registry rejected the request without a bearer challenge")
                })?
                .to_str()?;
            let challenge = ContainerRegistryBearerChallenge::parse(&url, challenge)?;
            let authorization = challenge.authorize(auth_context, tenant, &scope).await?;
            scope.bail_if_cancelled()?;
            let header = authorization
                .header()
                .ok_or_else(|| eyre::eyre!("registry exchange returned anonymous authorization"))?;
            Ok(client
                .get(url)
                .headers(headers)
                .header(AUTHORIZATION, header)
                .send()
                .await?)
        })
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn disabled_authentication_rejects_before_registry_contact() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|scope| async move {
                let error = execute_container_registry_request(
                    &AuthContext::explicit(AuthSource::None),
                    Method::GET,
                    Url::parse("https://synthetic-unused.azurecr.io/acr/v1/_catalog")?,
                    None,
                    None,
                    Some("00000000-0000-0000-0000-000000000001".parse()?),
                    &scope,
                )
                .await
                .expect_err("none authentication must fail before transport");
                assert!(error.to_string().contains("none"));
                Ok(())
            })
            .await
    }
}

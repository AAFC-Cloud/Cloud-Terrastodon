use crate::RequestHeaders;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::ContainerRegistryBearerChallenge;
use cloud_terrastodon_azure_types::ContainerRegistryLoginServer;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
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
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    selection: &AuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<Response> {
    ensure!(
        method == Method::GET && body.is_none(),
        "registry metadata transport currently supports bodyless GET requests"
    );
    ensure!(
        matches!(selection, AuthenticationSelection::AzureBearer { .. }),
        "Container Registry requires an explicitly tenant-bound bearer selection"
    );
    executor.admit(selection)?;
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
    executor
        .cancellable_scope(|child| async move {
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
            let authorization = child
                .container_registry_authorization(selection, &challenge)
                .await?;
            let Authorization::Bearer(authorization) = authorization else {
                eyre::bail!(
                    "registry exchange returned authorization other than a bearer credential"
                );
            };
            child.cancellation().bail_if_cancelled()?;
            Ok(client
                .get(url)
                .headers(headers)
                .header(AUTHORIZATION, authorization.header())
                .send()
                .await?)
        })
        .await
}

use crate::RequestHeaders;
use crate::RestService;
use cloud_terrastodon_auth_types::AuthenticationSelection;
use cloud_terrastodon_auth_types::Authorization;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::ensure;
use http::Method;
use reqwest::ClientBuilder;
use reqwest::Response;
use reqwest::Url;
use reqwest::header::AUTHORIZATION;
use reqwest::header::CONTENT_TYPE;
use reqwest::tls::Version;
use tracing::debug;

pub async fn read_optional_body(body: Option<String>) -> Result<Option<String>> {
    let Some(body) = body else {
        return Ok(None);
    };
    if let Some(file_path) = body.strip_prefix('@') {
        Ok(Some(std::fs::read_to_string(file_path)?))
    } else {
        Ok(Some(body))
    }
}

/// Execute one service exchange using only the executor's admitted credentials.
/// Request bodies, response decoding and artifact policy are owned by RestRequest.
pub async fn execute_rest_request(
    service: RestService,
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    selection: &AuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<Response> {
    match (service, selection) {
        (RestService::AzureRetailPrices, AuthenticationSelection::Anonymous)
        | (RestService::AzureDevOps, AuthenticationSelection::AzureDevOpsPersonalAccessToken) => {}
        (RestService::AzureRetailPrices, _) => {
            eyre::bail!("the public retail-price service requires anonymous authentication");
        }
        (_, AuthenticationSelection::AzureBearer { .. }) => {}
        _ => {
            eyre::bail!("the selected authentication cannot authorize this REST service");
        }
    }
    // Public lower-level callers retain admission before HTTP contact, including
    // the unauthenticated request which obtains an ACR challenge.
    executor.admit(selection)?;
    if service == RestService::AzureContainerRegistry {
        return crate::container_registry_request_execution::execute_container_registry_request(
            method, url, body, headers, selection, executor,
        )
        .await;
    }
    let headers = headers
        .map(|headers| headers.to_header_map())
        .transpose()?
        .unwrap_or_default();
    ensure!(
        !headers.contains_key(AUTHORIZATION),
        "REST authorization must come from the executor's selected credential provider"
    );
    executor.cancellable_scope(|child| async move {
        let authorization = match service {
            RestService::AzureRetailPrices => Authorization::Anonymous,
            RestService::AzureDevOps => child.authorization(selection, AzureCredentialResource::AzureDevOps).await?,
            RestService::MicrosoftGraph => child.authorization(selection, AzureCredentialResource::MicrosoftGraph).await?,
            RestService::AzureResourceManager => child.authorization(selection, AzureCredentialResource::AzureResourceManager).await?,
            RestService::AzureKeyVault => child.authorization(selection, AzureCredentialResource::AzureKeyVault).await?,
            RestService::AzureContainerRegistry => {
                eyre::bail!("Container Registry requires its challenge exchange");
            }
        };
        ensure!(matches!((selection, &authorization),
            (AuthenticationSelection::Anonymous, Authorization::Anonymous)
            | (AuthenticationSelection::AzureBearer { .. }, Authorization::Bearer(_))
            | (AuthenticationSelection::AzureDevOpsPersonalAccessToken, Authorization::PersonalAccessToken(_))),
            "the credential provider returned authorization inconsistent with the request selection");
        child.cancellation().bail_if_cancelled()?;
        debug!(?method, %url, ?service, "Executing REST request");
        let has_content_type = headers.contains_key(CONTENT_TYPE);
        let mut request = ClientBuilder::new().min_tls_version(Version::TLS_1_2).build()?
            .request(method, url).headers(headers);
        if let Some(header) = authorization.header() {
            request = request.header(AUTHORIZATION, header);
        }
        if let Some(body) = body {
            if !has_content_type {
                request = request.header(CONTENT_TYPE, "application/json");
            }
            request = request.body(body);
        }
        Ok(request.send().await?)
    }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
    use std::sync::Arc;
    use teamy_cancellation::CancellationToken;

    #[tokio::test]
    async fn disabled_authentication_rejects_before_any_service_contact() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|scope| async move {
                let executor = CloudTerrastodonExecutor::new(
                    WorkCache::temporary()?,
                    Arc::new(AnonymousAuthenticationProvider),
                    scope,
                );
                let selection = AuthenticationSelection::AzureBearer {
                    auth_source: AzureBearerAuthSource::AzureCli,
                    tenant_id: "11111111-1111-1111-1111-111111111111".parse()?,
                };
                for service in [
                    RestService::AzureResourceManager,
                    RestService::MicrosoftGraph,
                    RestService::AzureDevOps,
                    RestService::AzureKeyVault,
                    RestService::AzureContainerRegistry,
                ] {
                    let error = execute_rest_request(
                        service,
                        Method::GET,
                        Url::parse("file:///synthetic-no-request")?,
                        None,
                        None,
                        &selection,
                        &executor,
                    )
                    .await
                    .expect_err("disabled authentication must fail before transport");
                    assert!(error.to_string().contains("anonymous"));
                }
                Ok(())
            })
            .await
    }
}

use crate::RequestHeaders;
use crate::RestService;
use cloud_terrastodon_auth::AuthBearerExt;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_auth::AzureBearerToken;
use cloud_terrastodon_auth::create_azure_devops_rest_client;
use cloud_terrastodon_auth::fetch_azure_bearer_access_token;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureCredentialResource;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use http::Method;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Response;
use reqwest::Url;
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

#[expect(
    clippy::too_many_arguments,
    reason = "request execution keeps the authentication context explicit"
)]
pub async fn execute_rest_request(
    auth_context: &AuthContext,
    service: RestService,
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    tenant: Option<AzureTenantId>,
    bearer_token: Option<String>,
    cancellation: &CancellationScope,
) -> Result<Response> {
    match service {
        RestService::AzureRetailPrices => {
            // The public catalogue is anonymous, including --auth-source none.
            // https://learn.microsoft.com/rest/api/cost-management/retail-prices/azure-retail-prices
            eyre::ensure!(
                bearer_token.is_none() && tenant.is_none(),
                "the public retail-price service does not accept bearer credentials or a tenant"
            );
            let mut request = create_tls12_client()?.request(method, url);
            if let Some(body) = body {
                request = request.header(CONTENT_TYPE, "application/json").body(body);
            }
            if let Some(headers) = headers {
                let headers = headers.to_header_map()?;
                eyre::ensure!(
                    !headers.contains_key(reqwest::header::AUTHORIZATION),
                    "the public retail-price service does not accept authorization headers"
                );
                request = request.headers(headers);
            }
            cancellation
                .cancellable_scope(|_| async { Ok::<_, eyre::Report>(request.send().await?) })
                .await
        }
        RestService::AzureDevOps => {
            execute_azure_devops_request(
                auth_context,
                method,
                url,
                body,
                headers,
                tenant,
                bearer_token,
                cancellation,
            )
            .await
        }
        RestService::MicrosoftGraph => {
            execute_azure_bearer_request(
                auth_context,
                method,
                url,
                body,
                headers,
                tenant,
                bearer_token,
                AzureCredentialResource::MicrosoftGraph,
                cancellation,
            )
            .await
        }
        RestService::AzureResourceManager => {
            execute_azure_bearer_request(
                auth_context,
                method,
                url,
                body,
                headers,
                tenant,
                bearer_token,
                AzureCredentialResource::AzureResourceManager,
                cancellation,
            )
            .await
        }
        RestService::AzureKeyVault => {
            execute_azure_bearer_request(
                auth_context,
                method,
                url,
                body,
                headers,
                tenant,
                bearer_token,
                AzureCredentialResource::AzureKeyVault,
                cancellation,
            )
            .await
        }
        RestService::AzureContainerRegistry => {
            eyre::ensure!(
                bearer_token.is_none(),
                "Container Registry authenticates using the explicitly selected credential source"
            );
            crate::container_registry_request_execution::execute_container_registry_request(
                auth_context,
                method,
                url,
                body,
                headers,
                tenant,
                cancellation,
            )
            .await
        }
    }
}

#[expect(
    clippy::too_many_arguments,
    reason = "request execution keeps the authentication context explicit"
)]
pub async fn execute_azure_devops_request(
    auth_context: &AuthContext,
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    tenant: Option<AzureTenantId>,
    bearer_token: Option<String>,
    cancellation: &CancellationScope,
) -> Result<Response> {
    let source = auth_context.source();
    eyre::ensure!(
        source != Some(AuthSource::None),
        "authentication is disabled by auth source none; cannot execute an authenticated Azure DevOps request"
    );

    if matches!(source, Some(AuthSource::PersonalAccessToken)) {
        let pat = auth_context.personal_access_token(cancellation).await?;
        return execute_authenticated_azure_devops_request(
            method,
            url,
            body,
            headers,
            pat.as_ref(),
            cancellation,
        )
        .await;
    }

    let tenant = tenant.ok_or_else(|| {
        eyre::eyre!("Azure DevOps bearer authentication requires an explicit tenant")
    })?;
    if let Some(bearer_token) = bearer_token {
        let bearer_token = AzureBearerToken::new(bearer_token);
        return execute_authenticated_azure_devops_request(
            method,
            url,
            body,
            headers,
            &bearer_token,
            cancellation,
        )
        .await;
    }

    let token = fetch_azure_bearer_access_token(
        auth_context,
        tenant,
        AzureCredentialResource::AzureDevOps,
        cancellation,
    )
    .await?;
    execute_authenticated_azure_devops_request(method, url, body, headers, &token, cancellation)
        .await
}

async fn execute_authenticated_azure_devops_request(
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    token: &impl AuthBearerExt,
    cancellation: &CancellationScope,
) -> Result<Response> {
    let client = create_azure_devops_rest_client(token).await?;
    debug!(?method, %url, "Executing Azure DevOps REST request");
    let mut request_builder = client.request(method, url);
    if let Some(body) = body {
        request_builder = request_builder
            .header(CONTENT_TYPE, "application/json")
            .body(body);
    }
    if let Some(headers) = headers {
        request_builder = request_builder.headers(headers.to_header_map()?);
    }
    cancellation
        .cancellable_scope(|_| async { Ok::<_, eyre::Report>(request_builder.send().await?) })
        .await
}

#[expect(
    clippy::too_many_arguments,
    reason = "request execution keeps the authentication context explicit"
)]
pub async fn execute_azure_bearer_request(
    auth_context: &AuthContext,
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    tenant: Option<AzureTenantId>,
    bearer_token: Option<String>,
    resource: AzureCredentialResource,
    cancellation: &CancellationScope,
) -> Result<Response> {
    eyre::ensure!(
        auth_context.source() != Some(AuthSource::None),
        "authentication is disabled by auth source none; cannot execute an authenticated Azure request"
    );
    let tenant = tenant
        .ok_or_else(|| eyre::eyre!("Azure bearer authentication requires an explicit tenant"))?;
    let token = match bearer_token {
        Some(token) => token,
        None => fetch_azure_bearer_access_token(auth_context, tenant, resource, cancellation)
            .await?
            .access_token
            .as_str()
            .to_owned(),
    };
    let client = create_tls12_client()?;
    debug!(?method, %url, ?resource, ?tenant, "Executing Azure REST request");
    let mut request_builder = client.request(method, url).bearer_auth(token);
    if let Some(body) = body {
        request_builder = request_builder
            .header(CONTENT_TYPE, "application/json")
            .body(body);
    }
    if let Some(headers) = headers {
        request_builder = request_builder.headers(headers.to_header_map()?);
    }
    cancellation
        .cancellable_scope(|_| async { Ok::<_, eyre::Report>(request_builder.send().await?) })
        .await
}

fn create_tls12_client() -> Result<Client> {
    Ok(ClientBuilder::new()
        .min_tls_version(Version::TLS_1_2)
        .build()?)
}

#[cfg(test)]
mod tests {
    use super::AzureCredentialResource;
    use super::execute_azure_bearer_request;
    use super::execute_azure_devops_request;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_auth_types::AuthSource;
    use cloud_terrastodon_command::CancellationToken;
    use http::Method;
    use reqwest::Url;

    #[tokio::test]
    async fn bearer_requests_require_a_tenant_before_credentials_or_network() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|scope| async move {
                let context = AuthContext::explicit(AuthSource::AzureCli);
                let url = Url::parse("file:///synthetic-no-request")?;
                for token in [None, Some("synthetic-unused-token".to_owned())] {
                    let error = execute_azure_devops_request(
                        &context,
                        Method::GET,
                        url.clone(),
                        None,
                        None,
                        None,
                        token.clone(),
                        &scope,
                    )
                    .await
                    .unwrap_err();
                    assert!(error.to_string().contains("explicit tenant"));
                    let error = execute_azure_bearer_request(
                        &context,
                        Method::GET,
                        url.clone(),
                        None,
                        None,
                        None,
                        token,
                        AzureCredentialResource::MicrosoftGraph,
                        &scope,
                    )
                    .await
                    .unwrap_err();
                    assert!(error.to_string().contains("explicit tenant"));
                }
                Ok(())
            })
            .await
    }

    #[tokio::test]
    async fn disabled_authentication_cannot_be_overridden_by_an_explicit_bearer_token()
    -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let context = AuthContext::resolve(AuthSource::None)?;
                // The unsupported scheme prevents network access even if the
                // policy guard regresses and reaches the HTTP client.
                let url = Url::parse("file:///synthetic-no-request")?;
                let azure_devops = execute_azure_devops_request(
                    &context,
                    Method::GET,
                    url.clone(),
                    None,
                    None,
                    None,
                    Some("synthetic-unused-token".into()),
                    &cancellation,
                )
                .await
                .expect_err("disabled authentication must reject an explicit bearer token");
                let azure = execute_azure_bearer_request(
                    &context,
                    Method::GET,
                    url,
                    None,
                    None,
                    None,
                    Some("synthetic-unused-token".into()),
                    AzureCredentialResource::MicrosoftGraph,
                    &cancellation,
                )
                .await
                .expect_err("disabled authentication must reject an explicit bearer token");
                assert!(azure_devops.to_string().contains("auth source none"));
                assert!(azure.to_string().contains("auth source none"));
                Ok(())
            })
            .await
    }

    #[test]
    fn explicit_context_preserves_source() {
        let context = AuthContext::explicit(AuthSource::PersonalAccessToken);
        assert_eq!(context.source(), Some(AuthSource::PersonalAccessToken));
    }

    #[test]
    fn auth_source_remains_a_request_policy() {
        let context = AuthContext::explicit(AuthSource::AzureCli);
        assert_eq!(context.source(), Some(AuthSource::AzureCli));
    }

    #[test]
    fn auth_source_type_remains_available_for_callers() {
        assert_eq!(AuthSource::Auto.to_string(), "auto");
    }
}

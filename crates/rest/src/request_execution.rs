use crate::RequestHeaders;
use crate::RestService;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AuthSource;
use cloud_terrastodon_credentials::create_azure_devops_rest_client;
use cloud_terrastodon_credentials::fetch_azure_bearer_access_token;
use cloud_terrastodon_credentials::get_azure_devops_personal_access_token_from_credential_manager;
use eyre::Result;
use http::Method;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Response;
use reqwest::Url;
use reqwest::header::CONTENT_TYPE;
use reqwest::tls::Version;
use tracing::debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AzureRestResource {
    MicrosoftGraph,
    AzureResourceManager,
    AzureDevOps,
}

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
                AzureRestResource::MicrosoftGraph,
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
                AzureRestResource::AzureResourceManager,
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
        let pat = get_azure_devops_personal_access_token_from_credential_manager()
            .await
            .map_err(|error| {
                eyre::eyre!(
                    "Azure DevOps PAT authentication was selected but no PAT was available: {error}"
                )
            })?;
        return execute_authenticated_azure_devops_request(
            method,
            url,
            body,
            headers,
            &pat,
            cancellation,
        )
        .await;
    }

    if let Some(bearer_token) = bearer_token {
        let bearer_token = cloud_terrastodon_credentials::AzureBearerToken::new(bearer_token);
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
        cloud_terrastodon_credentials::AzureRestResource::AzureDevOps,
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
    token: &impl cloud_terrastodon_credentials::AuthBearerExt,
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
    resource: AzureRestResource,
    cancellation: &CancellationScope,
) -> Result<Response> {
    eyre::ensure!(
        auth_context.source() != Some(AuthSource::None),
        "authentication is disabled by auth source none; cannot execute an authenticated Azure request"
    );
    let token = match bearer_token {
        Some(token) => token,
        None => match resource {
            AzureRestResource::MicrosoftGraph => fetch_azure_bearer_access_token(
                auth_context,
                tenant,
                cloud_terrastodon_credentials::AzureRestResource::MicrosoftGraph,
                cancellation,
            )
            .await?
            .access_token
            .as_str()
            .to_owned(),
            AzureRestResource::AzureResourceManager => fetch_azure_bearer_access_token(
                auth_context,
                tenant,
                cloud_terrastodon_credentials::AzureRestResource::AzureResourceManager,
                cancellation,
            )
            .await?
            .access_token
            .as_str()
            .to_owned(),
            AzureRestResource::AzureDevOps => fetch_azure_bearer_access_token(
                auth_context,
                tenant,
                cloud_terrastodon_credentials::AzureRestResource::AzureDevOps,
                cancellation,
            )
            .await?
            .access_token
            .as_str()
            .to_owned(),
        },
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
    use super::AzureRestResource;
    use super::execute_azure_bearer_request;
    use super::execute_azure_devops_request;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;
    use cloud_terrastodon_credentials::AuthSource;
    use http::Method;
    use reqwest::Url;

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
                    AzureRestResource::MicrosoftGraph,
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

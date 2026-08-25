use crate::RequestHeaders;
use crate::RestService;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AuthSource;
use cloud_terrastodon_credentials::create_azure_devops_rest_client;
use cloud_terrastodon_credentials::fetch_azure_bearer_access_token;
use cloud_terrastodon_credentials::get_azure_devops_personal_access_token_from_credential_manager;
use eyre::Result;
use eyre::bail;
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
) -> Result<Response> {
    match service {
        RestService::AzureDevOps => {
            if tenant.is_some() {
                bail!("--tenant is not supported for Azure DevOps REST URLs")
            }
            execute_azure_devops_request(auth_context, method, url, body, headers).await
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
            )
            .await
        }
    }
}

pub async fn execute_azure_devops_request(
    auth_context: &AuthContext,
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
) -> Result<Response> {
    let source = auth_context.source();

    if matches!(source, AuthSource::PersonalAccessToken) {
        let pat = get_azure_devops_personal_access_token_from_credential_manager()
            .await
            .map_err(|error| {
                eyre::eyre!(
                    "Azure DevOps PAT authentication was selected but no PAT was available: {error}"
                )
            })?;
        return execute_authenticated_azure_devops_request(method, url, body, headers, &pat).await;
    }

    let token = fetch_azure_bearer_access_token(
        auth_context,
        None,
        cloud_terrastodon_credentials::AzureRestResource::AzureDevOps,
    )
    .await?;
    execute_authenticated_azure_devops_request(method, url, body, headers, &token).await
}

async fn execute_authenticated_azure_devops_request(
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    token: &impl cloud_terrastodon_credentials::AuthBearerExt,
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
    Ok(request_builder.send().await?)
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
) -> Result<Response> {
    let token = match bearer_token {
        Some(token) => token,
        None => match resource {
            AzureRestResource::MicrosoftGraph => fetch_azure_bearer_access_token(
                auth_context,
                tenant,
                cloud_terrastodon_credentials::AzureRestResource::MicrosoftGraph,
            )
            .await?
            .access_token
            .as_str()
            .to_owned(),
            AzureRestResource::AzureResourceManager => fetch_azure_bearer_access_token(
                auth_context,
                tenant,
                cloud_terrastodon_credentials::AzureRestResource::AzureResourceManager,
            )
            .await?
            .access_token
            .as_str()
            .to_owned(),
            AzureRestResource::AzureDevOps => fetch_azure_bearer_access_token(
                auth_context,
                tenant,
                cloud_terrastodon_credentials::AzureRestResource::AzureDevOps,
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
    Ok(request_builder.send().await?)
}

fn create_tls12_client() -> Result<Client> {
    Ok(ClientBuilder::new()
        .min_tls_version(Version::TLS_1_2)
        .build()?)
}

#[cfg(test)]
mod tests {
    use cloud_terrastodon_credentials::AuthContext;
    use cloud_terrastodon_credentials::AuthSource;

    #[test]
    fn explicit_context_preserves_source() {
        let context = AuthContext::explicit(AuthSource::PersonalAccessToken);
        assert_eq!(context.source(), AuthSource::PersonalAccessToken);
    }

    #[test]
    fn auth_source_remains_a_request_policy() {
        let context = AuthContext::explicit(AuthSource::AzureCli);
        assert_eq!(context.source(), AuthSource::AzureCli);
    }

    #[test]
    fn auth_source_type_remains_available_for_callers() {
        assert_eq!(AuthSource::Auto.to_string(), "auto");
    }
}

use crate::AzureRestResource;
use crate::RequestHeaders;
use crate::RestService;
use crate::create_azure_devops_rest_client;
use crate::fetch_azure_access_token;
use crate::get_azure_devops_personal_access_token_from_credential_manager;
use cloud_terrastodon_azure_types::AzureTenantId;
use eyre::Result;
use eyre::bail;
use http::Method;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Response;
use reqwest::Url;
use reqwest::header::CONTENT_TYPE;
use reqwest::tls::Version;

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

pub async fn execute_rest_request(
    service: RestService,
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    tenant: Option<AzureTenantId>,
) -> Result<Response> {
    match service {
        RestService::AzureDevOps => {
            if tenant.is_some() {
                bail!("--tenant is not supported for Azure DevOps REST URLs")
            }
            execute_azure_devops_request(method, url, body, headers).await
        }
        RestService::MicrosoftGraph => {
            execute_azure_bearer_request(
                method,
                url,
                body,
                headers,
                tenant,
                AzureRestResource::MicrosoftGraph,
            )
            .await
        }
        RestService::AzureResourceManager => {
            execute_azure_bearer_request(
                method,
                url,
                body,
                headers,
                tenant,
                AzureRestResource::AzureResourceManager,
            )
            .await
        }
    }
}

pub async fn execute_azure_devops_request(
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
) -> Result<Response> {
    let pat = get_azure_devops_personal_access_token_from_credential_manager().await?;
    let client = create_azure_devops_rest_client(&pat).await?;
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

pub async fn execute_azure_bearer_request(
    method: Method,
    url: Url,
    body: Option<String>,
    headers: Option<RequestHeaders>,
    tenant: Option<AzureTenantId>,
    resource: AzureRestResource,
) -> Result<Response> {
    let token = fetch_azure_access_token::<String>(tenant, resource).await?;
    let client = create_tls12_client()?;
    let mut request_builder = client.request(method, url).bearer_auth(&token.access_token);
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

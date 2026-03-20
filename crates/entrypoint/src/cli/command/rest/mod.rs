use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::AzureTenantArgumentExt;
use cloud_terrastodon_azure::prelude::AzureTenantId;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_credentials::create_azure_devops_rest_client;
use cloud_terrastodon_credentials::get_azure_devops_personal_access_token_from_credential_manager;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use http::Method;
use reqwest::Client;
use reqwest::ClientBuilder;
use reqwest::Response;
use reqwest::Url;
use reqwest::header::CONTENT_TYPE;
use reqwest::tls::Version;
use serde::Deserialize;

/// Arguments for issuing raw REST calls with Cloud Terrastodon's auth helpers.
#[derive(Args, Debug, Clone)]
pub struct RestArgs {
    /// The HTTP method to use for the REST call.
    #[arg(long)]
    pub method: Method,

    /// The REST API URL to call.
    #[arg(long)]
    pub url: String,

    /// Optional request body for POST/PUT requests. If begins with '@', reads from file.
    #[arg(long)]
    pub body: Option<String>,

    /// Optional tracked tenant id or alias to use when acquiring Azure access tokens.
    #[arg(long)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

impl RestArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = match self.tenant {
            Some(tenant) => Some(tenant.resolve().await?),
            None => None,
        };
        let url = Url::parse(&self.url).with_context(|| format!("parsing URL '{}'", self.url))?;
        let service = RestService::infer(&url)
            .ok_or_else(|| eyre::eyre!("unsupported REST host '{}'", url.host_str().unwrap_or("")))?;
        let body = read_optional_body(self.body).await?;

        let response = match service {
            RestService::AzureDevOps => {
                if tenant.is_some() {
                    bail!("--tenant is not supported for Azure DevOps REST URLs")
                }
                execute_azure_devops_request(self.method, url, body).await?
            }
            RestService::MicrosoftGraph => {
                execute_azure_bearer_request(self.method, url, body, tenant, AzureResource::MicrosoftGraph)
                    .await?
            }
            RestService::AzureResourceManager => {
                execute_azure_bearer_request(
                    self.method,
                    url,
                    body,
                    tenant,
                    AzureResource::AzureResourceManager,
                )
                .await?
            }
        };

        print_response(response).await
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RestService {
    AzureDevOps,
    MicrosoftGraph,
    AzureResourceManager,
}

impl RestService {
    fn infer(url: &Url) -> Option<Self> {
        let host = url.host_str()?.to_ascii_lowercase();
        match host.as_str() {
            "graph.microsoft.com" => Some(Self::MicrosoftGraph),
            "management.azure.com" => Some(Self::AzureResourceManager),
            "dev.azure.com" | "vssps.dev.azure.com" | "vsrm.dev.azure.com"
            | "vsaex.dev.azure.com" | "app.vssps.visualstudio.com" => Some(Self::AzureDevOps),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum AzureResource {
    MicrosoftGraph,
    AzureResourceManager,
}

#[derive(Debug, Deserialize)]
struct AzureCliAccessToken {
    #[serde(rename = "accessToken")]
    access_token: String,
}

impl AzureResource {
    fn resource_type(self) -> Option<&'static str> {
        match self {
            AzureResource::MicrosoftGraph => Some("ms-graph"),
            AzureResource::AzureResourceManager => None,
        }
    }
}

async fn read_optional_body(body: Option<String>) -> Result<Option<String>> {
    let Some(body) = body else {
        return Ok(None);
    };

    if let Some(file_path) = body.strip_prefix('@') {
        Ok(Some(tokio::fs::read_to_string(file_path).await?))
    } else {
        Ok(Some(body))
    }
}

async fn execute_azure_devops_request(
    method: Method,
    url: Url,
    body: Option<String>,
) -> Result<Response> {
    let pat = get_azure_devops_personal_access_token_from_credential_manager().await?;
    let client = create_azure_devops_rest_client(&pat).await?;
    let mut request_builder = client.request(method, url);
    if let Some(body) = body {
        request_builder = request_builder.header(CONTENT_TYPE, "application/json").body(body);
    }
    Ok(request_builder.send().await?)
}

async fn execute_azure_bearer_request(
    method: Method,
    url: Url,
    body: Option<String>,
    tenant: Option<AzureTenantId>,
    resource: AzureResource,
) -> Result<Response> {
    let token = fetch_azure_access_token(tenant, resource).await?;
    let client = create_tls12_client()?;
    let mut request_builder = client.request(method, url).bearer_auth(&token.access_token);
    if let Some(body) = body {
        request_builder = request_builder.header(CONTENT_TYPE, "application/json").body(body);
    }
    Ok(request_builder.send().await?)
}

fn create_tls12_client() -> Result<Client> {
    Ok(ClientBuilder::new()
        .min_tls_version(Version::TLS_1_2)
        .build()?)
}

async fn fetch_azure_access_token(
    tenant: Option<AzureTenantId>,
    resource: AzureResource,
) -> Result<AzureCliAccessToken> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["account", "get-access-token", "--output", "json"]);
    if let Some(tenant) = tenant {
        let tenant = tenant.to_string();
        cmd.args(["--tenant", tenant.as_str()]);
    }
    if let Some(resource_type) = resource.resource_type() {
        cmd.args(["--resource-type", resource_type]);
    }
    cmd.run::<AzureCliAccessToken>().await
}

async fn print_response(response: Response) -> Result<()> {
    let status = response.status();
    let content = response.text().await?;
    println!("{}", content);
    if !status.is_success() {
        bail!(
            "REST call failed with status {}: {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("Unknown error")
        );
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::RestService;
    use reqwest::Url;

    #[test]
    fn infers_microsoft_graph() {
        let url = Url::parse("https://graph.microsoft.com/v1.0/organization").unwrap();
        assert_eq!(RestService::infer(&url), Some(RestService::MicrosoftGraph));
    }

    #[test]
    fn infers_azure_resource_manager() {
        let url = Url::parse("https://management.azure.com/subscriptions?api-version=2020-01-01").unwrap();
        assert_eq!(RestService::infer(&url), Some(RestService::AzureResourceManager));
    }

    #[test]
    fn infers_azure_devops_hosts() {
        for host in [
            "https://dev.azure.com/example/_apis/projects?api-version=7.1",
            "https://vssps.dev.azure.com/example/_apis/graph/users?api-version=7.1-preview.1",
            "https://app.vssps.visualstudio.com/_apis/profile/profiles/me?api-version=6.0",
        ] {
            let url = Url::parse(host).unwrap();
            assert_eq!(RestService::infer(&url), Some(RestService::AzureDevOps));
        }
    }

    #[test]
    fn rejects_unknown_hosts() {
        let url = Url::parse("https://example.com/api").unwrap();
        assert_eq!(RestService::infer(&url), None);
    }
}

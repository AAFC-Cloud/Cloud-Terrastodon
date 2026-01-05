use clap::Args;
use cloud_terrastodon_credentials::create_azure_devops_rest_client;
use cloud_terrastodon_credentials::get_azure_devops_personal_access_token_from_credential_manager;
use eyre::Result;
use http::Method;

/// Arguments for issuing raw Azure DevOps REST calls.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsRestArgs {
    /// The HTTP method to use for the REST call.
    #[arg(long)]
    pub method: Method,

    /// The Azure DevOps REST API URL to call (e.g., https://dev.azure.com/{organization}/_apis/projects).
    #[arg(long)]
    pub url: String,

    /// Optional request body for POST/PUT requests. If begins with '@', reads from file.
    #[arg(long)]
    pub body: Option<String>,
}

impl AzureDevOpsRestArgs {
    pub async fn invoke(self) -> Result<()> {
        let pat = get_azure_devops_personal_access_token_from_credential_manager().await?;
        let client = create_azure_devops_rest_client(&pat).await?;
        let mut request_builder = client.request(self.method, &self.url);
        if let Some(body) = self.body {
            if let Some(file_path) = body.strip_prefix('@') {
                let file_content = tokio::fs::read_to_string(file_path).await?;
                request_builder = request_builder.body(file_content);
            } else {
                request_builder = request_builder.body(body);
            }
        }
        let response = request_builder.send().await?;
        let status = response.status();
        let content = response.text().await?;
        println!("{}", content);
        if !status.is_success() {
            eyre::bail!(
                "Azure DevOps REST call failed with status {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown error")
            );
        }
        Ok(())
    }
}

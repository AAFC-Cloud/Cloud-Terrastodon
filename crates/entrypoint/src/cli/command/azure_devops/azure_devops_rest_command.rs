use clap::Args;
use cloud_terrastodon_credentials::{create_azure_devops_rest_client, get_azure_devops_pat};
use eyre::Result;

/// Arguments for issuing raw Azure DevOps REST calls.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsRestArgs {
    /// The HTTP method to use for the REST call.
    #[arg(long)]
    pub method: String,

    /// The Azure DevOps REST API URL to call (e.g., https://dev.azure.com/{organization}/_apis/projects).
    #[arg(long)]
    pub url: String,

    /// Optional request body for POST/PUT requests. If begins with '@', reads from file.
    #[arg(long)]
    pub body: Option<String>,
}

impl AzureDevOpsRestArgs {
    pub async fn invoke(self) -> Result<()> {
        let pat = get_azure_devops_pat().await?;
        let client = create_azure_devops_rest_client(&pat).await?;
        Ok(())
    }
}

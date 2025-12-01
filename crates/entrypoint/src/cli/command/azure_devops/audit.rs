use crate::noninteractive::prelude::audit_azure_devops;
use clap::Args;
use eyre::Result;

/// Arguments for auditing Azure DevOps resources.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAuditArgs {}

impl AzureDevOpsAuditArgs {
    pub async fn invoke(self) -> Result<()> {
        audit_azure_devops().await
    }
}

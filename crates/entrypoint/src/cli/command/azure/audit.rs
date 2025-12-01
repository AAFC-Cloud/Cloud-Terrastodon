use crate::noninteractive::prelude::audit_azure;
use clap::Args;
use eyre::Result;

/// Arguments for auditing Azure resources.
#[derive(Args, Debug, Clone)]
pub struct AzureAuditArgs {}

impl AzureAuditArgs {
    pub async fn invoke(self) -> Result<()> {
        audit_azure().await
    }
}

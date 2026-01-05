use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsWorkItemQueryId;
use cloud_terrastodon_azure_devops::prelude::fetch_work_items_for_query;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// Invoke a work item query.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsWorkItemQueryInvokeArgs {
    /// Project id or project name.
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Query id.
    pub id: AzureDevOpsWorkItemQueryId,
}

impl AzureDevOpsWorkItemQueryInvokeArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let result = fetch_work_items_for_query(&org_url, &self.id).await?;
        to_writer_pretty(stdout(), &result)?;
        Ok(())
    }
}

use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryId;
use cloud_terrastodon_azure_devops::fetch_work_items_for_query;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// Invoke a work item query.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsWorkItemQueryInvokeArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Query id.
    #[facet(figue::positional, proxy = String)]
    pub id: AzureDevOpsWorkItemQueryId,
}

impl AzureDevOpsWorkItemQueryInvokeArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let result = fetch_work_items_for_query(&org_url, &self.id).await?;
        to_writer_pretty(stdout(), &result)?;
        Ok(())
    }
}

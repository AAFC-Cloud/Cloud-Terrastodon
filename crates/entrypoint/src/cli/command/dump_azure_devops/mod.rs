use crate::noninteractive::dump_azure_devops;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use eyre::Result;

/// Dump Azure DevOps metadata to disk.
#[derive(facet::Facet, Debug, Clone, Default)]
pub struct DumpAzureDevOpsArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
}

impl DumpAzureDevOpsArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        dump_azure_devops(org_url).await?;
        Ok(())
    }
}

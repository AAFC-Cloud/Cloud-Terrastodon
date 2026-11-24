use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganizationStore;
use eyre::Result;
use eyre::bail;
use facet::Facet;

/// Show the local registration for an Azure DevOps organization.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsOrganizationShowArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::positional)]
    pub organization: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsOrganizationShowArgs {
    pub async fn invoke(self) -> Result<CliOutput> {
        let Some(organization) = TrackedAzureDevOpsOrganizationStore::configured()
            .show(&self.organization)
            .await?
        else {
            bail!(
                "Azure DevOps organization '{}' is not tracked on this device",
                self.organization
            );
        };
        Ok(CliOutput::facet(organization))
    }
}

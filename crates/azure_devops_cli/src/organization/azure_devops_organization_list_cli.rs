use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganizationStore;
use eyre::Result;
use facet::Facet;

/// List Azure DevOps organizations tracked on this device.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsOrganizationListArgs {}

impl AzureDevOpsOrganizationListArgs {
    pub async fn invoke(self) -> Result<CliOutput> {
        let organizations = TrackedAzureDevOpsOrganizationStore::configured()
            .list()
            .await?;
        Ok(CliOutput::facet(organizations))
    }
}

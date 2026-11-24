use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganizationStore;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use eyre::bail;
use facet::Facet;

/// Remove a local Azure DevOps organization registration.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsOrganizationRemoveArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::positional)]
    pub organization: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsOrganizationRemoveArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let Some(organization) = TrackedAzureDevOpsOrganizationStore::configured()
            .remove(&self.organization, cancellation)
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

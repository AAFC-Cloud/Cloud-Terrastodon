use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganization;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganizationStore;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Track an Azure DevOps organization on this device.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsOrganizationAddArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::positional)]
    pub organization: AzureDevOpsOrganizationUrl,

    /// Associated Azure tenant ID. Omit to clear an existing association.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantId>,
}

impl AzureDevOpsOrganizationAddArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let organization = TrackedAzureDevOpsOrganizationStore::configured()
            .add(
                TrackedAzureDevOpsOrganization {
                    url: self.organization,
                    tenant_id: self.tenant,
                },
                cancellation,
            )
            .await?;
        Ok(CliOutput::facet(organization))
    }
}

use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps license entitlements (users) for the organization.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserListArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
}

impl AzureDevOpsLicenseEntitlementUserListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let entitlements = fetch_azure_devops_user_license_entitlements(&org_url).await?;
        to_writer_pretty(stdout(), &entitlements)?;
        Ok(())
    }
}

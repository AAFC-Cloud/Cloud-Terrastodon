use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlement;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// Show a single Azure DevOps user license entitlement by user id.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserShowArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named, proxy = String)]
    pub user: AzureDevOpsUserArgument<'static>,
}

impl AzureDevOpsLicenseEntitlementUserShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let found = fetch_azure_devops_user_license_entitlement(&org_url, &self.user).await?;
        to_writer_pretty(stdout(), &found)?;
        Ok(())
    }
}

use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlements;
use cloud_terrastodon_command::to_writer_pretty;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps license entitlements (users) for the organization.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserListArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsLicenseEntitlementUserListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(auth_context)?;
        let entitlements =
            fetch_azure_devops_user_license_entitlements(&self.org, &azure_devops_auth_context)
                .await?;
        to_writer_pretty(stdout(), &entitlements)?;
        Ok(())
    }
}

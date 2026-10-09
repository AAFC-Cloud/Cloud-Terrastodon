use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_user_license_entitlement;
use cloud_terrastodon_command::to_writer_pretty;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use std::io::stdout;

/// Show a single Azure DevOps user license entitlement by user id.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserShowArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named, proxy = String)]
    pub user: AzureDevOpsUserArgument<'static>,
}

impl AzureDevOpsLicenseEntitlementUserShowArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(auth_context)?;
        let found = fetch_azure_devops_user_license_entitlement(
            &self.org,
            &self.user,
            &azure_devops_auth_context,
        )
        .await?;
        to_writer_pretty(stdout(), &found)?;
        Ok(())
    }
}

use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsGroupLicenseEntitlementsRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// List Azure DevOps group license entitlements for the organization.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementGroupListArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsLicenseEntitlementGroupListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let entitlements = AzureDevOpsGroupLicenseEntitlementsRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(entitlements))
    }
}

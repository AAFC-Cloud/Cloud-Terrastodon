use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlementShowRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// Show a single Azure DevOps user license entitlement by user id.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsLicenseEntitlementUserShowArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named, proxy = String)]
    pub user: AzureDevOpsUserArgument<'static>,
}

impl AzureDevOpsLicenseEntitlementUserShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let found = AzureDevOpsUserLicenseEntitlementShowRequest {
            org_url: Cow::Borrowed(&self.org),
            user: (&self.user).into(),
            auth_selection,
        }
        .execute(executor)
        .await?;
        Ok(CliOutput::facet(found))
    }
}

use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolEntitlementListForProjectRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

/// List Azure DevOps agent pool entitlements (queues) in a project.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolEntitlementListArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(figue::named)]
    pub pool: Option<AzureDevOpsAgentPoolArgument<'static>>,
}

impl AzureDevOpsAgentPoolEntitlementListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let entitlements = AzureDevOpsAgentPoolEntitlementListForProjectRequest {
            org_url: Cow::Borrowed(&self.org),
            project: (self.project),
            auth_selection,
        }
        .execute(executor)
        .await?;
        let entitlements: Vec<_> = entitlements
            .into_iter()
            .filter(|entitlement| {
                self.pool
                    .as_ref()
                    .is_none_or(|pool| pool.matches_entitlement(entitlement))
            })
            .collect();
        Ok(CliOutput::facet(entitlements))
    }
}

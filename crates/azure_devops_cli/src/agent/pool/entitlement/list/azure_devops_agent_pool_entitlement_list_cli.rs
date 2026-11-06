use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_pool_entitlements_for_project;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
use std::io::stdout;

/// List Azure DevOps agent pool entitlements (queues) in a project.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPoolEntitlementListArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

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
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &azure_devops_auth_context.bind_executor(executor)?;
        let entitlements = fetch_azure_devops_agent_pool_entitlements_for_project(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            executor,
        )
        .await?;
        let entitlements: Vec<_> = entitlements
            .into_iter()
            .filter(|entitlement| {
                self.pool
                    .as_ref()
                    .is_none_or(|pool| pool.matches_entitlement(entitlement))
            })
            .collect();
        facet_json::to_writer_std_pretty(stdout(), &entitlements)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(())
    }
}

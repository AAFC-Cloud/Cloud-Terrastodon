use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_pool_entitlements_for_project;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps agent pool entitlements (queues) in a project.
#[derive(facet::Facet, Debug, Clone)]
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
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let entitlements = fetch_azure_devops_agent_pool_entitlements_for_project(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            cancellation,
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
        to_writer_pretty(stdout(), &entitlements)?;
        Ok(())
    }
}

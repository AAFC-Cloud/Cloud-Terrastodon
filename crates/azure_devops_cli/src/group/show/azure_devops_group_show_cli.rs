use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::io::stdout;

/// Show Azure DevOps group details.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsGroupShowArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Group identifier (display name, principal name, origin id, or descriptor).
    #[facet(figue::named)]
    pub group: String,
}

impl AzureDevOpsGroupShowArgs {
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
        let groups = fetch_azure_devops_groups_for_project(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            executor,
        )
        .await?;
        if let Some(group) = groups.into_iter().find(|g| {
            g.display_name == self.group
                || g.principal_name == self.group
                || g.origin_id == self.group
                || g.descriptor.to_string() == self.group
        }) {
            facet_json::to_writer_std_pretty(stdout(), &group)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
            Ok(())
        } else {
            bail!("No group found matching '{}'.", self.group);
        }
    }
}

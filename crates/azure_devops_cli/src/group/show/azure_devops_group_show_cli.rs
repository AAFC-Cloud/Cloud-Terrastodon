use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps group details.
#[derive(facet::Facet, Debug, Clone)]
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
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let groups = fetch_azure_devops_groups_for_project(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            cancellation,
        )
        .await?;
        if let Some(group) = groups.into_iter().find(|g| {
            g.display_name == self.group
                || g.principal_name == self.group
                || g.origin_id == self.group
                || g.descriptor.to_string() == self.group
        }) {
            to_writer_pretty(stdout(), &group)?;
            Ok(())
        } else {
            bail!("No group found matching '{}'.", self.group);
        }
    }
}

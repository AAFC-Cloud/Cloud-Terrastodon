use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_project_members;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::Write;
use std::io::stdout;

/// List users that are transitively members of an Azure DevOps project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectMemberListArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsProjectMemberListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let members = fetch_azure_devops_project_members(
            &self.org,
            self.project,
            &azure_devops_auth_context,
            cancellation,
        )
        .await?;

        let mut out = stdout().lock();
        to_writer_pretty(&mut out, &members)?;
        out.write_all(b"\n")?;

        Ok(())
    }
}

use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_command::to_writer_pretty;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use std::io::stdout;

/// List Azure DevOps groups in a project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsGroupListArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsGroupListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(auth_context)?;
        let groups = fetch_azure_devops_groups_for_project(
            &org_url,
            self.project,
            &azure_devops_auth_context,
        )
        .await?;
        to_writer_pretty(stdout(), &groups)?;
        Ok(())
    }
}

use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_project_members;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::Write;
use std::io::stdout;

/// List users that are transitively members of an Azure DevOps project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectMemberListArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsProjectMemberListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let members = fetch_azure_devops_project_members(&org_url, self.project).await?;

        let mut out = stdout().lock();
        to_writer_pretty(&mut out, &members)?;
        out.write_all(b"\n")?;

        Ok(())
    }
}

use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_project_members;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::Write;
use std::io::stdout;

/// List users that are transitively members of an Azure DevOps project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectMemberListArgs {
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsProjectMemberListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let members = fetch_azure_devops_project_members(&org_url, self.project).await?;

        let mut out = stdout().lock();
        to_writer_pretty(&mut out, &members)?;
        out.write_all(b"\n")?;

        Ok(())
    }
}

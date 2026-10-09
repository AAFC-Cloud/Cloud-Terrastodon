use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_project_members;
use cloud_terrastodon_command::to_writer_pretty;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use eyre::Result;
use std::io::Write;
use std::io::stdout;

/// List users that are transitively members of an Azure DevOps project.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectMemberListArgs {
    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    /// Project id or project name.
    #[facet(figue::named, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,
}

impl AzureDevOpsProjectMemberListArgs {
    pub async fn invoke(self, auth_context: &AuthContext) -> Result<()> {
        let azure_devops_auth_context = AzureDevOpsAuthContext::new(auth_context)?;
        let members =
            fetch_azure_devops_project_members(&self.org, self.project, &azure_devops_auth_context)
                .await?;

        let mut out = stdout().lock();
        to_writer_pretty(&mut out, &members)?;
        out.write_all(b"\n")?;

        Ok(())
    }
}

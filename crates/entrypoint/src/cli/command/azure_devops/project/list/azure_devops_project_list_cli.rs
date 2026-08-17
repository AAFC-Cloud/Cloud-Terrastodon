use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::io::Write;
use std::io::stdout;

/// Azure DevOps project-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsProjectListArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
}

impl AzureDevOpsProjectListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        let mut out = stdout().lock();
        to_writer_pretty(&mut out, &projects)?;
        out.write_all(b"\n")?;

        Ok(())
    }
}

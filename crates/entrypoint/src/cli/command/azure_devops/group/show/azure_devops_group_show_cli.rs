use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps group details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsGroupShowArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    /// Project id or project name.
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Group identifier (display name, principal name, origin id, or descriptor).
    #[facet(figue::named)]
    pub group: String,
}

impl AzureDevOpsGroupShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let groups = fetch_azure_devops_groups_for_project(&org_url, self.project).await?;
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

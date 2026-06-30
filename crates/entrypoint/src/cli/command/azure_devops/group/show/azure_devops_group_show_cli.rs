use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::fetch_azure_devops_groups_for_project;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use eyre::bail;
use std::io::stdout;

/// Show Azure DevOps group details.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsGroupShowArgs {
    /// Project id or project name.
    #[facet(figue::named, opaque, proxy = String)]
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Group identifier (display name, principal name, origin id, or descriptor).
    #[facet(figue::named)]
    pub group: String,
}

impl AzureDevOpsGroupShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
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

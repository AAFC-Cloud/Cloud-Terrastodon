use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_groups;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use eyre::bail;
use serde_json::to_writer_pretty;
use std::io::stdout;

/// Show Azure DevOps group details.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsGroupShowArgs {
    /// Project id or project name.
    pub project: AzureDevOpsProjectArgument<'static>,

    /// Group identifier (display name, principal name, origin id, or descriptor).
    pub group: String,
}

impl AzureDevOpsGroupShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let groups = fetch_azure_devops_groups(&org_url, self.project).await?;
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

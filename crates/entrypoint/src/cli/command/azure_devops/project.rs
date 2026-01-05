use clap::Args;
use clap::Subcommand;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use eyre::Result;
use eyre::bail;
use serde_json::to_writer_pretty;
use std::io::Write;
use std::io::stdout;

/// Azure DevOps project-related commands.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsProjectArgs {
    #[command(subcommand)]
    pub command: AzureDevOpsProjectCommand,
}

#[derive(Subcommand, Debug, Clone)]
pub enum AzureDevOpsProjectCommand {
    /// List Azure DevOps projects in the organization.
    List,
    /// Show details for a single Azure DevOps project by id or name.
    Show {
        /// Project id (UUID) or project name.
        identifier: String,
    },
}

impl AzureDevOpsProjectArgs {
    pub async fn invoke(self) -> Result<()> {
        match self.command {
            AzureDevOpsProjectCommand::List => {
                let org_url = get_default_organization_url().await?;
                let projects = fetch_all_azure_devops_projects(&org_url).await?;
                let mut out = stdout().lock();
                to_writer_pretty(&mut out, &projects)?;
                out.write_all(b"\n")?;
            }
            AzureDevOpsProjectCommand::Show { identifier } => {
                let org_url = get_default_organization_url().await?;
                let projects = fetch_all_azure_devops_projects(&org_url).await?;

                // Parse the argument (must be a valid id or name) and find the project.
                let arg: AzureDevOpsProjectArgument<'static> = identifier.parse()?;
                let maybe = projects.into_iter().find(|p| arg.matches(p));

                if let Some(project) = maybe {
                    to_writer_pretty(stdout(), &project)?;
                } else {
                    bail!("No project found matching '{}'.", identifier);
                }
            }
        }

        Ok(())
    }
}

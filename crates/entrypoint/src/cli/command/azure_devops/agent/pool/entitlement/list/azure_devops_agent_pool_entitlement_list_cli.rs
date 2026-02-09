use clap::Args;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsAgentPoolArgument;
use cloud_terrastodon_azure_devops::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_agent_pool_entitlements_for_pool;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_agent_pool_entitlements_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_command::ParallelFallibleWorkQueue;
use eyre::Result;
use serde_json::to_writer_pretty;
use std::io::stdout;
use tracing::info;

/// List Azure DevOps agent pool entitlements (queues) in a project.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentPoolEntitlementListArgs {
    /// Project id or project name.
    #[arg(long)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
    #[arg(long)]
    pub pool: Option<AzureDevOpsAgentPoolArgument<'static>>,
}

impl AzureDevOpsAgentPoolEntitlementListArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
        match (self.project, self.pool) {
            (None, None) => {
                // Print the entitlements for all pools and projects by enumerating projects and pools
                info!("Fetching projects...");
                let projects = fetch_all_azure_devops_projects(&org_url).await?;
                let mut entitlements = Vec::new();
                let mut work =
                    ParallelFallibleWorkQueue::new("fetching agent pool entitlements", 8);
                for project in projects {
                    let org_url = org_url.clone();
                    let project_id = project.id.clone();
                    work.enqueue(async move {
                        let project_entitlements =
                            fetch_azure_devops_agent_pool_entitlements_for_project(
                                &org_url, project_id,
                            )
                            .await?;
                        Ok(project_entitlements)
                    });
                }
                let results = work.join().await?;
                for project_entitlements in results.into_iter() {
                    entitlements.extend(project_entitlements);
                }
                to_writer_pretty(stdout(), &entitlements)?;
            }
            (Some(project), Some(pool)) => {
                // Print the entitlements for the project that match the pool
                let entitlements =
                    fetch_azure_devops_agent_pool_entitlements_for_project(&org_url, project)
                        .await?;
                let entitlements: Vec<_> = entitlements
                    .into_iter()
                    .filter(|e| pool.matches_entitlement(e))
                    .collect();
                to_writer_pretty(stdout(), &entitlements)?;
            }
            (Some(project), None) => {
                // Print the entitlements for this project
                let entitlements =
                    fetch_azure_devops_agent_pool_entitlements_for_project(&org_url, project)
                        .await?;
                to_writer_pretty(stdout(), &entitlements)?;
            }
            (None, Some(pool)) => {
                // Print the entitlements for this pool by enumerating projects
                let entitlements =
                    fetch_azure_devops_agent_pool_entitlements_for_pool(&org_url, pool).await?;
                to_writer_pretty(stdout(), &entitlements)?;
            }
        }
        Ok(())
    }
}

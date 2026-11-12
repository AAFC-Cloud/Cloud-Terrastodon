use cloud_terrastodon_azure_devops::AzureDevOpsAgentPool;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPoolId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProject;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
use std::collections::HashMap;
use std::collections::HashSet;
use std::io::Write;
use std::sync::Arc;

/// Pool memberships and the project inventory used to interpret them.
#[derive(Facet)]
pub struct AzureDevOpsAgentPoolSummary {
    pub org: AzureDevOpsOrganizationUrl,
    pub pools: Arc<Vec<AzureDevOpsAgentPool>>,
    pub projects: Arc<Vec<Arc<AzureDevOpsProject>>>,
    pub pool_projects: HashMap<AzureDevOpsAgentPoolId, HashSet<AzureDevOpsProjectId>>,
}

impl AzureDevOpsAgentPoolSummary {
    pub fn write_text(
        &self,
        writer: &mut dyn Write,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        writeln!(writer, "Found {} agent pools", self.pools.len())?;
        for pool in self.pools.iter() {
            cancellation.bail_if_cancelled()?;
            writeln!(writer, "────────────────────────────────────────")?;
            writeln!(writer, "Pool: {}", pool.name)?;
            writeln!(writer, "Pool ID: {}", pool.id)?;
            let Some(included) = self
                .pool_projects
                .get(&pool.id)
                .filter(|ids| !ids.is_empty())
            else {
                writeln!(writer, "Projects: None\n")?;
                continue;
            };
            if included.len() == self.projects.len() {
                writeln!(writer, "Projects: ALL\n")?;
                continue;
            }
            let show_excluded = self.projects.len().saturating_sub(included.len()) <= 5;
            let mut projects = self
                .projects
                .iter()
                .filter(|project| included.contains(&project.id) != show_excluded)
                .collect::<Vec<_>>();
            projects.sort_by(|left, right| left.name.cmp(&right.name));
            writeln!(
                writer,
                "{}Projects ({}):",
                if show_excluded { "Excluded " } else { "" },
                projects.len()
            )?;
            for project in projects {
                writeln!(writer, "  - {} ({})", project.name, project.id)?;
            }
            writeln!(writer)?;
        }
        Ok(())
    }
}

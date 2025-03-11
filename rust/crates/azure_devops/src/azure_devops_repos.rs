use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevOpsProjectId;
use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevOpsRepo;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn fetch_all_azure_devops_repos_for_project(
    project_id: &AzureDevOpsProjectId,
) -> Result<Vec<AzureDevOpsRepo>> {
    info!("Fetching repos for project {project_id:?}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "repos",
        "list",
        "--output",
        "json",
        "--project",
        project_id.to_string().as_ref(),
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "repos", "list", project_id.to_string().replace(" ","_").as_ref()]),
        valid_for: Duration::from_hours(8),
    });
    let repos: Vec<AzureDevOpsRepo> = cmd.run().await?;
    info!("Found {} repos", repos.len());
    Ok(repos)
}

#[cfg(test)]
mod tests {
    use crate::prelude::fetch_all_azure_devops_projects;

    use super::*;

    #[tokio::test]
    async fn test_fetch_all_azure_devops_repos() -> Result<()> {
        let projects = fetch_all_azure_devops_projects().await?;
        let mut repo_counts = Vec::new();
        for project in projects.into_iter().take(5) {
            let repos = fetch_all_azure_devops_repos_for_project(&project.id).await?;
            for repo in repos.iter().take(3) {
                println!("Found project {} repo: {}", project.name, repo.name);
            }
            repo_counts.push(repos.len());
        }
        assert!(repo_counts.into_iter().sum::<usize>() > 0);
        Ok(())
    }
}

use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevOpsProjectId;
use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevOpsRepo;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Context;
use eyre::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::info;

pub async fn fetch_all_azure_devops_repos_for_project(
    project_id: &AzureDevOpsProjectId,
) -> Result<Vec<AzureDevOpsRepo>> {
    debug!("Fetching repos for project {project_id:?}");
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
        path: PathBuf::from_iter([
            "az",
            "repos",
            "list",
            project_id.to_string().replace(" ", "_").as_ref(),
        ]),
        valid_for: Duration::from_hours(8),
    });
    let repos: Vec<AzureDevOpsRepo> = cmd.run().await?;
    debug!("Found {} repos for {project_id:?}", repos.len());
    Ok(repos)
}

pub async fn fetch_azure_devops_repos_batch(
    project_ids: Vec<AzureDevOpsProjectId>,
) -> Result<HashMap<AzureDevOpsProjectId, Vec<AzureDevOpsRepo>>> {
    info!("Fetching repos for {} projects", project_ids.len());
    let mut rtn: HashMap<AzureDevOpsProjectId, Vec<AzureDevOpsRepo>> = HashMap::new();
    let mut set = JoinSet::new();
    let project_count = project_ids.len();
    for project_id in project_ids {
        set.spawn(async move {
            let repos = fetch_all_azure_devops_repos_for_project(&project_id).await;
            (project_id, repos)
        });
    }
    while let Some(res) = set.join_next().await {
        let (project_id, repos) = res?;
        let repos = repos.wrap_err(format!("Fetching repos for project {project_id:?}"))?;
        rtn.insert(project_id, repos);
    }
    info!(
        "Found {} repos across {} projects",
        rtn.values().map(|x| x.len()).sum::<usize>(),
        project_count
    );
    Ok(rtn)
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

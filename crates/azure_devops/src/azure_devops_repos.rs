use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectId;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsRepo;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Context;
use eyre::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::debug;
use tracing::info;

pub async fn fetch_all_azure_devops_repos_for_project(
    org_url: &AzureDevOpsOrganizationUrl,
    project_id: &AzureDevOpsProjectId,
) -> Result<Vec<AzureDevOpsRepo>> {
    debug!("Fetching repos for project {project_id:?}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "repos",
        "list",
        "--organization",
        org_url.to_string().as_ref(),
        "--output",
        "json",
        "--project",
        project_id.to_string().as_ref(),
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "repos", "list", project_id.to_string().as_ref()]),
        valid_for: Duration::from_hours(8),
    });
    let repos: Vec<AzureDevOpsRepo> = cmd.run().await?;
    debug!("Found {} repos for {project_id:?}", repos.len());
    Ok(repos)
}

pub async fn fetch_azure_devops_repos_batch(
    org_url: &AzureDevOpsOrganizationUrl,
    project_ids: Vec<AzureDevOpsProjectId>,
) -> Result<HashMap<AzureDevOpsProjectId, Vec<AzureDevOpsRepo>>> {
    info!("Fetching repos for {} projects", project_ids.len());
    let mut rtn: HashMap<AzureDevOpsProjectId, Vec<AzureDevOpsRepo>> = HashMap::new();
    let mut set = JoinSet::new();
    let project_count = project_ids.len();
    for project_id in project_ids {
        let org_url = org_url.clone();
        set.spawn(async move {
            let repos = fetch_all_azure_devops_repos_for_project(&org_url, &project_id).await;
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
    use super::*;
    use crate::prelude::{fetch_all_azure_devops_projects, get_default_organization_url};

    #[tokio::test]
    async fn test_fetch_all_azure_devops_repos() -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        let mut repo_counts = Vec::new();
        for project in projects.into_iter().take(5) {
            let repos = fetch_all_azure_devops_repos_for_project(&org_url, &project.id).await?;
            for repo in repos.iter().take(3) {
                println!("Found project {} repo: {}", project.name, repo.name);
            }
            repo_counts.push(repos.len());
        }
        assert!(repo_counts.into_iter().sum::<usize>() > 0);
        Ok(())
    }
}

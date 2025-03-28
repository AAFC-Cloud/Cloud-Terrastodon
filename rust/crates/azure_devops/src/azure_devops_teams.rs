use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevOpsProjectId;
use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevopsTeam;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Context;
use eyre::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::task::JoinSet;
use tracing::info;

pub async fn fetch_azure_devops_teams_for_project(
    project_id: &AzureDevOpsProjectId,
) -> Result<Vec<AzureDevopsTeam>> {
    info!("Fetching Azure DevOps teams for project {project_id}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "team",
        "list",
        "--output",
        "json",
        "--project",
        &project_id.to_string(),
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "devops",
            "team",
            "list",
            "--project",
            &project_id.to_string(),
        ]),
        valid_for: Duration::from_hours(8),
    });

    let response = cmd.run::<Vec<AzureDevopsTeam>>().await?;
    info!(
        "Found {} Azure DevOps teams for project {project_id}",
        response.len()
    );
    Ok(response)
}

pub async fn fetch_azure_devops_teams_batch(
    project_ids: Vec<AzureDevOpsProjectId>,
) -> Result<HashMap<AzureDevOpsProjectId, Vec<AzureDevopsTeam>>> {
    info!("Fetching teams for {} projects", project_ids.len());
    let mut rtn: HashMap<AzureDevOpsProjectId, Vec<AzureDevopsTeam>> = HashMap::new();
    let mut set = JoinSet::new();
    let project_count = project_ids.len();
    for project_id in project_ids {
        set.spawn(async move {
            let teams = fetch_azure_devops_teams_for_project(&project_id).await;
            (project_id, teams)
        });
    }
    while let Some(res) = set.join_next().await {
        let (project_id, repos) = res?;
        let teams = repos.wrap_err(format!("Fetching repos for project {project_id:?}"))?;
        rtn.insert(project_id, teams);
    }
    info!(
        "Found {} teams across {} projects",
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
    async fn it_works() -> Result<()> {
        let project = fetch_all_azure_devops_projects()
            .await?
            .into_iter()
            .next()
            .unwrap();
        let results = fetch_azure_devops_teams_for_project(&project.id).await?;
        assert!(results.len() > 0);
        for value in results.iter().take(5) {
            println!("Found value: {value:#?}");
        }
        Ok(())
    }
}

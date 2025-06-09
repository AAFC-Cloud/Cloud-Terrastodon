use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsTeam;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn fetch_azure_devops_teams_for_project(
    project: impl Into<AzureDevOpsProjectArgument<'_>>,
) -> Result<Vec<AzureDevOpsTeam>> {
    let project: AzureDevOpsProjectArgument = project.into();
    info!("Fetching Azure DevOps teams for project {project}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "team",
        "list",
        "--output",
        "json",
        "--project",
        &project.to_string(),
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "devops",
            "team",
            "list",
            "--project",
            &project.to_string(),
        ]),
        valid_for: Duration::from_hours(8),
    });

    let response = cmd.run::<Vec<AzureDevOpsTeam>>().await?;
    info!(
        "Found {} Azure DevOps teams for project {project}",
        response.len()
    );
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::fetch_all_azure_devops_projects;

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

use std::path::PathBuf;
use std::time::Duration;

use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevopsProject;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use tracing::field::debug;
use tracing::info;

pub async fn fetch_all_azure_devops_projects() -> Result<Vec<AzureDevopsProject>> {
    info!("Fetching Azure DevOps projects");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "project", "list", "--output", "json"]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "devops", "project", "list"]),
        valid_for: Duration::from_hours(8),
    });

    #[derive(Serialize, Deserialize)]
    pub struct Response {
        #[serde(rename = "continuationToken")]
        continuation_token: Option<String>,
        value: Vec<AzureDevopsProject>,
    }

    let mut projects = Vec::new();
    let mut response = cmd.run::<Response>().await?;
    projects.extend(response.value);

    while let Some(continuation) = &response.continuation_token {
        debug("Fetching the next page of projects");
        let mut next_page_cmd = cmd.clone();
        next_page_cmd.args(["--continuation-token", continuation.as_ref()]);

        response = next_page_cmd.run::<Response>().await?;
        projects.extend(response.value);
    }

    info!("Found {} Azure DevOps projects", projects.len());
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetch_all_azure_devops_projects() -> Result<()> {
        let projects = fetch_all_azure_devops_projects().await?;
        assert!(projects.len() > 0);
        for project in projects.iter().take(5) {
            println!("Found project: {project:#?}");
        }
        Ok(())
    }
}

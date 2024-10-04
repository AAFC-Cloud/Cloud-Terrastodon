use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevopsProject;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use serde::Deserialize;
use serde::Serialize;

pub async fn fetch_all_azure_devops_projects() -> Result<Vec<AzureDevopsProject>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "project", "list", "--output", "json"]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from("az devops project list"),
        valid_for: Duration::from_hours(8),
    });

    #[derive(Serialize, Deserialize)]
    pub struct Response {
        #[serde(rename="continuationToken")]
        continuation_token: Option<String>,
        value: Vec<AzureDevopsProject>,
    }

    let mut rtn = Vec::new();
    let mut response = cmd.run::<Response>().await?;
    rtn.extend(response.value);

    while let Some(continuation) = &response.continuation_token {
        let mut next_page_cmd = cmd.clone();
        next_page_cmd.args(["--continuation-token", continuation.as_ref()]);

        response = next_page_cmd.run::<Response>().await?;
        rtn.extend(response.value);
    }

    Ok(rtn)
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
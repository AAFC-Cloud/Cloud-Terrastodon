use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProject;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;
use tracing::field::debug;

pub async fn fetch_all_azure_devops_projects(
    org_url: &AzureDevOpsOrganizationUrl,
) -> Result<Vec<AzureDevOpsProject>> {
    debug!("Fetching Azure DevOps projects");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "project",
        "list",
        "--organization",
        org_url.to_string().as_str(),
        "--output",
        "json",
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "devops", "project", "list"]),
        valid_for: Duration::from_hours(8),
    });

    #[derive(Serialize, Deserialize)]
    pub struct Response {
        #[serde(rename = "continuationToken")]
        continuation_token: Option<String>,
        value: Vec<AzureDevOpsProject>,
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

    debug!("Found {} Azure DevOps projects", projects.len());
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    async fn test_fetch_all_azure_devops_projects() -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let projects = fetch_all_azure_devops_projects(&org_url).await?;
        assert!(!projects.is_empty());
        for project in projects.iter().take(5) {
            println!("Found project: {project:#?}");
        }
        Ok(())
    }
}

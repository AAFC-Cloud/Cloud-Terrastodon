use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsTeam;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

pub async fn fetch_azure_devops_teams_for_project(
    org_url: &AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'_>>,
) -> Result<Vec<AzureDevOpsTeam>> {
    let project: AzureDevOpsProjectArgument = project.into();
    debug!("Fetching Azure DevOps teams for project {project}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "team",
        "list",
        "--organization",
        org_url.to_string().as_str(),
        "--output",
        "json",
        "--project",
        &project.to_string(),
    ]);
    cmd.use_cache_behaviour(Some(CacheKey::new(PathBuf::from_iter([
        "az",
        "devops",
        "team",
        "list",
        "--project",
        &project.to_string(),
    ]))));

    let response = cmd.run::<Vec<AzureDevOpsTeam>>().await?;
    debug!(
        "Found {} Azure DevOps teams for project {project}",
        response.len()
    );
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let org_url = get_default_organization_url().await?;
        let project = fetch_all_azure_devops_projects(&org_url)
            .await?
            .into_iter()
            .next()
            .unwrap();
        let results = fetch_azure_devops_teams_for_project(&org_url, &project.id).await?;
        assert!(!results.is_empty());
        for value in results.iter().take(5) {
            println!("Found value: {value:#?}");
        }
        Ok(())
    }
}

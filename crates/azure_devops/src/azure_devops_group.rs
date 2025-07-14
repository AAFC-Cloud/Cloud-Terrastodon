use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsGroup;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_azure_devops_groups(
    project: impl Into<AzureDevOpsProjectArgument<'_>>,
) -> eyre::Result<Vec<AzureDevOpsGroup>> {
    let project: AzureDevOpsProjectArgument = project.into();
    debug!("Fetching Azure DevOps groups for project {project}");

    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "devops",
        "security",
        "group",
        "list",
        "--project",
        &project.to_string(),
        "--output",
        "json",
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "devops",
            "security",
            "group",
            "list",
            "--project",
            &project.to_string(),
        ]),
        valid_for: Duration::from_hours(8),
    });

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Response {
        continuation_token: Option<String>,
        graph_groups: Vec<AzureDevOpsGroup>,
    }

    let response = cmd.run::<Response>().await?;
    assert!(
        response.continuation_token.is_none(),
        "Continuation token found in Azure DevOps group list response"
    );

    debug!(
        "Found {} Azure DevOps groups for project {}",
        response.graph_groups.len(),
        project
    );
    Ok(response.graph_groups)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::fetch_azure_devops_groups;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let project = fetch_all_azure_devops_projects()
            .await?
            .into_iter()
            .next()
            .expect("No Azure DevOps projects found");
        let groups = fetch_azure_devops_groups(&project).await?;
        assert!(
            !groups.is_empty(),
            "Expected at least one Azure DevOps group"
        );
        for group in &groups {
            println!("Group: {:#?}", group);
        }
        Ok(())
    }
}

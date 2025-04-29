use std::path::PathBuf;
use std::time::Duration;

use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_command::prelude::CacheBehaviour;
use cloud_terrastodon_command::prelude::CommandBuilder;
use cloud_terrastodon_command::prelude::CommandKind;
use serde::Deserialize;
use serde_json::Value;
use tracing::info;

pub async fn fetch_queries_for_project(
    project_name: &AzureDevOpsProjectName,
) -> eyre::Result<Vec<AzureDevOpsWorkItemQuery>> {
    info!("Fetching queries for Azure DevOps project {project_name}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "invoke"]);
    cmd.args(["--area", "wit"]);
    cmd.args(["--resource", "queries"]);
    cmd.args(["--encoding", "utf-8"]);
    cmd.args([
        "--route-parameters",
        format!("project={project_name}").as_str(),
    ]);
    cmd.args(["--query-parameters", "$expand=all", "$depth=2"]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "devops",
            "query",
            "list",
            project_name.replace(" ", "_").as_ref(),
        ]),
        valid_for: Duration::from_hours(8),
    });
    #[derive(Deserialize)]
    struct InvokeResponse {
        continuation_token: Option<Value>,
        count: u32,
        value: Vec<AzureDevOpsWorkItemQuery>,
    }
    let resp = cmd.run::<InvokeResponse>().await?;
    let queries = resp.value;
    let total = AzureDevOpsWorkItemQuery::flatten_many(&queries).len();
    info!(
        "Found {} queries for Azure DevOps project {project_name} ({} counting children)",
        resp.count, total
    );
    if resp.continuation_token.is_some() {
        todo!("Add support for continuation token...");
    }
    Ok(queries)
}

#[cfg(test)]
mod test {
    use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQuery;

    use crate::prelude::fetch_queries_for_project;
    use crate::prelude::get_default_project_name;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let project_name = get_default_project_name().await?;
        println!("Fetching queries for {project_name:?}");
        let queries = fetch_queries_for_project(&project_name).await?;
        for entry in AzureDevOpsWorkItemQuery::flatten_many(&queries) {
            println!(
                "{}{} ({})",
                ".".repeat(entry.parents.len()),
                entry.child.name,
                if entry.child.is_folder {
                    "folder"
                } else {
                    "query"
                }
            );
        }
        Ok(())
    }
}

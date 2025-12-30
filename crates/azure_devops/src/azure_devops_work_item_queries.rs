use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::info;

pub async fn fetch_queries_for_project(
    org_url: &AzureDevOpsOrganizationUrl,
    project_name: &AzureDevOpsProjectName,
) -> eyre::Result<Vec<AzureDevOpsWorkItemQuery>> {
    info!("Fetching queries for Azure DevOps project {project_name}");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["devops", "invoke"]);
    cmd.args(["--organization", org_url.to_string().as_str()]);
    cmd.args(["--area", "wit"]);
    cmd.args(["--resource", "queries"]);
    cmd.args(["--encoding", "utf-8"]);
    cmd.args([
        "--route-parameters",
        format!("project={project_name}").as_str(),
    ]);
    cmd.args(["--query-parameters", "$expand=all", "$depth=2"]);
    cmd.use_cache_behaviour(Some(CacheKey::new(PathBuf::from_iter([
        "az",
        "devops",
        "query",
        "list",
        project_name.as_ref(),
    ]))));
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
    use crate::prelude::fetch_queries_for_project;
    use crate::prelude::get_default_organization_url;
    use crate::prelude::get_default_project_name;
    use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQuery;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let org_url = get_default_organization_url().await?;
        let project_name = get_default_project_name().await?;
        println!("Fetching queries for {project_name:?}");
        let queries = fetch_queries_for_project(&org_url, &project_name).await?;
        for entry in AzureDevOpsWorkItemQuery::flatten_many(&queries) {
            println!(
                "{}{} ({}) ({})",
                ".".repeat(entry.parents.len()),
                entry.child.name,
                if entry.child.is_folder {
                    "folder"
                } else {
                    "query"
                },
                entry.child.id
            );
        }
        Ok(())
    }
}

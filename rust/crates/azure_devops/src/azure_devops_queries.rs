use cloud_terrastodon_core_azure_devops_types::prelude::AzureDevopsWorkItemQuery;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use serde::Deserialize;
use serde_json::Value;
use tracing::info;

pub async fn fetch_queries_for_project(
    project_name: &str,
) -> eyre::Result<Vec<AzureDevopsWorkItemQuery>> {
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
    cmd.args(["--query-parameters", "$expand=all","$depth=2"]);
    #[derive(Deserialize)]
    struct InvokeResponse {
        continuation_token: Option<Value>,
        count: u32,
        value: Vec<AzureDevopsWorkItemQuery>,
    }
    let resp = cmd.run::<InvokeResponse>().await?;
    info!("Found {} queries for Azure DevOps project {project_name}", resp.count);
    if resp.continuation_token.is_some() {
        todo!("Add support for continuation token...");
    }
    Ok(resp.value)
}

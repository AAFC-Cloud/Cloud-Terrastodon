use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use tracing::info;

pub struct WorkItemQueriesForProjectRequest<'a> {
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: AzureDevOpsProjectArgument<'a>,
}

pub fn fetch_queries_for_project<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project: impl Into<AzureDevOpsProjectArgument<'a>>,
) -> WorkItemQueriesForProjectRequest<'a> {
    WorkItemQueriesForProjectRequest {
        org_url,
        project: project.into(),
    }
}

#[async_trait]
impl<'a> CacheableCommand for WorkItemQueriesForProjectRequest<'a> {
    type Output = Vec<AzureDevOpsWorkItemQuery>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            "query",
            "list",
            self.project.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        info!("Fetching queries for Azure DevOps project {}", self.project);
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["devops", "invoke"]);
        cmd.args(["--organization", self.org_url.to_string().as_str()]);
        cmd.args(["--area", "wit"]);
        cmd.args(["--resource", "queries"]);
        cmd.args(["--encoding", "utf-8"]);
        cmd.args([
            "--route-parameters",
            format!("project={}", self.project).as_str(),
        ]);
        cmd.args(["--query-parameters", "$expand=all", "$depth=2"]);
        cmd.cache(self.cache_key());

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
            "Found {} queries for Azure DevOps project {} ({} counting children)",
            resp.count, self.project, total
        );
        if resp.continuation_token.is_some() {
            todo!("Add support for continuation token...");
        }
        Ok(queries)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(WorkItemQueriesForProjectRequest<'a>, 'a);

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

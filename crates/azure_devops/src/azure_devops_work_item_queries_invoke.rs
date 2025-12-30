#![allow(deprecated)]
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQueryId;
use cloud_terrastodon_azure_devops_types::prelude::WorkItemQueryResult;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::bstr::ByteSlice;
use std::path::PathBuf;
use tracing::debug;

/// <https://developercommunity.visualstudio.com/t/Its-impossible-to-use-az-devops-invoke/10880749>
#[deprecated(note = "WIP, the return type and behaviour isn't in a good spot yet.")]
pub struct WorkItemsForQueryRequest<'a> {
    org_url: &'a AzureDevOpsOrganizationUrl,
    project_name: &'a AzureDevOpsProjectName,
    query_id: &'a AzureDevOpsWorkItemQueryId,
}

pub fn fetch_work_items_for_query<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    project_name: &'a AzureDevOpsProjectName,
    query_id: &'a AzureDevOpsWorkItemQueryId,
) -> WorkItemsForQueryRequest<'a> {
    WorkItemsForQueryRequest {
        org_url,
        project_name,
        query_id,
    }
}

#[async_trait]
impl<'a> CacheableCommand for WorkItemsForQueryRequest<'a> {
    type Output = Option<WorkItemQueryResult>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "boards",
            "query",
            &self.query_id.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(
            "Fetching work item query results for {} from project {} in organization {}",
            self.query_id, self.project_name, self.org_url.organization_name
        );

        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "boards",
            "query",
            "--organization",
            self.org_url.to_string().as_str(),
            "--id",
            self.query_id.to_string().as_str(),
            "--output",
            "json",
        ]);
        cmd.cache(CacheKey::new(PathBuf::from_iter([
            "az",
            "boards",
            "query",
            &self.query_id.to_string(),
        ])));
        let output = cmd.run_raw().await?;
        if output.stdout.trim().is_empty() {
            return Ok(None);
        }
        let mut rtn: Vec<WorkItemQueryResult> = output.try_interpret(&cmd).await?;
        assert_eq!(rtn.len(), 1);
        Ok(Some(rtn.remove(0)))

        // let url = format!(
        //     "https://dev.azure.com/{org_name}/{project_name}/_apis/wit/wiql/{query_id}?api-version=7.1"
        // );
        // debug!("Fetching work item query results from {url}");
        // let client = create_azure_devops_rest_client().await?;
        // let resp = client.get(&url).send().await?.error_for_status()?;
        // let rtn: WorkItemQueryResult = resp.json().await?;
        // Ok(Some(rtn))
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(WorkItemsForQueryRequest<'a>, 'a);

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::fetch_queries_for_project;
    use crate::prelude::fetch_work_items_for_query;
    use crate::prelude::get_default_organization_url;
    use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQuery;
    use eyre::Context;
    use eyre::bail;

    #[tokio::test]
    #[ignore]
    pub async fn it_works() -> eyre::Result<()> {
        // get all projects
        let org_url = get_default_organization_url().await?;
        let mut projects = fetch_all_azure_devops_projects(&org_url).await?;
        while let Some(project) = projects.pop() {
            // get queries for project
            let queries = fetch_queries_for_project(&org_url, &project.name).await?;

            // get query we can run
            let query = AzureDevOpsWorkItemQuery::flatten_many(&queries)
                .into_iter()
                .find(|x| !x.child.is_folder)
                .map(|x| x.child)
                .cloned();
            let Some(query) = query else {
                continue;
            };

            // get items from the query
            let items = fetch_work_items_for_query(&org_url, &project.name, &query.id)
                .await
                .wrap_err(format!(
                    "Failed to fetch work items for query {query} from project {project}",
                    query = query.name,
                    project = project.name
                ))?;
            println!(
                "Result for query {query} from project {project}",
                query = query.name,
                project = project.name
            );
            println!("{:#?}", items);

            // success if found some items
            if let Some(items) = items {
                if items.work_items.is_empty() {
                    continue;
                } else {
                    return Ok(());
                }
            }
        }
        bail!("Failed to find any work items");
    }
}

#![allow(deprecated)]
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQueryId;
use cloud_terrastodon_azure_devops_types::prelude::WorkItemQueryResult;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use tracing::debug;

pub struct WorkItemsForQueryRequest<'a> {
    pub org_url: &'a AzureDevOpsOrganizationUrl,
    pub query_id: &'a AzureDevOpsWorkItemQueryId,
}

pub fn fetch_work_items_for_query<'a>(
    org_url: &'a AzureDevOpsOrganizationUrl,
    query_id: &'a AzureDevOpsWorkItemQueryId,
) -> WorkItemsForQueryRequest<'a> {
    WorkItemsForQueryRequest { org_url, query_id }
}

#[async_trait]
impl<'a> CacheableCommand for WorkItemsForQueryRequest<'a> {
    type Output = Option<WorkItemQueryResult>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "devops",
            self.org_url.organization_name.as_ref(),
            "boards",
            "query",
            &self.query_id.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(
            "Fetching work item query results for {} from in organization {}",
            self.query_id, self.org_url.organization_name
        );
        // We have to use REST instead of `az devops invoke` because of https://developercommunity.visualstudio.com/t/Its-impossible-to-use-az-devops-invoke/10880749
        // There is also `az boards query --organization {} --id {}` but it has a different output format.
        // We want to hit the API to get the fields as described in https://learn.microsoft.com/en-us/rest/api/azure/devops/wit/wiql/query-by-wiql?view=azure-devops-rest-7.1&tabs=HTTP
        let url = format!(
            "{org_url}/_apis/wit/wiql/{query_id}?api-version=7.1",
            org_url = self.org_url,
            query_id = self.query_id,
        );

        let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
        cmd.cache(self.cache_key());
        cmd.args([
            "az",
            "devops",
            "rest",
            "--method",
            "GET",
            "--url",
            url.as_ref(),
        ]);
        Ok(cmd.run().await?)
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
            let items = fetch_work_items_for_query(&org_url, &query.id)
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

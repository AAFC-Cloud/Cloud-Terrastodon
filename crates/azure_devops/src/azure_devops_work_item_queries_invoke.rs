use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationName;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQueryId;
use cloud_terrastodon_azure_devops_types::prelude::WorkItemQueryResult;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::bstr::ByteSlice;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

#[deprecated(note = "WIP, the return type and behaviour isn't in a good spot yet.")]
pub async fn fetch_work_items_for_query(
    org_name: &AzureDevOpsOrganizationName,
    project_name: &AzureDevOpsProjectName,
    query_id: &AzureDevOpsWorkItemQueryId,
) -> eyre::Result<Option<WorkItemQueryResult>> {
    debug!(
        "Fetching work item query results for {query_id} from project {project_name} in organization {org_name}"
    );
    // https://developercommunity.visualstudio.com/t/Its-impossible-to-use-az-devops-invoke/10880749

    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "boards",
        "query",
        "--id",
        &query_id.to_string(),
        "--output",
        "json",
    ]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter(["az", "boards", "query", &query_id.to_string()]),
        valid_for: Duration::from_hours(8),
    });
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

#[cfg(test)]
#[allow(deprecated)]
mod test {
    use crate::prelude::fetch_all_azure_devops_projects;
    use crate::prelude::fetch_queries_for_project;
    use crate::prelude::fetch_work_items_for_query;
    use crate::prelude::get_default_organization_name;
    use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsWorkItemQuery;
    use eyre::Context;
    use eyre::bail;

    #[tokio::test]
    #[ignore]
    pub async fn it_works() -> eyre::Result<()> {
        // get all projects
        let org_name = get_default_organization_name().await?;
        let mut projects = fetch_all_azure_devops_projects().await?;
        while let Some(project) = projects.pop() {
            // get queries for project
            let queries = fetch_queries_for_project(&project.name).await?;

            // get query we can run
            let query = AzureDevOpsWorkItemQuery::flatten_many(&queries)
                .into_iter()
                .filter(|x| !x.child.is_folder)
                .next()
                .map(|x| x.child)
                .cloned();
            let Some(query) = query else {
                continue;
            };

            // get items from the query
            let items = fetch_work_items_for_query(&org_name, &project.name, &query.id)
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

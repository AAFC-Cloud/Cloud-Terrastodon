use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueriesForProjectRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use itertools::Itertools;
use std::borrow::Cow;
use tracing::info;

pub async fn dump_work_items(
    org_url: &AzureDevOpsOrganizationUrl,
    auth_selection: AzureDevOpsAuthSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    let projects = AzureDevOpsProjectsListRequest {
        org_url: Cow::Borrowed(org_url),
        auth_selection,
    }
    .execute(executor)
    .await?;
    let projects = PickerTui::<_>::new()
        .set_header("Pick the projects to export from")
        .pick_many(
            projects.iter().cloned().map(|p| Choice {
                key: p.name.to_string(),
                value: p,
            }),
            executor.cancellation(),
        )
        .await?;
    let mut queries = Vec::new();
    for proj in &projects {
        let found = AzureDevOpsWorkItemQueriesForProjectRequest {
            org_url: Cow::Borrowed(org_url),
            auth_selection,
            project: (&proj.name).into(),
        }
        .execute(executor)
        .await?;
        queries.push(found);
    }
    let queries =
        AzureDevOpsWorkItemQuery::flatten_many(queries.iter().flat_map(|page| page.iter()));
    let queries = PickerTui::<_>::new()
        .set_header("Pick the queries that return the items to export")
        .pick_many(
            queries
                .iter()
                .filter(|entry| !entry.child.is_folder)
                .map(|entry| Choice {
                    key: entry
                        .parents
                        .iter()
                        .chain(std::iter::once(&entry.child))
                        .map(|x| &x.name)
                        .join("/"),
                    value: &entry.child,
                }),
            executor.cancellation(),
        )
        .await?;
    info!("You chose {} queries", queries.len());

    Ok(())
}

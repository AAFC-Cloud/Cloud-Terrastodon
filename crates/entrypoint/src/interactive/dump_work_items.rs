use cloud_terrastodon_azure_devops::prelude::AzureDevOpsWorkItemQuery;
use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_queries_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use itertools::Itertools;
use tracing::info;

pub async fn dump_work_items() -> eyre::Result<()> {
    let org_url = get_default_organization_url().await?;
    let projects = fetch_all_azure_devops_projects(&org_url).await?;
    let projects = PickerTui::new()
        .set_header("Pick the projects to export from")
        .pick_many(projects.into_iter().map(|p| Choice {
            key: p.name.to_string(),
            value: p,
        }))?;
    let mut queries = Vec::new();
    for proj in &projects {
        let found = fetch_queries_for_project(&org_url, &proj.name).await?;
        queries.extend(found);
    }
    let queries = AzureDevOpsWorkItemQuery::flatten_many(&queries);
    let queries = PickerTui::new()
        .set_header("Pick the queries that return the items to export")
        .pick_many(
            queries
                .into_iter()
                .filter(|entry| !entry.child.is_folder)
                .map(|entry| Choice {
                    key: entry
                        .parents
                        .into_iter()
                        .chain(std::iter::once(entry.child))
                        .map(|x| &x.name)
                        .join("/"),
                    value: entry.child,
                }),
        )?;
    info!("You chose {} queries", queries.len());

    Ok(())
}

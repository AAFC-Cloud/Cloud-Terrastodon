use cloud_terrastodon_core_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_core_azure_devops::prelude::fetch_queries_for_project;
use cloud_terrastodon_core_azure_devops::prelude::flatten_queries;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use itertools::Itertools;
use tracing::info;

pub async fn dump_work_items() -> eyre::Result<()> {
    let projects = fetch_all_azure_devops_projects().await?;
    let projects = pick_many(FzfArgs {
        choices: projects
            .into_iter()
            .map(|p| Choice {
                key: p.name.to_string(),
                value: p,
            })
            .collect_vec(),

        header: Some("Pick the projects to export from".to_string()),
        ..Default::default()
    })?;
    let mut queries = Vec::new();
    for proj in &projects {
        let found = fetch_queries_for_project(&proj.name).await?;
        queries.extend(found);
    }
    let queries = flatten_queries(&queries);
    let queries = pick_many(FzfArgs {
        choices: queries
            .into_iter()
            .map(|(parents, query)| Choice {
                key: parents
                    .into_iter()
                    .chain(std::iter::once(query))
                    .map(|x| &x.name)
                    .join("/"),
                value: query,
            })
            .collect_vec(),
        header: Some("Pick the queries that return the items to export".to_string()),
        ..Default::default()
    })?;
    info!("You chose {} queries", queries.len());
    Ok(())
}

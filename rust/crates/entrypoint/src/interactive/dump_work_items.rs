use cloud_terrastodon_core_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_core_azure_devops::prelude::fetch_queries_for_project;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use itertools::Itertools;

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
        prompt: None,
        header: Some("Pick the projects to export from".to_string()),
    })?;
    let mut queries = Vec::new();
    for proj in &projects {
        let found = fetch_queries_for_project(&proj.name).await?;
        queries.extend(found);
    }
    for query in queries {
        println!("Found query: {}", query.name);
    }

    Ok(())
}

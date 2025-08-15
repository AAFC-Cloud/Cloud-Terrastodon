use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use eyre::Result;

pub async fn browse_azure_devops_projects() -> Result<()> {
    let org_url = get_default_organization_url().await?;
    let projects = fetch_all_azure_devops_projects(&org_url).await?;
    let chosen = pick_many(FzfArgs {
        choices: projects
            .into_iter()
            .map(|project| Choice {
                key: format!(
                    "{} {:64} - {}",
                    project.id,
                    project.name,
                    project.description.clone().unwrap_or_default()
                ),
                value: project,
            })
            .collect(),
        prompt: Some("Azure DevOps Projects: ".to_string()),
        ..Default::default()
    })?;

    println!("You chose:");
    println!(
        "{:#?}",
        chosen.into_iter().map(|x| x.value).collect::<Vec<_>>()
    );
    Ok(())
}

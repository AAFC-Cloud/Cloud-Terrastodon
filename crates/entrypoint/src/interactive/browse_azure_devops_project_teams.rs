use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick;
use cloud_terrastodon_user_input::pick_many;
use eyre::Result;

pub async fn browse_azure_devops_project_teams() -> Result<()> {
    let projects = fetch_all_azure_devops_projects().await?;
    let project = pick(FzfArgs {
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

    let teams = fetch_azure_devops_teams_for_project(&project.value).await?;
    let teams = pick_many(FzfArgs {
        choices: teams
            .into_iter()
            .map(|team| Choice {
                key: format!("{} {:64} - {}", team.id, team.name, team.description),
                value: team,
            })
            .collect(),
        prompt: Some("Azure DevOps Teams: ".to_string()),
        ..Default::default()
    })?;
    println!("{:#?}", teams);
    Ok(())
}

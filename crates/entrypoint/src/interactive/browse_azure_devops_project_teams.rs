use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;

pub async fn browse_azure_devops_project_teams() -> Result<()> {
    let org_url = get_default_organization_url().await?;
    let projects = fetch_all_azure_devops_projects(&org_url).await?;
    let project = PickerTui::new()
        .set_header("Azure DevOps Projects")
        .pick_one(projects.into_iter().map(|project| Choice {
            key: format!(
                "{} {:64} - {}",
                project.id,
                project.name,
                project.description.clone().unwrap_or_default()
            ),
            value: project,
        }))?;

    let teams = fetch_azure_devops_teams_for_project(&org_url, &project.id).await?;
    let teams = PickerTui::new()
        .set_header("Azure DevOps Teams")
        .pick_many(teams.into_iter().map(|team| Choice {
            key: format!("{} {:64} - {}", team.id, team.name, team.description),
            value: team,
        }))?;
    println!("{teams:#?}");
    Ok(())
}

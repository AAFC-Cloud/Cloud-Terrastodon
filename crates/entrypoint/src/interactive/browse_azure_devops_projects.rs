use cloud_terrastodon_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::prelude::get_default_organization_url;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;

pub async fn browse_azure_devops_projects() -> Result<()> {
    let org_url = get_default_organization_url().await?;
    let projects = fetch_all_azure_devops_projects(&org_url).await?;
    let chosen = PickerTui::from(projects.into_iter().map(|project| Choice {
        key: format!(
            "{} {:64} - {}",
            project.id,
            project.name,
            project.description.clone().unwrap_or_default()
        ),
        value: project,
    }))
    .set_header("Azure DevOps Projects")
    .pick_many()?;

    println!("You chose:");
    println!("{:#?}", chosen);
    Ok(())
}

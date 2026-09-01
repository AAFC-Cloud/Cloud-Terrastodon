use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_credentials::AzureDevOpsAuthContext;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;

pub async fn browse_azure_devops_projects(auth_context: &AzureDevOpsAuthContext) -> Result<()> {
    let org_url = get_default_organization_url().await?;
    let projects = fetch_all_azure_devops_projects(&org_url, auth_context).await?;
    let (chosen, maybe_error) = PickerTui::<_>::new()
        .set_header("Azure DevOps Projects")
        .pick_many(projects.into_iter().map(|project| Choice {
            key: format!(
                "{} {:64} - {}",
                project.id,
                project.name,
                project.description.clone().unwrap_or_default()
            ),
            value: project,
        }))
        .await
        .into_chosen_and_maybe_error()?;

    println!("You chose:");
    println!("{:#?}", chosen);
    maybe_error
}

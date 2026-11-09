use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;

pub async fn browse_azure_devops_projects(
    auth_context: &AzureDevOpsAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let cancellation = executor.cancellation();
    let org_url = get_default_organization_url(cancellation).await?;
    let projects =
        fetch_all_azure_devops_projects(&org_url, auth_context.selection()?, executor).await?;
    let (chosen, maybe_error) = PickerTui::<_>::new()
        .set_header("Azure DevOps Projects")
        .pick_many(
            projects.iter().cloned().map(|project| Choice {
                key: format!(
                    "{} {:64} - {}",
                    project.id,
                    project.name,
                    project.description.clone().unwrap_or_default()
                ),
                value: project,
            }),
            executor.cancellation(),
        )
        .await
        .into_chosen_and_maybe_error()?;

    println!("You chose:");
    println!("{:#?}", chosen);
    maybe_error
}

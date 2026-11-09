use cloud_terrastodon_auth::AzureDevOpsAuthContext;
use cloud_terrastodon_azure_devops::fetch_all_azure_devops_projects;
use cloud_terrastodon_azure_devops::fetch_azure_devops_teams_for_project;
use cloud_terrastodon_azure_devops::get_default_organization_url;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;

pub async fn browse_azure_devops_project_teams(
    auth_context: &AzureDevOpsAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let cancellation = executor.cancellation();
    let org_url = get_default_organization_url(cancellation).await?;
    let projects =
        fetch_all_azure_devops_projects(&org_url, auth_context.selection()?, executor).await?;
    let project = PickerTui::<_>::new()
        .set_header("Azure DevOps Projects")
        .pick_one(
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
        .await?;

    let teams =
        fetch_azure_devops_teams_for_project(&org_url, &project.id, auth_context, executor).await?;
    let (teams, maybe_error) = PickerTui::<_>::new()
        .set_header("Azure DevOps Teams")
        .pick_many(
            teams.into_iter().map(|team| Choice {
                key: format!("{} {:64} - {}", team.id, team.name, team.description),
                value: team,
            }),
            executor.cancellation(),
        )
        .await
        .into_chosen_and_maybe_error()?;
    println!("{teams:#?}");
    maybe_error
}

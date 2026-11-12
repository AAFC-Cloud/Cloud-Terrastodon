use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectsListRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsTeamsForProjectRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::borrow::Cow;

pub async fn browse_azure_devops_project_teams(
    org_url: &AzureDevOpsOrganizationUrl,
    auth_selection: AzureDevOpsAuthSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let projects = AzureDevOpsProjectsListRequest {
        org_url: Cow::Borrowed(org_url),
        auth_selection,
    }
    .execute(executor)
    .await?;
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

    let teams = AzureDevOpsTeamsForProjectRequest {
        org_url: Cow::Borrowed(org_url),
        auth_selection,
        project: (&project.id).into(),
    }
    .execute(executor)
    .await?;
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

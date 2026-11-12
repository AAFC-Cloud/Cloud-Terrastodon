use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::get_role_assignment_choices;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use std::sync::Arc;
use tracing::info;

pub async fn browse_role_assignments(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let choices = get_role_assignment_choices(authentication, executor).await?;

    info!("Picking");
    let (chosen, maybe_error): (Vec<Arc<RoleAssignment>>, _) = PickerTui::<_>::new()
        .set_header("Role assignments")
        .pick_many(choices, executor.cancellation())
        .await
        .into_chosen_and_maybe_error()?;

    info!("You chose:");
    for value in &chosen {
        info!("{:#?}", value);
    }
    maybe_error
}

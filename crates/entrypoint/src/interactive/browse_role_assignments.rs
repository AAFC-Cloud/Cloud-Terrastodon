use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::get_role_assignment_choices;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_role_assignments(auth_context: &AzureTenantAuthContext) -> Result<()> {
    let choices = get_role_assignment_choices(auth_context).await?;

    info!("Picking");
    let (chosen, maybe_error): (Vec<RoleAssignment>, _) = PickerTui::<_>::new()
        .set_header("Role assignments")
        .pick_many(choices)
        .await
        .into_chosen_and_maybe_error()?;

    info!("You chose:");
    for value in &chosen {
        info!("{:#?}", value);
    }
    maybe_error
}

use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::RoleAssignment;
use cloud_terrastodon_azure::get_role_assignment_choices;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_role_assignments(tenant_id: AzureTenantId) -> Result<()> {
    let choices = get_role_assignment_choices(tenant_id).await?;

    info!("Picking");
    let chosen: Vec<RoleAssignment> = PickerTui::new()
        .set_header("Role assignments")
        .pick_many(choices)?;

    info!("You chose:");
    for value in chosen {
        info!("{:#?}", value);
    }
    Ok(())
}

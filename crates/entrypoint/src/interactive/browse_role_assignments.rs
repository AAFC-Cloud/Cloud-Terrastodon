use cloud_terrastodon_azure::prelude::RoleAssignment;
use cloud_terrastodon_azure::prelude::get_role_assignment_choices;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_role_assignments() -> Result<()> {
    let choices = get_role_assignment_choices().await?;

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

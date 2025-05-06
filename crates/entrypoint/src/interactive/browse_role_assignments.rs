use cloud_terrastodon_azure::prelude::get_role_assignment_choices;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use eyre::Result;
use tracing::info;

pub async fn browse_role_assignments() -> Result<()> {
    let choices = get_role_assignment_choices().await?;

    info!("Picking");
    let chosen = pick_many(FzfArgs {
        choices,
        prompt: Some("Role assignments: ".to_string()),
        ..Default::default()
    })?;

    info!("You chose:");
    for choice in chosen {
        info!("{:#?}", choice.value);
    }
    Ok(())
}

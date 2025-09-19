use cloud_terrastodon_azure::prelude::get_resource_group_choices;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use tracing::info;

pub async fn browse_resource_groups() -> Result<()> {
    let choices = get_resource_group_choices().await?;

    info!("Prompting user");
    let chosen = PickerTui::new(choices)
        .set_header("Browsing resource groups")
        .pick_many()?;

    info!("You chose:");
    for (rg, sub) in chosen {
        info!("{} - {} - {}", rg.name.to_owned(), sub, rg.id);
    }

    Ok(())
}

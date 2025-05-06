use cloud_terrastodon_azure::prelude::get_resource_group_choices;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use eyre::Result;
use tracing::info;

pub async fn browse_resource_groups() -> Result<()> {
    let choices = get_resource_group_choices().await?;

    info!("Prompting user");
    let chosen = pick_many(FzfArgs {
        choices,
        header: Some("Browsing resource groups".to_string()),
        ..Default::default()
    })?;

    info!("You chose:");
    for rg in chosen {
        info!("{} - {} - {}", rg.0.name.to_owned(), rg.1, rg.0.id);
    }

    Ok(())
}

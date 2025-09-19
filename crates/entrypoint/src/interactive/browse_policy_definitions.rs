use cloud_terrastodon_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_user_input::PickerTui;
use itertools::Itertools;
use tracing::info;

pub async fn browse_policy_definitions() -> eyre::Result<()> {
    let policy_definitions = fetch_all_policy_definitions().await?;
    let chosen = PickerTui::new(policy_definitions).pick_many()?;
    let msg = format!(
        "You chose:\n{}",
        chosen.iter().map(|x| format!("- {x:#}")).join("\n")
    );
    info!("{msg}");

    Ok(())
}

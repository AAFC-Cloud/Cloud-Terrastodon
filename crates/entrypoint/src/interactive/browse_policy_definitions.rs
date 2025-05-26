use cloud_terrastodon_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick_many;
use itertools::Itertools;
use tracing::info;

pub async fn browse_policy_definitions() -> eyre::Result<()> {
    let policy_definitions = fetch_all_policy_definitions().await?;
    let chosen = pick_many(FzfArgs {
        choices: policy_definitions,
        ..Default::default()
    })?;
    let msg = format!(
        "You chose:\n{}",
        chosen.iter().map(|x| format!("- {x:#}")).join("\n")
    );
    info!("{msg}");

    Ok(())
}

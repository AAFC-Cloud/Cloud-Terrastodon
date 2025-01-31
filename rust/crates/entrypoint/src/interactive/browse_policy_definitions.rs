use cloud_terrastodon_core_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_core_user_input::prelude::pick_many;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
use itertools::Itertools;
use tracing::info;

pub async fn browse_policy_definitions() -> eyre::Result<()> {
    let policy_definitions = fetch_all_policy_definitions().await?;
    let chosen = pick_many(FzfArgs {
        choices: policy_definitions,
        prompt: None,
        header: None,
    })?;
    let msg = format!(
        "You chose:\n{}",
        chosen
            .iter()
            .map(|x| format!("- {:#}", x))
            .join("\n")
    );
    info!("{msg}");

    Ok(())
}

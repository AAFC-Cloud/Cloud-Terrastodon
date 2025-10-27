use cloud_terrastodon_azure::prelude::PolicyDefinition;
use cloud_terrastodon_azure::prelude::fetch_all_policy_definitions;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use itertools::Itertools;
use tracing::info;

pub async fn browse_policy_definitions() -> eyre::Result<()> {
    let policy_definitions = fetch_all_policy_definitions()
        .await?
        .into_iter()
        .map(|def| Choice {
            key: match def.description.as_ref() {
                Some(desc) => format!("{def} - {desc}"),
                None => format!("{def} - no description"),
            },
            value: def,
        });
    let chosen: Vec<PolicyDefinition> = PickerTui::new(policy_definitions).pick_many()?;
    let msg = format!(
        "You chose:\n{}",
        chosen.iter().map(|x| format!("- {}", x.id.expanded_form())).join("\n")
    );
    info!("{msg}");

    Ok(())
}

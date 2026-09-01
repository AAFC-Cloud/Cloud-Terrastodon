use cloud_terrastodon_azure::PolicyDefinition;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_policy_definitions;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use itertools::Itertools;
use tracing::info;

pub async fn browse_policy_definitions(auth_context: &AzureTenantAuthContext) -> eyre::Result<()> {
    let policy_definitions = fetch_all_policy_definitions(auth_context)
        .await?
        .into_iter()
        .map(|def| Choice {
            key: match def.description.as_ref() {
                Some(desc) => format!("{def} - {desc}"),
                None => format!("{def} - no description"),
            },
            value: def,
        });
    let (chosen, maybe_error): (Vec<PolicyDefinition>, _) = PickerTui::<_>::new()
        .pick_many(policy_definitions)
        .await
        .into_chosen_and_maybe_error()?;
    let msg = format!(
        "You chose:\n{}",
        chosen
            .iter()
            .map(|x| format!("- {}", x.id.expanded_form()))
            .join("\n")
    );
    info!("{msg}");

    maybe_error
}

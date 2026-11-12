use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzurePolicyDefinitionListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use itertools::Itertools;
use tracing::info;

pub async fn browse_policy_definitions(
    authentication: &AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    let policy_definitions = AzurePolicyDefinitionListRequest {
        authentication: *authentication,
    }
    .execute(executor)
    .await?
    .into_iter()
    .map(|def| Choice {
        key: match def.description.as_ref() {
            Some(desc) => format!("{def} - {desc}"),
            None => format!("{def} - no description"),
        },
        value: def,
    });
    let (chosen, maybe_error) = PickerTui::<_>::new()
        .pick_many(policy_definitions, executor.cancellation())
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

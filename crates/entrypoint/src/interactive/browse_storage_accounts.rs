use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_storage_accounts;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;

pub async fn browse_storage_accounts(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    let storage_accounts = fetch_all_storage_accounts(auth_context, executor).await?;
    let (chosen, maybe_error) = PickerTui::<_>::new()
        .pick_many(
            storage_accounts.into_iter().map(|storage_account| Choice {
                key: storage_account.id.expanded_form(),
                value: storage_account,
            }),
            executor.cancellation(),
        )
        .await
        .into_chosen_and_maybe_error()?;
    println!(
        "You chose: {}",
        facet_json::to_string_pretty(&chosen).map_err(|error| eyre::eyre!("{error:?}"))?
    );
    maybe_error
}

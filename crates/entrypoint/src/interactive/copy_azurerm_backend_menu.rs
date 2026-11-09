use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::SubscriptionName;
use cloud_terrastodon_azure::fetch_all_storage_accounts;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use cloud_terrastodon_azure::fetch_storage_account_blob_container_names;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;
use tokio::join;
use tracing::info;

pub async fn copy_azurerm_backend_menu(
    auth_context: &AzureTenantAuthContext,
    executor: &CloudTerrastodonExecutor,
) -> Result<()> {
    info!("Picking storage account");
    let chosen_storage_account = PickerTui::<_>::new()
        .set_header("Picking the storage account for the state file")
        .pick_one_reloadable(async |invalidate| {
            info!("Fetching storage accounts");
            info!("Fetching subscriptions");
            let (storage_accounts, subscriptions) = join!(
                fetch_all_storage_accounts(auth_context, executor).with_invalidation(invalidate),
                fetch_all_subscriptions(auth_context, executor).with_invalidation(invalidate)
            );
            let storage_accounts = storage_accounts?;
            let subscriptions = subscriptions?
                .into_iter()
                .map(|sub| (sub.id.to_owned(), sub))
                .collect::<HashMap<_, _>>();

            let unknown_subscription = SubscriptionName::try_new("Unknown Subscription")?;
            Ok(storage_accounts.into_iter().map(move |sa| {
                let sub_name = subscriptions
                    .get(&sa.id.resource_group_id.subscription_id)
                    .map(|sub| sub.name.to_owned())
                    .unwrap_or_else(|| unknown_subscription.clone());
                Choice {
                    key: format!(
                        "{:<32} {:<64} {}",
                        sub_name.to_string(),
                        sa.id.resource_group_id.resource_group_name.to_string(),
                        sa.name
                    ),
                    value: (sa, sub_name),
                }
            }))
        })
        .await?;

    info!("Fetching blob containers for {}", chosen_storage_account.1);
    let chosen_blob_container = PickerTui::<_>::new()
        .set_header("Blob Container Name")
        .pick_one_reloadable(async |invalidate| {
            let blob_container_names = fetch_storage_account_blob_container_names(
                &chosen_storage_account.0.id,
                auth_context,
                executor,
            )
            .with_invalidation(invalidate)
            .await?;

            if blob_container_names.is_empty() {
                bail!("No blob containers found in {}", chosen_storage_account.1);
            }

            Ok(blob_container_names.into_iter().map(|name| Choice {
                key: name.to_string(),
                value: name,
            }))
        })
        .await?;

    let output = format!(
        r#"
    resource_group_name  = "{}"
    storage_account_name = "{}"
    container_name       = "{}"
    subscription_id      = "{}" # {}
    "#,
        chosen_storage_account
            .0
            .id
            .resource_group_id
            .resource_group_name,
        chosen_storage_account.0.name,
        chosen_blob_container,
        chosen_storage_account
            .0
            .id
            .resource_group_id
            .subscription_id
            .short_form(),
        chosen_storage_account.1
    );

    info!("You chose:\n{output}");

    Ok(())
}

#[cfg(test)]
mod test {
    use cloud_terrastodon_azure::SubscriptionName;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        SubscriptionName::try_new("Unknown Subscription")?;
        Ok(())
    }
}

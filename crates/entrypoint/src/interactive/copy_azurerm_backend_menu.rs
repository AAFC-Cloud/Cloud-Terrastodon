use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::SubscriptionName;
use cloud_terrastodon_azure::fetch_all_storage_accounts;
use cloud_terrastodon_azure::fetch_all_subscriptions;
use cloud_terrastodon_command::CacheInvalidatableIntoFuture;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;
use tokio::join;
use tracing::info;

pub async fn copy_azurerm_backend_menu(tenant_id: AzureTenantId) -> Result<()> {
    info!("Picking storage account");
    let chosen_storage_account = PickerTui::<_>::new()
        .set_header("Picking the storage account for the state file")
        .pick_one_reloadable(|invalidate| async move {
            info!("Fetching storage accounts");
            info!("Fetching subscriptions");
            let (storage_accounts, subscriptions) = join!(
                fetch_all_storage_accounts(tenant_id).with_invalidation(invalidate),
                fetch_all_subscriptions(tenant_id).with_invalidation(invalidate)
            );
            let storage_accounts = storage_accounts?;
            let subscriptions = subscriptions?
                .into_iter()
                .map(|sub| (sub.id.to_owned(), sub))
                .collect::<HashMap<_, _>>();

            Ok(storage_accounts.into_iter().map(move |sa| {
                let sub_name = subscriptions
                    .get(&sa.id.resource_group_id.subscription_id)
                    .map(|sub| sub.name.to_owned())
                    .unwrap_or_else(|| SubscriptionName::try_new("Unknown Subscription").unwrap());
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
        .pick_one_reloadable(|_invalidate| {
            let chosen_storage_account = &chosen_storage_account;
            async move {
                let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
                cmd.args(["storage", "container", "list", "--account-name"]);
                cmd.arg(&*chosen_storage_account.0.name);
                cmd.arg("--subscription");
                cmd.arg(
                    chosen_storage_account
                        .0
                        .id
                        .resource_group_id
                        .subscription_id
                        .short_form(),
                );
                cmd.args(["--query", "[].name", "--output", "json"]);
                let blob_container_names = cmd.run::<Vec<String>>().await?;

                if blob_container_names.is_empty() {
                    bail!("No blob containers found in {}", chosen_storage_account.1);
                }

                Ok(blob_container_names.into_iter().map(|name| Choice {
                    key: name.clone(),
                    value: name,
                }))
            }
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

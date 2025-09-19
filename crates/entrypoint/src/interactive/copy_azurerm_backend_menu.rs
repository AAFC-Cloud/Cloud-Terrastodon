use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::StorageAccount;
use cloud_terrastodon_azure::prelude::SubscriptionName;
use cloud_terrastodon_azure::prelude::fetch_all_storage_accounts;
use cloud_terrastodon_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use eyre::bail;
use std::collections::HashMap;
use tokio::join;
use tracing::info;

pub async fn copy_azurerm_backend_menu() -> Result<()> {
    info!("Fetching storage accounts");
    info!("Fetching subscriptions");
    let (storage_accounts, subscriptions) =
        join!(fetch_all_storage_accounts(), fetch_all_subscriptions());
    let storage_accounts = storage_accounts?;
    let subscriptions = subscriptions?
        .into_iter()
        .map(|sub| (sub.id.to_owned(), sub))
        .collect::<HashMap<_, _>>();

    info!("Picking storage account");
    let chosen_storage_account = PickerTui::<(StorageAccount, String, SubscriptionName)>::new(
        storage_accounts.into_iter().map(|sa| {
            let sub_name = subscriptions
                .get(&sa.id.resource_group_id.subscription_id)
                .map(|sub| sub.name.to_owned())
                .unwrap_or_else(|| SubscriptionName::try_new("Unknown Subscription").unwrap());
            let key = format!(
                "{:32}\t{:64}\t{}",
                sub_name, sa.id.resource_group_id.resource_group_name, sa.name
            );
            let key_short = format!(
                "{} {} {}",
                sub_name, sa.id.resource_group_id.resource_group_name, sa.name
            );
            Choice {
                key,
                value: (sa, key_short, sub_name),
            }
        }),
    )
    .set_header("Picking the storage account for the state file")
    .pick_one()?;

    info!("Fetching blob containers for {}", chosen_storage_account.1);
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

    let chosen_blob_container = match blob_container_names.len() {
        0 => {
            bail!("No blob containers found in {}", chosen_storage_account.1);
        }
        1 => blob_container_names.first().unwrap(),
        _ => {
            info!("Picking blob container");
            &PickerTui::from(blob_container_names.into_iter().map(|name| Choice {
                key: name.to_owned(),
                value: name,
            }))
            .set_header("Blob Container Name")
            .pick_one()?
        }
    };

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
        chosen_storage_account.2
    );

    info!("You chose:\n{output}");

    Ok(())
}

#[cfg(test)]
mod test {
    use cloud_terrastodon_azure::prelude::SubscriptionName;

    #[test]
    pub fn it_works() -> eyre::Result<()> {
        SubscriptionName::try_new("Unknown Subscription")?;
        Ok(())
    }
}

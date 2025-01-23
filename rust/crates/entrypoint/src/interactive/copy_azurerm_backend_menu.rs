use std::collections::HashMap;

use eyre::bail;
use eyre::Result;
use cloud_terrastodon_core_azure::prelude::fetch_all_storage_accounts;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use cloud_terrastodon_core_user_input::prelude::pick;
use cloud_terrastodon_core_user_input::prelude::Choice;
use cloud_terrastodon_core_user_input::prelude::FzfArgs;
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
    let chosen_storage_account = pick(FzfArgs {
        choices: storage_accounts
            .into_iter()
            .map(|sa| {
                let sub_name = subscriptions
                    .get(&sa.subscription_id)
                    .map(|sub| sub.name.to_owned())
                    .unwrap_or_else(|| "Unknown Subscription".to_string());
                let key = format!("{:32}\t{:64}\t{}", sub_name, sa.resource_group, sa.name);
                let key_short = format!("{} {} {}", sub_name, sa.resource_group, sa.name);
                Choice {
                    key,
                    value: (sa, key_short, sub_name),
                }
            })
            .collect(),
        prompt: Some("Storage Account: ".to_string()),
        header: Some("Picking the storage account for the state file".to_string()),
    })?;

    info!("Fetching blob containers for {}", chosen_storage_account.1);
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["storage", "container", "list", "--account-name"]);
    cmd.arg(&chosen_storage_account.0.name);
    cmd.arg("--subscription");
    cmd.arg(chosen_storage_account.0.subscription_id.short_form());
    cmd.args(["--query", "[].name", "--output", "json"]);
    let blob_container_names = cmd.run::<Vec<String>>().await?;

    let chosen_blob_container = match blob_container_names.len() {
        0 => {
            bail!("No blob containers found in {}", chosen_storage_account.1);
        }
        1 => blob_container_names.first().unwrap(),
        _ => {
            info!("Picking blob container");
            &pick(FzfArgs {
                choices: blob_container_names
                    .into_iter()
                    .map(|name| Choice {
                        key: name.to_owned(),
                        value: name,
                    })
                    .collect(),
                prompt: None,
                header: Some("Blob Container Name: ".to_string()),
            })?
            .value
        }
    };

    let output = format!(
        r#"
    resource_group_name  = "{}"
    storage_account_name = "{}"
    container_name       = "{}"
    subscription_id      = "{}" # {}
    "#,
        chosen_storage_account.0.resource_group,
        chosen_storage_account.0.name,
        chosen_blob_container,
        chosen_storage_account.0.subscription_id.short_form(),
        chosen_storage_account.2
    );

    info!("You chose:\n{output}");

    Ok(())
}

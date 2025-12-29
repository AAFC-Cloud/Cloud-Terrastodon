use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::StorageAccount;
use cloud_terrastodon_azure::prelude::fetch_all_storage_accounts;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;

pub async fn browse_storage_accounts() -> Result<()> {
    let storage_accounts = fetch_all_storage_accounts().await?;
    let chosen: Vec<StorageAccount> = PickerTui::new().pick_many(
        storage_accounts.into_iter().map(|storage_account| Choice {
            key: storage_account.id.expanded_form(),
            value: storage_account,
        }),
    )?;
    println!("You chose: {}", serde_json::to_string_pretty(&chosen)?);
    Ok(())
}

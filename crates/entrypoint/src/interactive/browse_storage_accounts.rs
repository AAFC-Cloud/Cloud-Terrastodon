use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::StorageAccount;
use cloud_terrastodon_azure::fetch_all_storage_accounts;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;

pub async fn browse_storage_accounts(tenant_id: AzureTenantId) -> Result<()> {
    let storage_accounts = fetch_all_storage_accounts(tenant_id).await?;
    let chosen: Vec<StorageAccount> =
        PickerTui::new().pick_many(storage_accounts.into_iter().map(|storage_account| Choice {
            key: storage_account.id.expanded_form(),
            value: storage_account,
        }))?;
    println!("You chose: {}", serde_json::to_string_pretty(&chosen)?);
    Ok(())
}

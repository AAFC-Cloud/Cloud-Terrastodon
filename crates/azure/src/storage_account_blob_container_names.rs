use cloud_terrastodon_azure_types::prelude::StorageAccountBlobContainerName;
use cloud_terrastodon_azure_types::prelude::StorageAccountId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Duration;

/// This can fail due to network rules on the storage account
pub async fn fetch_storage_account_blob_container_names(
    storage_account_id: &StorageAccountId,
) -> Result<HashSet<StorageAccountBlobContainerName>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args([
        "storage",
        "container",
        "list",
        "--account-name",
        &storage_account_id.storage_account_name,
        "--subscription",
        storage_account_id
            .resource_group_id
            .subscription_id
            .as_hyphenated()
            .to_string()
            .as_ref(),
        "--query",
        "[].name",
        "--output",
        "json",
        "--auth-mode",
        "login",
    ]);
    cmd.use_cache_behaviour(Some(CacheKey {
        path: PathBuf::from("storage_accounts"),
        valid_for: Duration::MAX,
    }));
    let rtn = cmd.run().await?;
    Ok(rtn)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_storage_accounts;
    use crate::prelude::fetch_storage_account_blob_container_names;
    use eyre::bail;

    #[tokio::test]
    pub async fn blob_works() -> eyre::Result<()> {
        let storage_accounts = fetch_all_storage_accounts().await?;
        for sa in storage_accounts.into_iter() {
            if let Ok(blob_containers) = fetch_storage_account_blob_container_names(&sa.id).await {
                println!("Storage account: {sa:#?}");
                println!("Blob containers: {blob_containers:#?}");
                return Ok(());
            }
        }
        bail!("Failed to get any blob containers D:")
    }
}

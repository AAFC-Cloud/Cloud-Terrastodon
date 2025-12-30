use cloud_terrastodon_azure_types::prelude::StorageAccountBlobContainerName;
use cloud_terrastodon_azure_types::prelude::StorageAccountId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::impl_cacheable_into_future;
use eyre::Result;
use std::collections::HashSet;
use std::path::PathBuf;

/// This can fail due to network rules on the storage account
pub struct StorageAccountBlobContainerNamesListRequest {
    storage_account_id: StorageAccountId,
}

pub fn fetch_storage_account_blob_container_names(
    storage_account_id: StorageAccountId,
) -> StorageAccountBlobContainerNamesListRequest {
    StorageAccountBlobContainerNamesListRequest { storage_account_id }
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for StorageAccountBlobContainerNamesListRequest {
    type Output = HashSet<StorageAccountBlobContainerName>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from("storage_accounts"))
    }

    async fn run(self) -> Result<Self::Output> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        let subscription = self
            .storage_account_id
            .resource_group_id
            .subscription_id
            .as_hyphenated()
            .to_string();
        cmd.args([
            "storage",
            "container",
            "list",
            "--account-name",
            &self.storage_account_id.storage_account_name,
            "--subscription",
            subscription.as_ref(),
            "--query",
            "[].name",
            "--output",
            "json",
            "--auth-mode",
            "login",
        ]);
        cmd.cache(CacheKey::new(PathBuf::from("storage_accounts")));
        let rtn = cmd.run().await?;
        Ok(rtn)
    }
}

impl_cacheable_into_future!(StorageAccountBlobContainerNamesListRequest);

#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_storage_accounts;
    use crate::prelude::fetch_storage_account_blob_container_names;
    use eyre::bail;

    #[tokio::test]
    pub async fn blob_works() -> eyre::Result<()> {
        let storage_accounts = fetch_all_storage_accounts().await?;
        for sa in storage_accounts.into_iter() {
            if let Ok(blob_containers) = fetch_storage_account_blob_container_names(sa.id.clone()).await {
                println!("Storage account: {sa:#?}");
                println!("Blob containers: {blob_containers:#?}");
                return Ok(());
            }
        }
        bail!("Failed to get any blob containers D:")
    }
}

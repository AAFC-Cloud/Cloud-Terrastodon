use cloud_terrastodon_azure_types::prelude::Scope;
use cloud_terrastodon_azure_types::prelude::StorageAccountBlobContainerName;
use cloud_terrastodon_azure_types::prelude::StorageAccountId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::collections::HashSet;
use std::path::PathBuf;

/// This can fail due to network rules on the storage account
pub struct StorageAccountBlobContainerNamesListRequest<'a> {
    storage_account_id: &'a StorageAccountId,
}

pub fn fetch_storage_account_blob_container_names<'a>(
    storage_account_id: &'a StorageAccountId,
) -> StorageAccountBlobContainerNamesListRequest<'a> {
    StorageAccountBlobContainerNamesListRequest { storage_account_id }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand
    for StorageAccountBlobContainerNamesListRequest<'a>
{
    type Output = HashSet<StorageAccountBlobContainerName>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "storage_accounts",
            self.storage_account_id.expanded_form().as_ref(),
        ]))
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
        cmd.cache(self.cache_key());

        let rtn = cmd.run().await?;
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(StorageAccountBlobContainerNamesListRequest<'a>, 'a);

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

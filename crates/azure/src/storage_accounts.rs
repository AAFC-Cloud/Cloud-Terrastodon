use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::StorageAccount;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use indoc::indoc;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct StorageAccountListRequest;

pub fn fetch_all_storage_accounts() -> StorageAccountListRequest {
    StorageAccountListRequest
}

#[async_trait]
impl CacheableCommand for StorageAccountListRequest {
    type Output = Vec<StorageAccount>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "storage_accounts",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        ResourceGraphHelper::new(
            indoc! {r#"
                Resources
                | where type == "microsoft.storage/storageaccounts"
                | project id,name,kind,location,sku,properties,tags
            "#},
            Some(self.cache_key()),
        )
        .collect_all::<StorageAccount>()
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(StorageAccountListRequest);

#[cfg(test)]
mod test {
    use super::fetch_all_storage_accounts;
    use crate::prelude::is_storage_account_name_available;
    use cloud_terrastodon_azure_types::prelude::Slug;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let storage_accounts = fetch_all_storage_accounts().await?;
        let check_name = rand::random::<u64>() % storage_accounts.len() as u64;
        for (i, sa) in storage_accounts.into_iter().enumerate() {
            sa.name.validate_slug()?;
            if i == check_name as usize {
                println!("Storage account: {sa:?}");
                assert!(!is_storage_account_name_available(&sa.name).await?);
            }
        }
        Ok(())
    }
}

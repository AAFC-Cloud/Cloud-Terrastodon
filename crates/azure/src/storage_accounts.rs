use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::StorageAccount;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_storage_accounts() -> Result<Vec<StorageAccount>> {
    let mut query = ResourceGraphHelper::new(
        r#"
Resources
| where type == "microsoft.storage/storageaccounts"
| project id,name,kind,location,sku,properties,tags
        "#,
        CacheBehaviour::Some {
            path: PathBuf::from_iter(["az", "resource_graph", "storage_accounts"]),
            valid_for: Duration::MAX,
        },
    );
    query.collect_all().await
}

#[cfg(test)]
mod test {
    use super::fetch_all_storage_accounts;
    use crate::prelude::fetch_storage_account_blob_container_names;
    use crate::prelude::is_storage_account_name_available;
    use cloud_terrastodon_azure_types::prelude::Slug;
    use eyre::bail;

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

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::StorageAccount;
use cloud_terrastodon_core_command::prelude::CacheBehaviour;

use crate::prelude::ResourceGraphHelper;

pub async fn fetch_all_storage_accounts() -> Result<Vec<StorageAccount>> {
    let mut query = ResourceGraphHelper::new(
        r#"
Resources
| where type == "microsoft.storage/storageaccounts"
        "#
        .to_string(),
        CacheBehaviour::Some {
            path: PathBuf::from("storage_accounts"),
            valid_for: Duration::from_hours(8),
        },
    );
    query.collect_all().await
}

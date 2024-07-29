use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use azure_types::prelude::StorageAccount;
use command::prelude::CacheBehaviour;

use crate::prelude::QueryBuilder;

pub async fn fetch_all_storage_accounts() -> Result<Vec<StorageAccount>> {
    let mut query = QueryBuilder::new(
        r#"
Resources
| where type == "microsoft.storage/storageaccounts"
        "#.to_string(),
        CacheBehaviour::Some {
            path: PathBuf::from("storage_accounts"),
            valid_for: Duration::from_hours(8),
        },
    );
    query.collect_all().await
}

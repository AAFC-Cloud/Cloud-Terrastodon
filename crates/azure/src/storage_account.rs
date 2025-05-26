use std::path::PathBuf;
use std::time::Duration;

use cloud_terrastodon_azure_types::prelude::StorageAccount;
use cloud_terrastodon_azure_types::prelude::StorageAccountName;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use serde::Deserialize;
use serde_json::Value;

use crate::prelude::ResourceGraphHelper;

pub async fn fetch_all_storage_accounts() -> Result<Vec<StorageAccount>> {
    let mut query = ResourceGraphHelper::new(
        r#"
Resources
| where type == "microsoft.storage/storageaccounts"
        "#,
        CacheBehaviour::Some {
            path: PathBuf::from("storage_accounts"),
            valid_for: Duration::from_hours(8),
        },
    );
    query.collect_all().await
}

pub async fn is_storage_account_name_available(name: &StorageAccountName) -> eyre::Result<bool> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["storage", "account", "check-name", "--name", name]);
    #[derive(Deserialize)]
    #[allow(unused)]
    struct Response {
        message: String,
        #[serde(rename = "nameAvailable")]
        name_available: bool,
        reason: Value,
    }
    let response = cmd.run::<Response>().await?;
    Ok(response.name_available)
}

#[cfg(test)]
mod test {
    use crate::prelude::{fetch_all_storage_accounts, is_storage_account_name_available};
    use validator::Validate;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let storage_accounts = fetch_all_storage_accounts().await?;
        let check_name = rand::random::<usize>() % storage_accounts.len();
        for (i, sa) in storage_accounts.into_iter().enumerate() {
            sa.name.validate()?;
            if i == check_name {
                assert!(!is_storage_account_name_available(&sa.name).await?);
            }
        }
        Ok(())
    }
}

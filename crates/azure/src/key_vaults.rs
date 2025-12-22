use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::KeyVault;
use cloud_terrastodon_azure_types::prelude::KeyVaultName;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_all_key_vaults() -> Result<Vec<KeyVault>> {
    let mut query = ResourceGraphHelper::new(
        r#"
resources
| where type =~ "microsoft.keyvault/vaults"
| project id,name,location,properties,tags
        "#,
        CacheBehaviour::Some {
            path: PathBuf::from_iter(["az", "resource_graph", "key_vaults"]),
            valid_for: Duration::from_hours(8),
        },
    );
    query.collect_all().await
}

#[deprecated(note = "https://github.com/Azure/azure-cli/issues/31178")]
pub async fn is_key_vault_name_available(name: &KeyVaultName) -> eyre::Result<bool> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["keyvault", "check-name", "--name", name]);
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
    use crate::prelude::fetch_all_key_vaults;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let key_vaults = fetch_all_key_vaults().await?;
        assert!(!key_vaults.is_empty());
        Ok(())
    }
}

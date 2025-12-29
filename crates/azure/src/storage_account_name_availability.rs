use cloud_terrastodon_azure_types::prelude::StorageAccountName;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use serde::Deserialize;
use serde_json::Value;

pub async fn is_storage_account_name_available(name: &StorageAccountName) -> Result<bool> {
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

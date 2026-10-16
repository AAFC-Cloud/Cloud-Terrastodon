use cloud_terrastodon_azure_types::StorageAccountName;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;

pub async fn is_storage_account_name_available(
    name: &StorageAccountName,
    cancellation: &CancellationScope,
) -> Result<bool> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.cancellation_safe();
    cmd.args(["storage", "account", "check-name", "--name", name]);
    #[derive(facet::Facet)]
    #[allow(unused)]
    struct Response {
        message: String,
        #[facet(rename = "nameAvailable")]
        name_available: bool,
        reason: facet_json::RawJson<'static>,
    }
    let response = cmd.run::<Response>(cancellation).await?;
    Ok(response.name_available)
}

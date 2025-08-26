use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use serde_json::Value;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let resource_groups = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["group", "list"])
        .run::<Vec<Value>>()
        .await?;
    println!("Found {} resource groups", resource_groups.len());
    Ok(())
}

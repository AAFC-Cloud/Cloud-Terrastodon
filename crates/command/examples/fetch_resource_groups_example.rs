use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use facet_json::RawJson;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let resource_groups = CommandBuilder::new(CommandKind::AzureCLI)
        .args(["group", "list"])
        .run::<Vec<RawJson<'static>>>()
        .await?;
    println!("Found {} resource groups", resource_groups.len());
    Ok(())
}

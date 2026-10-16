use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use facet_json::RawJson;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            let resource_groups = CommandBuilder::new(CommandKind::AzureCLI)
                .cancellation_safe()
                .args(["group", "list"])
                .run::<Vec<RawJson<'static>>>(&cancellation)
                .await?;
            println!("Found {} resource groups", resource_groups.len());
            Ok(())
        })
        .await
}

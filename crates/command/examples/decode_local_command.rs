use cloud_terrastodon_command::CancellationToken;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let numbers = CommandBuilder::new(CommandKind::Echo)
                .cancellation_safe()
                .args(["[1,2,3]"])
                .run::<Vec<u64>>(&scope)
                .await?;
            println!("Decoded {} numbers from a local command", numbers.len());
            Ok(())
        })
        .await
}

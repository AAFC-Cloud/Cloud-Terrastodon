use cloud_terrastodon_user_input::are_you_sure;
use teamy_cancellation::CancellationToken;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            if are_you_sure(
                "This will delete the universe. Are you sure you want to proceed?",
                &scope,
            )
            .await?
            {
                println!("Proceeding!");
            } else {
                println!("Action cancelled.");
            }

            Ok(())
        })
        .await
}

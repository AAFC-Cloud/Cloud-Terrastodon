use cloud_terrastodon_user_input::PickerTui;
use teamy_cancellation::CancellationToken;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let nouns = vec!["dog", "cat", "house", "pickle", "mouse"];
            let chosen = PickerTui::<&str>::new()
                .set_header("Pick a noun")
                .set_query("ouse")
                .pick_one(nouns, &scope)
                .await?;
            println!("You chose {}", chosen);

            Ok(())
        })
        .await
}

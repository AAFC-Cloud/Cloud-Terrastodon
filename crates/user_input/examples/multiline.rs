use cloud_terrastodon_user_input::PickerTui;
use teamy_cancellation::CancellationToken;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(async |scope| {
            let choices = vec![
                "First\nSecond\nThird",
                "A\nB\nC",
                "IMPORT BRUH\nDO THING\nWOOHOO!",
                "single item",
                "another single item",
            ];
            let chosen = PickerTui::<&str>::new().pick_many(choices, &scope).await?;
            println!("You chose: {chosen:#?}");
            Ok(())
        })
        .await
}

use cloud_terrastodon_user_input::PickerTui;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let choices = vec![
        "First\nSecond\nThird",
        "A\nB\nC",
        "IMPORT BRUH\nDO THING\nWOOHOO!",
        "single item",
        "another single item",
    ];
    let chosen = PickerTui::<&str>::new().pick_many(choices).await?;
    println!("You chose: {chosen:#?}");
    Ok(())
}

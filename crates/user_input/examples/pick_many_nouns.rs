use cloud_terrastodon_user_input::PickerTui;

#[tokio::main]
pub async fn main() -> eyre::Result<()> {
    let nouns = vec!["dog", "cat", "house", "pickle", "mouse"];
    let chosen = PickerTui::<&str>::new()
        .set_header("Press tab to select entries")
        .pick_many(nouns)
        .await?;

    println!("You chose {:?}", chosen);

    Ok(())
}

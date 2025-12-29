use cloud_terrastodon_user_input::PickerTui;

pub fn main() -> eyre::Result<()> {
    let nouns = vec!["dog", "cat", "house", "pickle", "mouse"];
    let chosen = PickerTui::new()
        .set_header("Press tab to select entries")
        .pick_many(nouns)?;

    println!("You chose {:?}", chosen);

    Ok(())
}

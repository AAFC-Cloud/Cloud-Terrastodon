use cloud_terrastodon_user_input::PickerTui;

pub fn main() -> eyre::Result<()> {
    let nouns = vec!["dog", "cat", "house", "pickle", "mouse"];
    let chosen = PickerTui::new(nouns)
        .set_header("Pick a noun")
        .pick_one()?;
    println!("You chose {}", chosen);

    Ok(())
}

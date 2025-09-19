use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;

pub fn main() -> eyre::Result<()> {
    let mut choices = Vec::new();
    let mut dir = std::fs::read_dir(".")?;
    while let Some(entry) = dir.next() {
        let entry = entry?;
        choices.push(entry);
    }

    let chosen = PickerTui::from(choices.into_iter().map(|entry| Choice {
        key: entry.path().display().to_string(), // the value shown to the user
        value: entry, // the inner value we want to have after the user picks
    }))
    .set_header("Pick a path")
    .pick_one()?;

    println!("You chose {}", chosen.file_name().to_string_lossy());

    Ok(())
}

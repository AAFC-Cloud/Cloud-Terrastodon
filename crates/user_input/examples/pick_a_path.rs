use cloud_terrastodon_user_input::prelude::Choice;
use cloud_terrastodon_user_input::prelude::FzfArgs;
use cloud_terrastodon_user_input::prelude::pick;

pub fn main() -> eyre::Result<()> {
    let mut choices = Vec::new();
    let mut dir = std::fs::read_dir(".")?;
    while let Some(entry) = dir.next() {
        let entry = entry?;
        choices.push(entry);
    }

    let chosen = pick(FzfArgs {
        choices: choices
            .into_iter()
            .map(|entry| Choice {
                key: entry.path().display().to_string(), // the value shown to the user
                value: entry, // the inner value we want to have after the user picks
            })
            .collect(),
        header: Some("Pick a path".to_string()),
        ..Default::default()
    })?;

    println!("You chose {}", chosen.file_name().to_string_lossy());

    Ok(())
}

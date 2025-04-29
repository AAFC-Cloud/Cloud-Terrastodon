use cloud_terrastodon_user_input::prelude::FzfArgs;
use cloud_terrastodon_user_input::prelude::pick_many;

pub fn main() -> eyre::Result<()> {
    let nouns = vec!["dog", "cat", "house", "pickle", "mouse"];
    let chosen = pick_many(FzfArgs {
        choices: nouns,
        header: Some("Press tab to select entries".to_string()),
        prompt: Some("Pick some nouns >".to_string()),
        ..Default::default()
    })?;

    println!("You chose {:?}", chosen);

    Ok(())
}

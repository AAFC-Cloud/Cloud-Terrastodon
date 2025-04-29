use cloud_terrastodon_user_input::FzfArgs;
use cloud_terrastodon_user_input::pick;

pub fn main() -> eyre::Result<()> {
    let nouns = vec!["dog", "cat", "house", "pickle", "mouse"];
    let chosen = pick(FzfArgs {
        choices: nouns,
        header: Some("Pick a noun".to_string()),
        query: Some("ouse".to_string()),
        ..Default::default()
    })?;
    println!("You chose {}", chosen);

    Ok(())
}

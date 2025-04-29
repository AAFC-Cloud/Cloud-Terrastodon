use cloud_terrastodon_user_input::are_you_sure;

fn main() -> eyre::Result<()> {
    if are_you_sure("This will delete the universe. Are you sure you want to proceed?".to_string())? {
        println!("Proceeding!");
    } else {
        println!("Action cancelled.");
    }

    Ok(())
}
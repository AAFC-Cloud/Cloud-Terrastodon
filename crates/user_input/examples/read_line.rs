use cloud_terrastodon_user_input::read_line;
use std::io::Write;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    print!("Enter your name: ");
    std::io::stdout().flush()?;
    let user_input = read_line().await?;
    println!("Hello, {}!", user_input);

    Ok(())
}

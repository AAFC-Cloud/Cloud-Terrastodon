use cloud_terrastodon_user_input::prompt_line;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let user_input = prompt_line("Enter your name: ").await?;
    println!("Hello, {}!", user_input);

    Ok(())
}

use anyhow::Context;
use anyhow::Result;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
pub async fn read_line() -> Result<String> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut input = String::new();

    // Reading a line asynchronously
    reader
        .read_line(&mut input)
        .await
        .context("Failed to read line")?;

    // Remove the newline character from the end of the input
    let input = input.trim();
    Ok(input.to_string())
}
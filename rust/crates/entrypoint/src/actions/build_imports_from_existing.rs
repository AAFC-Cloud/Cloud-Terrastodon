use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use pathing_types::IgnoreDir;
use tofu::prelude::get_imports_from_existing;
use tofu::prelude::TofuWriter;
use tokio::io;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;

async fn read_line() -> Result<String> {
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

pub async fn build_imports_from_existing() -> Result<()> {
    println!("Enter the path to the existing workspace:");
    let tf_dir = read_line().await?;

    // println!("Enter the name for the output file [existing.tf]:");
    // let mut name = read_line().await.unwrap_or_default();
    // if name.is_empty() {
    //     name = "existing.tf".to_string();
    // }
    let name = "existing.tf";

    let imports = get_imports_from_existing(tf_dir).await?;
    if imports.is_empty() {
        return Err(anyhow!("Imports should not be empty"));
    }

    TofuWriter::new(IgnoreDir::Imports.join(name))
        .overwrite(imports)
        .await?;

    Ok(())
}

#![feature(let_chains)]
use anyhow::Result;
use entrypoint::prelude::menu_loop;

#[tokio::main]
async fn main() -> Result<()> {
    menu_loop().await?;

    Ok(())
}

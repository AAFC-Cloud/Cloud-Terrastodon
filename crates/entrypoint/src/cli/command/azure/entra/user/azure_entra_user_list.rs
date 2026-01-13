use clap::Args;
use cloud_terrastodon_azure::prelude::fetch_all_users;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) users.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraUserListArgs {}

impl AzureEntraUserListArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching users");
        let users = fetch_all_users().await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &users)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

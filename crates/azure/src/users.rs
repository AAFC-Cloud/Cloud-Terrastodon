use std::path::PathBuf;

use cloud_terrastodon_azure_types::prelude::User;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use tracing::debug;

pub async fn fetch_all_users() -> Result<Vec<User>> {
    debug!("Fetching users");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["ad", "user", "list", "--output", "json"]);
    cmd.use_cache_dir(PathBuf::from_iter(["az", "ad", "user", "list"]));
    let users: Vec<User> = cmd.run().await?;
    debug!("Found {} users", users.len());
    Ok(users)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_users().await?;
        println!("Found {} users:", result.len());
        for group in result {
            println!("- {} ({})", group.display_name, group.id);
        }
        Ok(())
    }
}

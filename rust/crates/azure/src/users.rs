use cloud_terrastodon_core_azure_types::prelude::User;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Result;
use tracing::info;

pub async fn fetch_all_users() -> Result<Vec<User>> {
    info!("Fetching users");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["ad", "user", "list", "--output", "json"]);
    cmd.use_cache_dir("az ad user list");
    let users: Vec<User> = cmd.run().await?;
    info!("Found {} users", users.len());
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

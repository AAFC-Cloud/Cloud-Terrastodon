use anyhow::Result;
use azure_types::prelude::User;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_users() -> Result<Vec<User>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["ad", "user", "list", "--output", "json"]);
    let mut cache = PathBuf::new();
    cache.push("ignore");
    cache.push("az ad user list");
    cmd.use_cache_dir(Some(cache));
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_users().await?;
        println!("Found {} users:", result.len());
        for group in result {
            println!("- {} ({})", group.display_name, group.id);
        }
        Ok(())
    }
}

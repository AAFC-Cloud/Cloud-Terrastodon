use anyhow::Result;
use azure_types::prelude::Subscription;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_subscriptions() -> Result<Vec<Subscription>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["account", "list", "--output", "json"]);
    let mut cache = PathBuf::new();
    cache.push("ignore");
    cache.push("az account list");
    cmd.use_cache_dir(Some(cache));
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_subscriptions().await?;
        println!("Found {} subscriptions:", result.len());
        for sub in result {
            println!("- {} ({})", sub.name, sub.id);
        }
        Ok(())
    }
}

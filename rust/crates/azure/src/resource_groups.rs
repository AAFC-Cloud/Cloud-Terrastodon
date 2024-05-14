use anyhow::Result;
use azure_types::resource_groups::ResourceGroup;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_resource_groups() -> Result<Vec<ResourceGroup>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["group", "list", "--output", "json"]);
    let mut cache = PathBuf::new();
    cache.push("ignore");
    cache.push("az group list");
    cmd.use_cache_dir(Some(cache));
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_resource_groups().await?;
        println!("Found {} resource groups:", result.len());
        for group in result {
            println!("- {} ({})", group.name, group.id);
        }
        Ok(())
    }
}

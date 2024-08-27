use anyhow::Result;
use cloud_terrastodon_core_azure_types::prelude::Group;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;

pub async fn fetch_groups() -> Result<Vec<Group>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["ad", "group", "list", "--output", "json"]);
    cmd.use_cache_dir("az ad group list");
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_groups().await?;
        println!("Found {} groups:", result.len());
        for group in result {
            println!("- {} ({})", group.display_name, group.id);
        }
        Ok(())
    }
}

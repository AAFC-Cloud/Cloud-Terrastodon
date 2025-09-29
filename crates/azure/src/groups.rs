use cloud_terrastodon_azure_types::prelude::Group;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

pub async fn fetch_all_groups() -> Result<Vec<Group>> {
    debug!("Fetching Azure AD groups");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["ad", "group", "list", "--output", "json"]);
    cmd.use_cache_dir(PathBuf::from_iter(["az", "ad", "group", "list"]));
    let rtn: Vec<Group> = cmd.run().await?;
    debug!("Found {} groups", rtn.len());
    Ok(rtn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_groups() -> Result<()> {
        let result = fetch_all_groups().await?;
        println!("Found {} groups:", result.len());
        for group in result {
            println!("- {} ({})", group.display_name, group.id);
        }
        Ok(())
    }
}

use anyhow::Result;
use azure_types::management_groups::ManagementGroup;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use std::path::PathBuf;

pub async fn fetch_management_groups() -> Result<Vec<ManagementGroup>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.use_cache_dir(Some(PathBuf::from_iter([
        "ignore",
        "az account management-group list",
    ])));
    cmd.args([
        "account",
        "management-group",
        "list",
        "--no-register",
        "--output",
        "json",
    ]);
    cmd.run().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_management_groups().await?;
        println!("Found {} management groups:", result.len());
        for mg in result {
            println!("- {} ({})", mg.display_name, mg.name);
        }
        Ok(())
    }
}

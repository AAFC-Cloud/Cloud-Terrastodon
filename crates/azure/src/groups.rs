use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::Group;
use cloud_terrastodon_azure_types::prelude::GroupId;
use cloud_terrastodon_azure_types::prelude::Principal;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::info;

pub async fn fetch_groups() -> Result<Vec<Group>> {
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["ad", "group", "list", "--output", "json"]);
    cmd.use_cache_dir(PathBuf::from_iter(["az", "ad", "group", "list"]));
    cmd.run().await
}

pub async fn fetch_group_members(group_id: GroupId) -> Result<Vec<Principal>> {
    let members = MicrosoftGraphHelper::new(
        format!("https://graph.microsoft.com/v1.0/groups/{group_id}/members"),
        CacheBehaviour::Some {
            path: PathBuf::from_iter([
                "group_members",
                group_id.as_hyphenated().to_string().as_ref(),
            ]),
            valid_for: Duration::from_hours(8),
        },
    )
    .fetch_all::<Principal>()
    .await?;
    info!("Found {} members for group {}", members.len(), group_id);
    Ok(members)
}
pub async fn fetch_group_owners(group_id: GroupId) -> Result<Vec<Principal>> {
    let owners = MicrosoftGraphHelper::new(
        format!("https://graph.microsoft.com/v1.0/groups/{group_id}/owners"),
        CacheBehaviour::Some {
            path: PathBuf::from_iter([
                "group_owners",
                group_id.as_hyphenated().to_string().as_ref(),
            ]),
            valid_for: Duration::from_hours(8),
        },
    )
    .fetch_all::<Principal>()
    .await?;
    info!("Found {} owners for group {}", owners.len(), group_id);
    Ok(owners)
}

#[cfg(test)]
mod tests {
    use super::*;
    use eyre::bail;

    #[tokio::test]
    async fn list_groups() -> Result<()> {
        let result = fetch_groups().await?;
        println!("Found {} groups:", result.len());
        for group in result {
            println!("- {} ({})", group.display_name, group.id);
        }
        Ok(())
    }
    #[tokio::test]
    async fn list_group_members() -> Result<()> {
        let groups = fetch_groups().await?;
        // there's a chance that some groups just don't have members lol
        // lets hope that we aren't unlucky many times in a row
        let tries = 10.min(groups.len());
        for group in groups.iter().take(tries) {
            println!("Checking group {} for members", group.id);
            let members = fetch_group_members(group.id).await?;
            if !members.is_empty() {
                println!("Found {} members for group {}", members.len(), group.id);
                return Ok(());
            }
        }
        bail!("Failed to ensure group member fetching worked after {tries} tries")
    }
    #[tokio::test]
    async fn list_group_owners() -> Result<()> {
        let groups = fetch_groups().await?;
        // there's a chance that some groups just don't have members lol
        // lets hope that we aren't unlucky many times in a row
        let tries = 10.min(groups.len());
        for group in groups.iter().take(tries) {
            println!("Checking group {} for owners", group.id);
            let owners = fetch_group_owners(group.id).await?;
            if !owners.is_empty() {
                println!("Found {} owners for group {}", owners.len(), group.id);
                return Ok(());
            }
        }
        bail!("Failed to ensure group owner fetching worked after {tries} tries")
    }
}

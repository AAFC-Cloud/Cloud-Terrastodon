use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::GroupId;
use cloud_terrastodon_azure_types::prelude::Principal;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub async fn fetch_group_owners(group_id: GroupId) -> Result<Vec<Principal>> {
    debug!("Fetching owners for group {}", group_id);
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
    debug!("Found {} owners for group {}", owners.len(), group_id);
    Ok(owners)
}

#[cfg(test)]
mod tests {
    use crate::groups::fetch_all_groups;

    use super::*;
    use eyre::bail;

    #[tokio::test]
    async fn list_group_owners() -> Result<()> {
        let groups = fetch_all_groups().await?;
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

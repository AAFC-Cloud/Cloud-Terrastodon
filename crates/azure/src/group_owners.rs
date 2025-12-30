use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::GroupId;
use cloud_terrastodon_azure_types::prelude::Principal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use tracing::debug;

pub struct GroupOwnersListRequest {
    group_id: GroupId,
}

pub fn fetch_group_owners(group_id: GroupId) -> GroupOwnersListRequest {
    GroupOwnersListRequest { group_id }
}

#[async_trait]
impl CacheableCommand for GroupOwnersListRequest {
    type Output = Vec<Principal>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "GET".to_string(),
            "group_owners".to_string(),
            self.group_id.as_hyphenated().to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching owners for group {}", self.group_id);
        let owners = MicrosoftGraphHelper::new(
            format!(
                "https://graph.microsoft.com/v1.0/groups/{}/owners",
                self.group_id
            ),
            Some(self.cache_key()),
        )
        .fetch_all::<Principal>()
        .await?;
        debug!("Found {} owners for group {}", owners.len(), self.group_id);
        Ok(owners)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GroupOwnersListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::fetch_all_groups;
    use eyre::bail;

    #[tokio::test]
    async fn list_group_owners() -> eyre::Result<()> {
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

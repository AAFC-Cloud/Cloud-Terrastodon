use crate::prelude::MicrosoftGraphBatchRequestEntry;
use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::GroupId;
use cloud_terrastodon_azure_types::prelude::Principal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use tracing::debug;

pub struct GroupMembersListRequest {
    pub group_id: GroupId,
}
impl GroupMembersListRequest {
    pub fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/groups/{}/members",
            self.group_id
        )
    }
}
impl From<GroupMembersListRequest> for MicrosoftGraphBatchRequestEntry<Vec<Principal>> {
    fn from(request: GroupMembersListRequest) -> Self {
        MicrosoftGraphBatchRequestEntry::new_get(format!("group-members-for-{}", request.group_id), request.url())
    }
}

pub fn fetch_group_members(group_id: GroupId) -> GroupMembersListRequest {
    GroupMembersListRequest { group_id }
}

#[async_trait]
impl CacheableCommand for GroupMembersListRequest {
    type Output = Vec<Principal>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "GET".to_string(),
            "group_members".to_string(),
            self.group_id.as_hyphenated().to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching members for group {}", self.group_id);
        let members = MicrosoftGraphHelper::new(
            format!(
                "https://graph.microsoft.com/v1.0/groups/{}/members",
                self.group_id
            ),
            Some(self.cache_key()),
        )
        .fetch_all::<Principal>()
        .await?;
        debug!(
            "Found {} members for group {}",
            members.len(),
            self.group_id
        );
        Ok(members)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GroupMembersListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::fetch_all_groups;
    use eyre::bail;

    #[tokio::test]
    async fn list_group_members() -> eyre::Result<()> {
        let groups = fetch_all_groups().await?;
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
}

use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::Principal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use tracing::debug;

#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct EntraGroupMembersListRequest {
    pub group_id: EntraGroupId,
    pub tenant_id: AzureTenantId,
}
impl EntraGroupMembersListRequest {
    pub fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/groups/{}/members",
            self.group_id
        )
    }
}
impl From<EntraGroupMembersListRequest> for MicrosoftGraphBatchRequestEntry<Vec<Principal>> {
    fn from(request: EntraGroupMembersListRequest) -> Self {
        MicrosoftGraphBatchRequestEntry::new_get(
            format!("group-members-for-{}", request.group_id),
            request.url(),
        )
    }
}

pub fn fetch_group_members(
    tenant_id: AzureTenantId,
    group_id: EntraGroupId,
) -> EntraGroupMembersListRequest {
    EntraGroupMembersListRequest {
        group_id,
        tenant_id,
    }
}

#[async_trait]
impl CacheableCommand for EntraGroupMembersListRequest {
    type Output = Vec<Principal>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "GET".to_string(),
            "group_members".to_string(),
            self.tenant_id.to_string(),
            self.group_id.as_hyphenated().to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(tenant_id = %self.tenant_id, group_id = %self.group_id, "Fetching group members");
        let query = MicrosoftGraphHelper::new(
            self.tenant_id,
            format!(
                "https://graph.microsoft.com/v1.0/groups/{}/members",
                self.group_id
            ),
            Some(self.cache_key()),
        );
        let members = query.fetch_all::<Principal>().await?;
        debug!(
            "Found {} members for group {}",
            members.len(),
            self.group_id
        );
        Ok(members)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraGroupMembersListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_all_groups;
    use crate::get_test_tenant_id;
    use eyre::bail;

    #[tokio::test]
    async fn list_group_members() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let groups = fetch_all_groups(tenant_id).await?;
        assert!(!groups.is_empty());
        // there's a chance that some groups just don't have members lol
        // lets hope that we aren't unlucky many times in a row
        let tries = 10.min(groups.len());
        for group in groups.iter().take(tries) {
            let members = fetch_group_members(tenant_id, group.id).await?;
            if !members.is_empty() {
                return Ok(());
            }
        }
        bail!("Failed to ensure group member fetching worked after {tries} tries")
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupMembersListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMembersListRequest);
cloud_terrastodon_registry::register_into_future!(EntraGroupMembersListRequest => Vec<Principal>);

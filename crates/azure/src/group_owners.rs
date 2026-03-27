use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::Principal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use tracing::debug;

pub struct GroupOwnersListRequest {
    pub group_id: EntraGroupId,
    pub tenant_id: AzureTenantId,
}

pub fn fetch_group_owners(
    tenant_id: AzureTenantId,
    group_id: EntraGroupId,
) -> GroupOwnersListRequest {
    GroupOwnersListRequest {
        group_id,
        tenant_id,
    }
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
            self.tenant_id.to_string(),
            self.group_id.as_hyphenated().to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(tenant_id = %self.tenant_id, group_id = %self.group_id, "Fetching group owners");
        let query = MicrosoftGraphHelper::new(
            self.tenant_id,
            format!(
                "https://graph.microsoft.com/v1.0/groups/{}/owners",
                self.group_id
            ),
            Some(self.cache_key()),
        );
        let owners = query.fetch_all::<Principal>().await?;
        debug!("Found {} owners for group {}", owners.len(), self.group_id);
        Ok(owners)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GroupOwnersListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use crate::groups::fetch_all_groups;
    use eyre::bail;

    #[tokio::test]
    async fn list_group_owners() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let groups = fetch_all_groups(tenant_id).await?;
        assert!(!groups.is_empty());
        // there's a chance that some groups just don't have members lol
        // lets hope that we aren't unlucky many times in a row
        let tries = 10.min(groups.len());
        for group in groups.iter().take(tries) {
            let owners = fetch_group_owners(tenant_id, group.id).await?;
            if !owners.is_empty() {
                return Ok(());
            }
        }
        bail!("Failed to ensure group owner fetching worked after {tries} tries")
    }
}

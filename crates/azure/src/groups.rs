use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::EntraGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct GroupListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_groups(tenant_id: AzureTenantId) -> GroupListRequest {
    GroupListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for GroupListRequest {
    type Output = Vec<EntraGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(tenant_id = %self.tenant_id, "Fetching Entra groups");
        let rtn: Vec<EntraGroup> = MicrosoftGraphHelper::new(
            "https://graph.microsoft.com/v1.0/groups",
            Some(self.cache_key()),
        )
        .tenant_id(self.tenant_id)
        .fetch_all()
        .await?;
        debug!(tenant_id = %self.tenant_id, count = rtn.len(), "Found Entra groups");
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GroupListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    async fn list_groups() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let result = fetch_all_groups(tenant_id).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}

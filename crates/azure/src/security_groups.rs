use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct SecurityGroupListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_security_groups(tenant_id: AzureTenantId) -> SecurityGroupListRequest {
    SecurityGroupListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for SecurityGroupListRequest {
    type Output = Vec<EntraGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter(["ms", "graph", "GET", "security_groups"]),
            valid_for: Duration::from_secs(2 * 60 * 60),
        }
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching security groups");
        let query = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/v1.0/groups?$filter=securityEnabled eq true",
            Some(self.cache_key()),
        );
        let groups: Vec<EntraGroup> = query.fetch_all().await?;
        debug!("Found {} security groups", groups.len());
        Ok(groups)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(SecurityGroupListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_all_security_groups;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::EntraGroup;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let groups: Vec<EntraGroup> =
            fetch_all_security_groups(get_test_tenant_id().await?).await?;
        assert!(groups.len() > 1);
        Ok(())
    }
}

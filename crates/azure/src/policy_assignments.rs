use crate::prelude::ResourceGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::PolicyAssignment;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
pub struct PolicyAssignmentListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_policy_assignments(tenant_id: AzureTenantId) -> PolicyAssignmentListRequest {
    PolicyAssignmentListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for PolicyAssignmentListRequest {
    type Output = Vec<PolicyAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_assignments",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let mut qb = ResourceGraphHelper::new(
            self.tenant_id,
            r#"
policyresources
| where type =~ "microsoft.authorization/policyassignments"
| project 
 id,
 name,
 location,
 identity,
 properties
    "#,
            Some(self.cache_key()),
        );
        qb.collect_all().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PolicyAssignmentListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_policy_assignments(get_test_tenant_id().await?).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}

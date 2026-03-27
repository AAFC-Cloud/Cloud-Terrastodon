use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::UnifiedRoleAssignment;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use tracing::debug;

/// Fetches Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
pub struct UnifiedRoleAssignmentListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_unified_role_assignments(
    tenant_id: AzureTenantId,
) -> UnifiedRoleAssignmentListRequest {
    UnifiedRoleAssignmentListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for UnifiedRoleAssignmentListRequest {
    type Output = Vec<UnifiedRoleAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_assignments",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching all unified role assignments");
        let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleAssignments";
        let query = MicrosoftGraphHelper::new(self.tenant_id, url, Some(self.cache_key()));
        let rtn = query.fetch_all().await?;
        debug!("Fetched {} unified role assignments", rtn.len());
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(UnifiedRoleAssignmentListRequest);

#[cfg(test)]
mod test {
    use crate::get_test_tenant_id;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let assignments =
            super::fetch_all_unified_role_assignments(get_test_tenant_id().await?).await?;
        assert!(!assignments.is_empty());
        Ok(())
    }
}

use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinitionCollection;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

/// Fetch all Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
pub struct UnifiedRoleDefinitionListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_unified_role_definitions(
    tenant_id: AzureTenantId,
) -> UnifiedRoleDefinitionListRequest {
    UnifiedRoleDefinitionListRequest { tenant_id }
}
pub fn fetch_all_entra_role_definitions(
    tenant_id: AzureTenantId,
) -> UnifiedRoleDefinitionListRequest {
    UnifiedRoleDefinitionListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for UnifiedRoleDefinitionListRequest {
    type Output = UnifiedRoleDefinitionCollection;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_definitions",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions"; // ?$top=500
        let query = MicrosoftGraphHelper::new(self.tenant_id, url, Some(self.cache_key()));
        query
            .fetch_all()
            .await
            .map(UnifiedRoleDefinitionCollection::new)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(UnifiedRoleDefinitionListRequest);
#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_unified_role_definitions;
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let role_definitions =
            fetch_all_unified_role_definitions(get_test_tenant_id().await?).await?;
        assert!(!role_definitions.is_empty());
        assert!(
            role_definitions
                .values()
                .all(|r| r.resource_scopes == vec!["/".to_string()])
        );
        Ok(())
    }
}

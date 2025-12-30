use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinitionCollection;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;

/// Fetch all Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
pub struct UnifiedRoleDefinitionListRequest;

pub fn fetch_all_unified_role_definitions() -> UnifiedRoleDefinitionListRequest {
    UnifiedRoleDefinitionListRequest
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
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions"; // ?$top=500
        let query = MicrosoftGraphHelper::new(url, Some(self.cache_key()));
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

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let role_definitions = fetch_all_unified_role_definitions().await?;
        println!(
            "Sample role definition: {:#?}",
            role_definitions.values().next()
        );
        println!("Found {} role definitions", role_definitions.len());
        assert!(
            role_definitions
                .values()
                .all(|r| r.resource_scopes == vec!["/".to_string()])
        );
        Ok(())
    }
}

use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::UnifiedRoleDefinitionCollection;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use std::borrow::Cow;
use std::path::PathBuf;

/// Fetch all Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
#[derive(facet::Facet)]
pub struct UnifiedRoleDefinitionListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for UnifiedRoleDefinitionListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_unified_role_definitions<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> UnifiedRoleDefinitionListRequest<'a> {
    UnifiedRoleDefinitionListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}
pub fn fetch_all_entra_role_definitions<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> UnifiedRoleDefinitionListRequest<'a> {
    UnifiedRoleDefinitionListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for UnifiedRoleDefinitionListRequest<'_> {
    type Output = UnifiedRoleDefinitionCollection;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_definitions",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions"; // ?$top=500
        let query =
            MicrosoftGraphHelper::new(url, Some(self.cache_key()), self.auth_context.as_ref());
        query
            .fetch_all()
            .await
            .map(UnifiedRoleDefinitionCollection::new)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(UnifiedRoleDefinitionListRequest<'a>, 'a);
#[cfg(test)]
mod test {
    use crate::fetch_all_unified_role_definitions;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let auth_context =
            AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let role_definitions = fetch_all_unified_role_definitions(&auth_context).await?;
        assert!(!role_definitions.is_empty());
        assert!(
            role_definitions
                .values()
                .all(|r| r.resource_scopes == vec!["/".to_string()])
        );
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(UnifiedRoleDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleDefinitionListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleDefinitionCollection);
cloud_terrastodon_registry::register_into_future!(UnifiedRoleDefinitionListRequest<'static> => UnifiedRoleDefinitionCollection);

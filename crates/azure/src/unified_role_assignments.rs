use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::UnifiedRoleAssignment;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

/// Fetches Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
#[derive(facet::Facet)]
pub struct UnifiedRoleAssignmentListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for UnifiedRoleAssignmentListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_unified_role_assignments<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> UnifiedRoleAssignmentListRequest<'a> {
    UnifiedRoleAssignmentListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for UnifiedRoleAssignmentListRequest<'_> {
    type Output = Vec<UnifiedRoleAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_assignments",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching all unified role assignments");
        let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleAssignments";
        let query =
            MicrosoftGraphHelper::new(url, Some(self.cache_key()), self.auth_context.as_ref());
        let rtn = query.fetch_all().await?;
        debug!("Fetched {} unified role assignments", rtn.len());
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(UnifiedRoleAssignmentListRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let auth_context =
            AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let assignments = super::fetch_all_unified_role_assignments(&auth_context).await?;
        assert!(!assignments.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(UnifiedRoleAssignmentListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(UnifiedRoleAssignmentListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(UnifiedRoleAssignmentListRequest<'static> => Vec<UnifiedRoleAssignment>);

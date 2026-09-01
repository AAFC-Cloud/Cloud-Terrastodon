use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::RoleAssignment;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

/// Fetches all AzureRM role assignments.
///
/// Not to be confused with Entra role assignments.
#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct RoleAssignmentListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for RoleAssignmentListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_role_assignments<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> RoleAssignmentListRequest<'a> {
    RoleAssignmentListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheableCommand for RoleAssignmentListRequest<'a> {
    type Output = Vec<RoleAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "role_assignments",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching role assignments");
        let query = ResourceGraphHelper::new(
            r#"
authorizationresources
| where type =~ "microsoft.authorization/roleassignments"
| project
    id,
    scope=properties.scope,
    role_definition_id=properties.roleDefinitionId,
    principal_id=properties.principalId
"#,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );
        let mut query = query;
        let role_assignments: Vec<RoleAssignment> = query.collect_all().await?;
        debug!("Found {} role assignments", role_assignments.len());
        Ok(role_assignments)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(RoleAssignmentListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::RoleAssignmentId;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let auth_context = AuthContext::explicit_azure_cli();
        let auth_context = auth_context.bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let result = fetch_all_role_assignments(&auth_context).await?;
        assert!(result.len() > 2);
        let _interesting_assignments = result
            .into_iter()
            .filter(|role_assignment| {
                matches!(
                    role_assignment.id,
                    RoleAssignmentId::Unscoped(_) | RoleAssignmentId::PortalScoped(_)
                )
            })
            .count();
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(RoleAssignmentListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(RoleAssignmentListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(RoleAssignmentListRequest<'static> => Vec<RoleAssignment>);

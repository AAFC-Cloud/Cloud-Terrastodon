use crate::fetch_all_role_assignments;
use crate::fetch_all_role_definitions;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::RoleDefinitionsAndAssignments;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use std::borrow::Cow;
use std::pin::Pin;
use tokio::try_join;

/// Fetches all AzureRM role assignments and role definitions.
///
/// Not to be confused with Entra role assignments and role definitions.
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct RoleDefinitionsAndAssignmentsListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> Arbitrary<'a> for RoleDefinitionsAndAssignmentsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: AzureTenantId::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_all_role_definitions_and_assignments<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> RoleDefinitionsAndAssignmentsListRequest<'a> {
    RoleDefinitionsAndAssignmentsListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheInvalidatable for RoleDefinitionsAndAssignmentsListRequest<'a> {
    async fn invalidate(&self) -> eyre::Result<()> {
        let definitions =
            fetch_all_role_definitions(self.tenant_id, self.auth_context.as_ref()).cache_key();
        let assignments =
            fetch_all_role_assignments(self.tenant_id, self.auth_context.as_ref()).cache_key();
        try_join!(definitions.invalidate(), assignments.invalidate())?;
        Ok(())
    }
}

impl<'a> IntoFuture for RoleDefinitionsAndAssignmentsListRequest<'a> {
    type Output = eyre::Result<RoleDefinitionsAndAssignments>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.run())
    }
}

impl<'a> RoleDefinitionsAndAssignmentsListRequest<'a> {
    async fn run(self) -> eyre::Result<RoleDefinitionsAndAssignments> {
        let auth_context = self.auth_context;
        let (role_definitions, role_assignments) = try_join!(
            fetch_all_role_definitions(self.tenant_id, auth_context.as_ref()),
            fetch_all_role_assignments(self.tenant_id, auth_context.as_ref())
        )?;

        RoleDefinitionsAndAssignments::try_new(role_definitions, role_assignments)
    }
}
cloud_terrastodon_registry::register_thing!(RoleDefinitionsAndAssignmentsListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(RoleDefinitionsAndAssignmentsListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    RoleDefinitionsAndAssignmentsListRequest<'static> => RoleDefinitionsAndAssignments,
    effects = [Read]
);

#[cfg(test)]
mod test {
    use crate::fetch_all_role_definitions_and_assignments;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let auth_context = AuthContext::default();
        let found =
            fetch_all_role_definitions_and_assignments(get_test_tenant_id().await?, &auth_context)
                .await?;
        assert!(!found.role_assignments.is_empty());
        assert!(!found.role_definitions.is_empty());
        Ok(())
    }
}

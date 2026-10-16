use crate::fetch_all_role_assignments;
use crate::fetch_all_role_definitions;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::RoleDefinitionsAndAssignments;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use std::borrow::Cow;
use std::pin::Pin;
use tokio::try_join;

/// Fetches all AzureRM role assignments and role definitions.
///
/// Not to be confused with Entra role assignments and role definitions.
#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct RoleDefinitionsAndAssignmentsListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> Arbitrary<'a> for RoleDefinitionsAndAssignmentsListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AzureTenantAuthContext::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_role_definitions_and_assignments<'a>(
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> RoleDefinitionsAndAssignmentsListRequest<'a> {
    RoleDefinitionsAndAssignmentsListRequest {
        cancellation: cancellation.clone(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheInvalidatable for RoleDefinitionsAndAssignmentsListRequest<'a> {
    async fn invalidate(&self) -> eyre::Result<()> {
        let definitions =
            fetch_all_role_definitions(self.auth_context.as_ref(), &self.cancellation).cache_key();
        let assignments =
            fetch_all_role_assignments(self.auth_context.as_ref(), &self.cancellation).cache_key();
        try_join!(definitions.invalidate(), assignments.invalidate())?;
        Ok(())
    }
}

impl<'a> IntoFuture for RoleDefinitionsAndAssignmentsListRequest<'a> {
    type Output = eyre::Result<RoleDefinitionsAndAssignments>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send + 'a>>;

    fn into_future(mut self) -> Self::IntoFuture {
        let cancellation = self.cancellation.clone();
        Box::pin(cancellation.cancellable_scope(move |scope| async move {
            self.cancellation = scope;
            self.run().await
        }))
    }
}

impl<'a> RoleDefinitionsAndAssignmentsListRequest<'a> {
    async fn run(self) -> eyre::Result<RoleDefinitionsAndAssignments> {
        let auth_context = self.auth_context;
        let (role_definitions, role_assignments) = try_join!(
            fetch_all_role_definitions(auth_context.as_ref(), &self.cancellation),
            fetch_all_role_assignments(auth_context.as_ref(), &self.cancellation)
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
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let found =
                    fetch_all_role_definitions_and_assignments(&auth_context, &cancellation)
                        .await?;
                assert!(!found.role_assignments.is_empty());
                assert!(!found.role_definitions.is_empty());
                Ok(())
            })
            .await
    }
}

use crate::fetch_all_entra_users;
use crate::fetch_all_security_groups;
use crate::fetch_all_service_principals;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::EntraPrincipal;
use cloud_terrastodon_azure_types::EntraPrincipalCollection;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use itertools::Itertools;
use std::borrow::Cow;
use std::path::PathBuf;
use tokio::try_join;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct PrincipalListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for PrincipalListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_principals<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, PrincipalListRequest<'a>> {
    executor.execute(PrincipalListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> InvalidatesOwnCacheKey for PrincipalListRequest<'a> {}

impl<'a> HasCacheKey for PrincipalListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "principals",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for PrincipalListRequest<'a> {
    type Error = eyre::Report;
    type Output = EntraPrincipalCollection;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching principals (users, security groups, and service principals)");
            let auth_context = self.auth_context;
            let (users, security_groups, service_principals) = try_join!(
                fetch_all_entra_users(auth_context.as_ref(), &executor),
                fetch_all_security_groups(auth_context.as_ref(), &executor),
                fetch_all_service_principals(auth_context.as_ref(), &executor)
            )?;
            let principals: Vec<EntraPrincipal> = users
                .into_iter()
                .map_into()
                .chain(security_groups.into_iter().map_into())
                .chain(service_principals.into_iter().map_into())
                .collect();
            debug!("Found {} principals", principals.len());
            Ok(EntraPrincipalCollection::new(principals))
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::fetch_all_principals;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context =
                    auth_context.bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let found = fetch_all_principals(&auth_context, &executor).await?;
                assert!(found.len() > 10);
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(PrincipalListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PrincipalListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraPrincipalCollection);
cloud_terrastodon_registry_execution::register_executable_request!(PrincipalListRequest<'static> => EntraPrincipalCollection, effects = [Read]);

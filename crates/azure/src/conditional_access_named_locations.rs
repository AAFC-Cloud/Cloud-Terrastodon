use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::ConditionalAccessNamedLocation;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(facet::Facet)]
pub struct ConditionalAccessNamedLocationListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ConditionalAccessNamedLocationListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[track_caller]
pub fn fetch_all_conditional_access_named_locations<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ConditionalAccessNamedLocationListRequest<'a>> {
    executor.execute(ConditionalAccessNamedLocationListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl InvalidatesOwnCacheKey for ConditionalAccessNamedLocationListRequest<'_> {}

impl HasCacheKey for ConditionalAccessNamedLocationListRequest<'_> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "conditional_access_named_locations",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for ConditionalAccessNamedLocationListRequest<'_> {
    type Error = eyre::Report;
    type Output = Vec<Arc<ConditionalAccessNamedLocation>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let query = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/beta/identity/conditionalAccess/namedLocations",
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            );

            let found = query.fetch_all(&executor).await?;
            Ok(found)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::fetch_all_conditional_access_named_locations;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let auth_context = AuthContext::explicit_azure_cli()
                    .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?;
                let found =
                    fetch_all_conditional_access_named_locations(&auth_context, &executor).await?;
                assert!(!found.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ConditionalAccessNamedLocationListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ConditionalAccessNamedLocationListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ConditionalAccessNamedLocationListRequest<'static> => Vec<Arc<ConditionalAccessNamedLocation>>, effects = [Read]);

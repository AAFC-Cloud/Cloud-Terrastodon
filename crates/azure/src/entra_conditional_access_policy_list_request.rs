use crate::MicrosoftGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ConditionalAccessPolicy;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Facet)]
pub struct EntraConditionalAccessPolicyListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraConditionalAccessPolicyListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for EntraConditionalAccessPolicyListRequest {}

impl HasCacheKey for EntraConditionalAccessPolicyListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "conditional_access_policies",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for EntraConditionalAccessPolicyListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<ConditionalAccessPolicy>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let query = MicrosoftGraphHelper::new(
                "https://graph.microsoft.com/beta/identity/conditionalAccess/policies",
                Some(self.cache_key()?),
                self.authentication,
            );

            let policies = query.fetch_all(&executor).await?;
            Ok(policies)
        })
    }
}

cloud_terrastodon_registry::register_thing!(EntraConditionalAccessPolicyListRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraConditionalAccessPolicyListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(EntraConditionalAccessPolicyListRequest => Vec<Arc<ConditionalAccessPolicy>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    EntraConditionalAccessPolicyListRequest,
    name = "conditional-access-policy-list",
    cached
);

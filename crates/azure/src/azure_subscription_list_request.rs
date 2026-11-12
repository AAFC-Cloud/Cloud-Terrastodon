use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::Subscription;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;
use indoc::indoc;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::debug;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureSubscriptionListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureSubscriptionListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureSubscriptionListRequest {}

impl HasCacheKey for AzureSubscriptionListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "subscriptions",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureSubscriptionListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<Subscription>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching subscriptions");
            let query = indoc! {r#"
        resourcecontainers
        | where type =~ "Microsoft.Resources/subscriptions"
        | project 
            name,
            id,
            tenant_id=tenantId,
            management_group_ancestors_chain=properties.managementGroupAncestorsChain,
            tags=tags
    "#};

            let subscriptions =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<Subscription>(&executor)
                    .await?;
            debug!("Found {} subscriptions", subscriptions.len());
            Ok(subscriptions)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureSubscriptionListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureSubscriptionListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureSubscriptionListRequest => Vec<Arc<Subscription>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureSubscriptionListRequest,
    name = "subscription-list",
    cached
);

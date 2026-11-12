use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ServiceGroup;
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
pub struct AzureServiceGroupListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureServiceGroupListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureServiceGroupListRequest {}

impl HasCacheKey for AzureServiceGroupListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "service_groups",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureServiceGroupListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<ServiceGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!("Fetching service groups");
            let query = indoc! {r#"
        resourcecontainers
        | where type =~ "microsoft.management/servicegroups"
        | project
            id,
            name,
            properties
    "#}
            .to_owned();

            let service_groups =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<ServiceGroup>(&executor)
                    .await?;
            debug!("Found {} service groups", service_groups.len());
            Ok(service_groups)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureServiceGroupListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureServiceGroupListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureServiceGroupListRequest => Vec<Arc<ServiceGroup>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureServiceGroupListRequest,
    name = "service-group-list",
    cached
);

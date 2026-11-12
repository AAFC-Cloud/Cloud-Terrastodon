use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzurePublicIpResource;
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
use tracing::info;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzurePublicIpListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzurePublicIpListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzurePublicIpListRequest {}

impl HasCacheKey for AzurePublicIpListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "public_ips",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzurePublicIpListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzurePublicIpResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!(%self.authentication.tenant_id, "Fetching public IP addresses");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/publicipaddresses"
        | project
            id,
            tenantId,
            name,
            sku,
            location,
            tags,
            properties
        "#}
            .to_owned();

            let public_ips =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<AzurePublicIpResource>(&executor)
                    .await?;
            info!(count = public_ips.len(), "Fetched public IP addresses");
            Ok(public_ips)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzurePublicIpListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzurePublicIpListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzurePublicIpListRequest => Vec<Arc<AzurePublicIpResource>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzurePublicIpListRequest,
    name = "public-ip-list",
    cached
);

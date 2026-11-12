use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureCognitiveServicesAccountResource;
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
#[derive(Arbitrary, Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct AzureCognitiveServicesAccountListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureCognitiveServicesAccountListRequest {}

impl HasCacheKey for AzureCognitiveServicesAccountListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "cognitive_services_accounts",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureCognitiveServicesAccountListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureCognitiveServicesAccountResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(%self.authentication.tenant_id, "Fetching Cognitive Services accounts");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.cognitiveservices/accounts"
        | project
            id,
            tenantId,
            name,
            kind,
            location,
            tags,
            sku,
            properties
        "#}
            .to_owned();

            let accounts =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<AzureCognitiveServicesAccountResource>(&executor)
                    .await?;
            debug!(
                count = accounts.len(),
                "Fetched Cognitive Services accounts"
            );
            Ok(accounts)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureCognitiveServicesAccountListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureCognitiveServicesAccountListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureCognitiveServicesAccountListRequest => Vec<Arc<AzureCognitiveServicesAccountResource>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureCognitiveServicesAccountListRequest,
    name = "cognitive-services-account-list",
    cached
);

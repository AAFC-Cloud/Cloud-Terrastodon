use crate::ResourceGraphHelper;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::AzureAppServiceResource;
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
pub struct AzureAppServiceListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureAppServiceListRequest {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            authentication: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

impl InvalidatesOwnCacheKey for AzureAppServiceListRequest {}

impl HasCacheKey for AzureAppServiceListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "app_services",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureAppServiceListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureAppServiceResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!(%self.authentication.tenant_id, "Fetching app services");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.web/sites"
        | project
            id,
            tenantId,
            name,
            kind,
            location,
            tags,
            properties
        "#}
            .to_owned();

            let app_services =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<AzureAppServiceResource>(&executor)
                    .await?;
            info!(count = app_services.len(), "Fetched app services");
            Ok(app_services)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_authentication::test_authentication;
    use cloud_terrastodon_registry::RuntimeValue;
    use eyre::Result;

    #[test]
    fn reflected_app_service_list_clone_retains_its_owned_tenant() -> Result<()> {
        let tenant_id = "11111111-1111-1111-1111-111111111111".parse()?;
        let original = RuntimeValue::from_box(Box::new(AzureAppServiceListRequest {
            authentication: AzureBearerAuthenticationSelection {
                tenant_id,
                ..test_authentication()
            },
        }))?;
        let cloned = original
            .try_clone()?
            .into_box::<AzureAppServiceListRequest>()?
            .downcast::<AzureAppServiceListRequest>()
            .expect("reflected cloning preserves the request type");
        drop(original);
        assert_eq!(cloned.authentication.tenant_id, tenant_id);
        assert_eq!(
            cloned.authentication.account_id,
            test_authentication().account_id
        );
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureAppServiceListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureAppServiceListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureAppServiceListRequest => Vec<Arc<AzureAppServiceResource>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureAppServiceListRequest,
    name = "app-service-list",
    cached
);

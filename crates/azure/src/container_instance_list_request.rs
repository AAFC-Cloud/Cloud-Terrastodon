use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureContainerInstanceResource;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct ContainerInstanceListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_container_instances<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ContainerInstanceListRequest<'a>> {
    executor.execute(ContainerInstanceListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for ContainerInstanceListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for ContainerInstanceListRequest<'a> {}

impl<'a> HasCacheKey for ContainerInstanceListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "container_instances",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for ContainerInstanceListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureContainerInstanceResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!(%self.auth_context.tenant_id, "Fetching Azure container instances");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.containerinstance/containergroups"
        | project
            id,
            tenantId,
            name,
            location,
            tags,
            identity,
            properties
        "#}
            .to_owned();

            let container_instances = ResourceGraphHelper::new(
                query,
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            )
            .collect_all::<AzureContainerInstanceResource>(&executor)
            .await?;
            info!(
                count = container_instances.len(),
                "Fetched Azure container instances"
            );
            Ok(container_instances)
        })
    }
}

cloud_terrastodon_registry::register_thing!(ContainerInstanceListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ContainerInstanceListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(
    ContainerInstanceListRequest<'static> => Vec<Arc<AzureContainerInstanceResource>>,
    effects = [Read]
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;

    #[test_log::test(tokio::test)]
    async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let result = fetch_all_container_instances(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &executor,
                )
                .await?;
                for container_instance in &result {
                    assert!(!container_instance.name.is_empty());
                }
                Ok(())
            })
            .await
    }
}

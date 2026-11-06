use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::ContainerRegistry;
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
#[derive(Debug, Clone, facet::Facet)]
#[facet(traits(Clone))]
pub struct ContainerRegistryListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_container_registries<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ContainerRegistryListRequest<'a>> {
    executor.execute(ContainerRegistryListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for ContainerRegistryListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for ContainerRegistryListRequest<'a> {}

impl<'a> HasCacheKey for ContainerRegistryListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "container_registries",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for ContainerRegistryListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<ContainerRegistry>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            let mut query = ResourceGraphHelper::new(
                r#"
Resources
| where type =~ "Microsoft.ContainerRegistry/registries"
        "#,
                Some(self.cache_key()?),
                self.auth_context.as_ref(),
            );
            query.collect_all(&executor).await
        })
    }
}

#[cfg(test)]
mod test {
    use crate::fetch_all_container_registries;
    use crate::fetch_container_registry_repository_names;
    use crate::fetch_container_registry_repository_tags;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_azure_types::Slug;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let found = fetch_all_container_registries(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &executor,
                )
                .await?;
                assert!(!found.is_empty());
                for registry in found.into_iter() {
                    registry.name.validate_slug()?;
                }
                Ok(())
            })
            .await
    }

    #[tokio::test]
    #[ignore]
    pub async fn it_works2() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let mut pass = false;
                let auth_context =
                    AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?;
                let found = fetch_all_container_registries(&auth_context, &executor).await?;
                let found_count = found.len();
                for (i, container_registry) in found.into_iter().enumerate() {
                    let repository_names = fetch_container_registry_repository_names(
                        &container_registry.id,
                        &auth_context,
                        &executor,
                    )
                    .await?;
                    assert!(i < found_count);

                    let found = repository_names.iter();
                    let found_count = found.len();
                    for (i, repository) in found.enumerate() {
                        assert!(i < found_count);
                        let tags = fetch_container_registry_repository_tags(
                            &container_registry.id,
                            repository,
                            &auth_context,
                            &executor,
                        )
                        .await?;
                        for tag in tags.iter() {
                            assert!(!tag.name.is_empty());
                            pass = true;
                        }
                        // comment this out to display all tags
                        // if !tags.is_empty() {
                        //     return Ok(());
                        // }
                    }
                }
                if !pass {
                    eyre::bail!("No container registries with tags found.");
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ContainerRegistryListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ContainerRegistryListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ContainerRegistryListRequest<'static> => Vec<Arc<ContainerRegistry>>, effects = [Read]);

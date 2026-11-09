use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::ResourceGroup;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_cache::InvalidatesOwnCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use facet::Facet;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::Arc;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct ResourceGroupListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_resource_groups<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, ResourceGroupListRequest<'a>> {
    executor.execute(ResourceGroupListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for ResourceGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for ResourceGroupListRequest<'a> {}

impl<'a> HasCacheKey for ResourceGroupListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "resource_groups",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for ResourceGroupListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<ResourceGroup>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            ResourceGraphHelper::new(
                indoc! {r#"
                resourcecontainers
                | where type =~ "microsoft.resources/subscriptions/resourcegroups"
                | join kind=leftouter (
                    resourcecontainers
                    | where type =~ "Microsoft.Resources/subscriptions"
                    | project subscriptionId,subscription_name=name
                ) on $left.subscriptionId == $right.subscriptionId
                | project
                    id,
                    tenant_id=tenantId,
                    location,
                    managed_by=managedBy,
                    name,
                    properties,
                    tags,
                    subscription_name
            "#},
                Some(self.cache_key()?),
                self.auth_context.tenant_id,
                self.auth_context.auth_source()?,
            )
            .collect_all::<ResourceGroup>(&executor)
            .await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_auth::AuthContext;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_user_input::PickerTui;
    use eyre::Result;

    #[test_log::test(tokio::test)]
    async fn it_works() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let result = fetch_all_resource_groups(
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &executor,
                )
                .await?;
                assert!(!result.is_empty());
                for rg in result {
                    assert!(!rg.name.is_empty());
                }
                Ok(())
            })
            .await
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn invalidation() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                fetch_all_resource_groups(
                    &AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?,
                    &executor,
                )
                .invalidate()
                .await?;
                Ok(())
            })
            .await
    }

    #[test_log::test(tokio::test)]
    #[ignore]
    async fn pick() -> Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let executor = crate::test_executor::test_executor(
                    &cloud_terrastodon_auth::AuthContext::explicit_azure_cli(),
                    &cancellation,
                )?;
                let chosen = PickerTui::<_>::new()
                    .pick_many_reloadable(|invalidate| {
                        let cancellation = cancellation.clone();
                        let executor = executor.clone();
                        async move {
                            let tenant_id = get_test_tenant_id(&cancellation).await?;
                            if invalidate {
                                fetch_all_resource_groups(
                                    &AuthContext::explicit_azure_cli()
                                        .bind_to_azure_tenant(tenant_id)?,
                                    &executor,
                                )
                                .invalidate()
                                .await?;
                            }
                            fetch_all_resource_groups(
                                &AuthContext::explicit_azure_cli()
                                    .bind_to_azure_tenant(tenant_id)?,
                                &executor,
                            )
                            .await
                        }
                    })
                    .await?;
                assert!(!chosen.is_empty());
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(ResourceGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ResourceGroupListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(ResourceGroupListRequest<'static> => Vec<Arc<ResourceGroup>>, effects = [Read]);

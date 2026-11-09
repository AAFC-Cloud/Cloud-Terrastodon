use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::VirtualNetwork;
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
use tracing::info;

#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct VirtualNetworkListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_virtual_networks<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, VirtualNetworkListRequest<'a>> {
    executor.execute(VirtualNetworkListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for VirtualNetworkListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for VirtualNetworkListRequest<'a> {}

impl<'a> HasCacheKey for VirtualNetworkListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "virtual_networks",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for VirtualNetworkListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<VirtualNetwork>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!("Fetching virtual networks");
            let query = indoc! {r#"
            Resources
            | where type == "microsoft.network/virtualnetworks"
            | project
                id,
                name,
                location,
                resource_group_name=resourceGroup,
                subscription_id=subscriptionId,
                tags,
                properties
        "#}
            .to_owned();

            let virtual_networks = ResourceGraphHelper::new(
                query,
                Some(self.cache_key()?),
                self.auth_context.tenant_id,
                self.auth_context.auth_source()?,
            )
            .collect_all::<VirtualNetwork>(&executor)
            .await?;
            info!("Found {} virtual networks", virtual_networks.len());
            Ok(virtual_networks)
        })
    }
}

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
                let result = fetch_all_virtual_networks(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &executor,
                )
                .await?;
                assert!(!result.is_empty());
                for vnet in result {
                    assert!(!vnet.name.is_empty());
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(VirtualNetworkListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(VirtualNetworkListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(VirtualNetworkListRequest<'static> => Vec<Arc<VirtualNetwork>>, effects = [Read]);

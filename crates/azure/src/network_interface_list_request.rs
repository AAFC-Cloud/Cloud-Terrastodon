use crate::ResourceGraphHelper;
use cloud_terrastodon_auth::AzureTenantAuthContext;
use cloud_terrastodon_azure_types::AzureNetworkInterfaceResource;
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

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, Facet)]
#[facet(traits(Clone))]
pub struct NetworkInterfaceListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

#[track_caller]
pub fn fetch_all_network_interfaces<'a>(
    auth_context: &'a AzureTenantAuthContext,
    executor: &'a CloudTerrastodonExecutor,
) -> RequestExecution<'a, NetworkInterfaceListRequest<'a>> {
    executor.execute(NetworkInterfaceListRequest {
        auth_context: Cow::Borrowed(auth_context),
    })
}

impl<'a> arbitrary::Arbitrary<'a> for NetworkInterfaceListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

impl<'a> InvalidatesOwnCacheKey for NetworkInterfaceListRequest<'a> {}

impl<'a> HasCacheKey for NetworkInterfaceListRequest<'a> {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "network_interfaces",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }
}

impl<'a> ExecutableRequest for NetworkInterfaceListRequest<'a> {
    type Error = eyre::Report;
    type Output = Vec<Arc<AzureNetworkInterfaceResource>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            info!(%self.auth_context.tenant_id, "Fetching network interfaces");
            let query = indoc! {r#"
        Resources
        | where type == "microsoft.network/networkinterfaces"
        | project
            id,
            tenantId,
            name,
            location,
            managedBy,
            tags,
            properties
        "#}
            .to_owned();

            let network_interfaces = ResourceGraphHelper::new(
                query,
                Some(self.cache_key()?),
                self.auth_context.tenant_id,
                self.auth_context.auth_source()?,
            )
            .collect_all::<AzureNetworkInterfaceResource>(&executor)
            .await?;
            info!(
                count = network_interfaces.len(),
                "Fetched network interfaces"
            );
            Ok(network_interfaces)
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
                let result = fetch_all_network_interfaces(
                    &AuthContext::explicit_azure_cli()
                        .bind_to_azure_tenant(get_test_tenant_id(&cancellation).await?)?,
                    &executor,
                )
                .await?;
                for network_interface in &result {
                    assert!(!network_interface.name.is_empty());
                }
                Ok(())
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(NetworkInterfaceListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(NetworkInterfaceListRequest<'static>);
cloud_terrastodon_registry_execution::register_executable_request!(NetworkInterfaceListRequest<'static> => Vec<Arc<AzureNetworkInterfaceResource>>, effects = [Read]);

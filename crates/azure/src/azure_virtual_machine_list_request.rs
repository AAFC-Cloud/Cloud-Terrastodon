use crate::ResourceGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::VirtualMachine;
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
pub struct AzureVirtualMachineListRequest {
    pub authentication: AzureBearerAuthenticationSelection,
}

impl InvalidatesOwnCacheKey for AzureVirtualMachineListRequest {}

impl HasCacheKey for AzureVirtualMachineListRequest {
    fn cache_key(&self) -> eyre::Result<CacheKey> {
        CacheKey::from_relative_path(PathBuf::from_iter([
            "az",
            "resource_graph",
            "virtual_machines",
            self.authentication.tenant_id.to_string().as_str(),
        ]))
    }
}

impl ExecutableRequest for AzureVirtualMachineListRequest {
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor, Self::Error> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    type Error = eyre::Report;
    type Output = Vec<Arc<VirtualMachine>>;
    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move {
            debug!(fetching = "virtual machines");
            let query = indoc! {r#"
            Resources
            | where type == "microsoft.compute/virtualmachines"
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

            let virtual_machines =
                ResourceGraphHelper::new(query, Some(self.cache_key()?), self.authentication)
                    .collect_all::<VirtualMachine>(&executor)
                    .await?;
            debug!(count = virtual_machines.len(), "Found virtual machines");
            Ok(virtual_machines)
        })
    }
}

cloud_terrastodon_registry::register_thing!(AzureVirtualMachineListRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureVirtualMachineListRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureVirtualMachineListRequest => Vec<Arc<VirtualMachine>>, effects = [Read]);

cloud_terrastodon_registry_execution::register_runtime_test!(
    AzureVirtualMachineListRequest,
    name = "virtual-machine-list",
    cached
);

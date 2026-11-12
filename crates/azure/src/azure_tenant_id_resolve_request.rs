use crate::AzureTenantArgumentExt;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantArgument;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;

/// Resolve an explicit tenant ID, tracked alias, or configured Cloud Terrastodon default.
#[derive(Debug, Clone, Arbitrary, Facet)]
pub struct AzureTenantIdResolveRequest {
    pub tenant: AzureTenantArgument<'static>,
}

impl ExecutableRequest for AzureTenantIdResolveRequest {
    type Output = AzureTenantId;
    type Error = eyre::Report;

    fn run<'exec>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'exec, Self::Output>
    where
        Self: 'exec,
        Self::Output: 'exec,
    {
        Box::pin(async move { self.tenant.resolve(executor.cancellation()).await })
    }
}

cloud_terrastodon_registry::register_thing!(AzureTenantIdResolveRequest);
cloud_terrastodon_registry::register_arbitrary!(AzureTenantIdResolveRequest);
cloud_terrastodon_registry_execution::register_executable_request!(AzureTenantIdResolveRequest => AzureTenantId, effects = [Read]);

use crate::get_tags_for_resources;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ResourceTagsId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use facet::Facet;

/// Verify typed tag reads for explicitly selected resources through the ARM batch workflow.
/// The batch transport does not reuse responses, so this case has no cached reads
/// to invalidate and never selects resources from a live inventory automatically.
#[derive(Debug, Facet)]
pub struct AzureResourceTagsVerificationRequest {
    pub authentication: AzureBearerAuthenticationSelection,
    pub resource_ids: Vec<ResourceTagsId>,
}

impl ExecutableRequest for AzureResourceTagsVerificationRequest {
    type Output = ();
    type Error = eyre::Report;
    fn bind_authentication(
        &self,
        executor: CloudTerrastodonExecutor,
    ) -> eyre::Result<CloudTerrastodonExecutor> {
        executor.with_authentication_lease(&self.authentication.into())
    }
    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, ()>
    where
        Self: 'a,
    {
        Box::pin(async move {
            eyre::ensure!(
                !self.resource_ids.is_empty(),
                "tag verification requires explicit resource IDs"
            );
            get_tags_for_resources(&self.authentication, self.resource_ids, &executor).await?;
            Ok(())
        })
    }
}

cloud_terrastodon_registry_execution::register_runtime_test!(AzureResourceTagsVerificationRequest => (), name = "resource-tags", effects = [Read], uncached);

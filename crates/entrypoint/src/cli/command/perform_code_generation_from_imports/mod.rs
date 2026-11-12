use crate::noninteractive::perform_import;
use crate::noninteractive::process_generated;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Perform code-generation from existing import definitions.
#[derive(Facet, Debug, Clone, Default)]
pub struct PerformCodeGenerationFromImportsArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl PerformCodeGenerationFromImportsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let cancellation = executor.cancellation();
        let authentication = self.authentication.resolve(executor).await?;
        perform_import(cancellation).await?;
        process_generated(&authentication, executor).await?;
        Ok(())
    }
}

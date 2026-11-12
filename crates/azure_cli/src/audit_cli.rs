use crate::AzureAuditReport;
use crate::audit_azure;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// Arguments for auditing Azure resources.
#[derive(Facet, Debug, Clone)]
pub struct AzureAuditArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
}

impl AzureAuditArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        let report = audit_azure(authentication, executor).await?;
        Ok(CliOutput::facet_with_text(
            report,
            AzureAuditReport::write_text,
        ))
    }
}

use crate::AzureStorageAccountAuditReport;
use crate::AzureStorageAccountAuditRequest;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

/// Audit storage accounts visible to the selected account and tenant.
#[derive(Debug, Clone, Facet)]
pub struct AzureStorageAccountAuditArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Refresh account settings and every account's cached container pages.
    #[facet(figue::named, default)]
    pub clean: bool,
}

impl AzureStorageAccountAuditArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let report = AzureStorageAccountAuditRequest {
            authentication: self.authentication.resolve(executor).await?,
        }
        .execute(executor)
        .with_invalidation(self.clean)
        .await?;
        Ok(CliOutput::facet_with_text(
            report,
            AzureStorageAccountAuditReport::write_text,
        ))
    }
}

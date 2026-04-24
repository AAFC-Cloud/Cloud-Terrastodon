use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_cognitive_services_accounts;
use eyre::Result;
use std::io::Write;

/// Arguments for listing Azure Cognitive Services accounts.
#[derive(Args, Debug, Clone)]
pub struct AzureCognitiveServicesListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureCognitiveServicesListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let accounts = fetch_all_cognitive_services_accounts(tenant_id).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &accounts)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

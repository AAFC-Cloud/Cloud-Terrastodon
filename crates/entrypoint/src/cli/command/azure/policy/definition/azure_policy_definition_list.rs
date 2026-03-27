use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_policy_definitions;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure policy definitions.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyDefinitionListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzurePolicyDefinitionListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching Azure policy definitions...");
        let policy_definitions = fetch_all_policy_definitions(tenant_id).await?;
        info!(
            count = policy_definitions.len(),
            "Fetched Azure policy definitions",
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &policy_definitions)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

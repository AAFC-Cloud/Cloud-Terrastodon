use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use eyre::bail;

/// Arguments for logging in to an Azure tenant via the Azure CLI.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantLoginArgs {
    /// Tracked tenant id or alias to log in to.
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureTenantLoginArgs {
    pub async fn invoke(self) -> Result<()> {
        if std::env::var("CLOUD_TERRASTODON_REAUTH")
            .unwrap_or_default()
            .to_uppercase()
            == "DENY"
        {
            bail!(
                "Reauthentication is disabled by the CLOUD_TERRASTODON_REAUTH environment variable. Please refresh your credentials and try again."
            )
        }
        let tenant_id = self.tenant.resolve().await?;
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["login", "--tenant", &tenant_id.to_string()]);
        cmd.should_announce(true);
        cmd.run_raw().await?;
        Ok(())
    }
}

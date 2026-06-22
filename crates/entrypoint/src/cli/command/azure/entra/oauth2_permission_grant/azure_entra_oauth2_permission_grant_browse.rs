use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::pick_oauth2_permission_grants;
use eyre::Result;
use std::io::Write;

/// Browse Entra OAuth2 permission grants interactively.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraOAuth2PermissionGrantBrowseArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraOAuth2PermissionGrantBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let chosen = pick_oauth2_permission_grants(self.tenant.resolve().await?).await?;
        let chosen = chosen
            .into_iter()
            .map(|grant| grant.grant)
            .collect::<Vec<_>>();
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

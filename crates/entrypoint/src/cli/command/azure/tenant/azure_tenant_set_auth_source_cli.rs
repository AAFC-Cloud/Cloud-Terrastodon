use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::set_tracked_tenant_auth_source;
use cloud_terrastodon_credentials::AuthSource;
use eyre::Result;
use std::io::Write;

/// Set the default authentication source for a tracked Azure tenant.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureTenantSetAuthSourceArgs {
    /// Tracked tenant id or alias to configure.
    #[facet(figue::positional)]
    pub tenant: AzureTenantArgument<'static>,

    /// Authentication source to use when no global source is specified.
    #[facet(figue::positional)]
    pub auth_source: AuthSource,
}

impl AzureTenantSetAuthSourceArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let auth_source = set_tracked_tenant_auth_source(tenant_id, self.auth_source).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &auth_source)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::set_tracked_tenant_auth_source;
use cloud_terrastodon_command::CancellationScope;
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
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        let tenant_id = self.tenant.resolve(cancellation).await?;
        let auth_source = set_tracked_tenant_auth_source(tenant_id, self.auth_source).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &auth_source)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

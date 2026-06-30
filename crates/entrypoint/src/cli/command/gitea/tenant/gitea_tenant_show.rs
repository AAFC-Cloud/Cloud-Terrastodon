use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::get_tracked_tenant;
use eyre::Result;
use eyre::bail;
use std::io::Write;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaTenantShowArgs {
    /// Tracked tenant URL or alias to show.
    #[facet(opaque, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaTenantShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_url = self.tenant.resolve().await?;
        let Some(tenant) = get_tracked_tenant(&tenant_url).await? else {
            bail!("Tracked Gitea tenant '{}' was not found.", tenant_url);
        };
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &tenant)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

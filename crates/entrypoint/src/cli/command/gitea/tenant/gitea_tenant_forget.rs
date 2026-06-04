use clap::Args;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::forget_tracked_tenant;
use eyre::Result;
use std::io::Write;

#[derive(Args, Debug, Clone)]
pub struct GiteaTenantForgetArgs {
    /// Tracked tenant URL or alias to forget.
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaTenantForgetArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_url = self.tenant.resolve().await?;
        let forgotten = forget_tracked_tenant(&tenant_url).await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &forgotten)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

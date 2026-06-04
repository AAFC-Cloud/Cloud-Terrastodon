use clap::Args;
use cloud_terrastodon_gitea::GiteaInstanceUrl;
use cloud_terrastodon_gitea::add_tracked_tenant;
use eyre::Result;
use std::io::Write;

#[derive(Args, Debug, Clone)]
pub struct GiteaTenantAddArgs {
    /// Gitea instance URL to track.
    pub tenant: GiteaInstanceUrl,
}

impl GiteaTenantAddArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = add_tracked_tenant(self.tenant).await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &tenant)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

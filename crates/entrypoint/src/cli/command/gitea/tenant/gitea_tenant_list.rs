use clap::Args;
use cloud_terrastodon_gitea::list_tracked_tenants;
use eyre::Result;
use std::io::Write;

#[derive(Args, Debug, Clone)]
pub struct GiteaTenantListArgs {}

impl GiteaTenantListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenants = list_tracked_tenants().await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &tenants)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_gitea::discover_and_track_tenants;
use eyre::Result;
use std::io::Write;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaTenantDiscoverArgs {}

impl GiteaTenantDiscoverArgs {
    pub async fn invoke(
        self,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let tenants = discover_and_track_tenants(cancellation).await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &tenants)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

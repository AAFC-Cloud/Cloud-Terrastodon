use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::discover_and_track_tenants;
use eyre::Result;
use facet::Facet;
use std::io::Write;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantDiscoverArgs {}

impl GiteaTenantDiscoverArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenants = discover_and_track_tenants(executor).await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &tenants)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

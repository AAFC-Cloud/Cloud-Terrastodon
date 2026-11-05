use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::forget_tracked_tenant;
use eyre::Result;
use std::io::Write;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaTenantForgetArgs {
    /// Tracked tenant URL or alias to forget.
    #[facet(figue::positional, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaTenantForgetArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenant_url = self.tenant.resolve(executor).await?;
        let forgotten = forget_tracked_tenant(&tenant_url).await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &forgotten)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

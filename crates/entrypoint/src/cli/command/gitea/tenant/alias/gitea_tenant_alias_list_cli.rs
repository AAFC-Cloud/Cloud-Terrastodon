use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::list_tracked_tenant_aliases;
use cloud_terrastodon_gitea::list_tracked_tenant_aliases_for;
use eyre::Result;
use facet::Facet;
use std::io::Write;

#[derive(Facet, Debug, Clone)]
pub struct GiteaTenantAliasListArgs {
    /// Optional tracked tenant URL or alias to filter by.
    #[facet(figue::named)]
    pub tenant: Option<GiteaTenantArgument<'static>>,
}

impl GiteaTenantAliasListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();

        if let Some(tenant) = self.tenant {
            let tenant_url = tenant.resolve(executor).await?;
            let mut aliases = list_tracked_tenant_aliases_for(&tenant_url).await?;
            aliases.sort();
            facet_json::to_writer_std_pretty(&mut handle, &aliases)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
        } else {
            let aliases = list_tracked_tenant_aliases().await?;
            facet_json::to_writer_std_pretty(&mut handle, &aliases)
                .map_err(|error| eyre::eyre!("{error:?}"))?;
        }

        handle.write_all(b"\n")?;
        Ok(())
    }
}

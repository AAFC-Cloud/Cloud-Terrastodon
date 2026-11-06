use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaOrganizationArgument;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_organizations;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::io::Write;

#[derive(Facet, Debug, Clone)]
pub struct GiteaOrgShowArgs {
    /// Organization id or name.
    #[facet(figue::positional, proxy = String)]
    pub organization: GiteaOrganizationArgument<'static>,

    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[facet(figue::named, default, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaOrgShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenant = self.tenant.resolve(executor).await?;
        let organizations = fetch_all_gitea_organizations(&tenant, executor).await?;
        let matches = organizations
            .into_iter()
            .filter(|organization| self.organization.matches(organization))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [organization] => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                facet_json::to_writer_std_pretty(&mut handle, organization)
                    .map_err(|error| eyre::eyre!("{error:?}"))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            [] => {
                bail!(
                    "No Gitea organization found matching '{}'.",
                    self.organization
                );
            }
            _ => {
                bail!(
                    "Multiple Gitea organizations matched '{}'. Please specify the numeric id.",
                    self.organization
                );
            }
        }
    }
}

use cloud_terrastodon_gitea::GiteaOrganizationArgument;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_organizations;
use eyre::Result;
use eyre::bail;
use std::io::Write;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaOrgShowArgs {
    /// Organization id or name.
    #[facet(figue::positional, opaque, proxy = String)]
    pub organization: GiteaOrganizationArgument<'static>,

    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[facet(figue::named, default, opaque, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaOrgShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.tenant.resolve().await?;
        let organizations = fetch_all_gitea_organizations(&tenant).await?;
        let matches = organizations
            .into_iter()
            .filter(|organization| self.organization.matches(organization))
            .collect::<Vec<_>>();
        match matches.as_slice() {
            [organization] => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                cloud_terrastodon_command::to_writer_pretty(&mut handle, organization)?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            [] => bail!(
                "No Gitea organization found matching '{}'.",
                self.organization
            ),
            _ => bail!(
                "Multiple Gitea organizations matched '{}'. Please specify the numeric id.",
                self.organization
            ),
        }
    }
}

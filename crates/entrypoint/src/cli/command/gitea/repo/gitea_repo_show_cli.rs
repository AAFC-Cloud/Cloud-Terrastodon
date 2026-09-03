use cloud_terrastodon_gitea::GiteaRepoArgument;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_repositories;
use cloud_terrastodon_gitea::fetch_gitea_repository;
use cloud_terrastodon_gitea::fetch_gitea_repository_by_id;
use eyre::Result;
use eyre::bail;
use std::io::Write;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaRepoShowArgs {
    /// Repository id, full name, or name.
    #[facet(figue::positional, proxy = String)]
    pub repo: GiteaRepoArgument<'static>,

    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[facet(figue::named, default, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaRepoShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.tenant.resolve().await?;
        let repository = match &self.repo {
            GiteaRepoArgument::Id(repo_id) => {
                fetch_gitea_repository_by_id(&tenant, *repo_id.as_ref()).await?
            }
            GiteaRepoArgument::FullName(full_name) => {
                fetch_gitea_repository(&tenant, full_name.as_ref()).await?
            }
            GiteaRepoArgument::Name(_) => {
                let repositories = fetch_all_gitea_repositories(&tenant).await?;
                let matches = repositories
                    .into_iter()
                    .filter(|repo| self.repo.matches(repo))
                    .collect::<Vec<_>>();
                match matches.as_slice() {
                    [repo] => repo.clone(),
                    [] => bail!("No Gitea repository found matching '{}'.", self.repo),
                    _ => bail!(
                        "Multiple Gitea repositories matched '{}'. Please specify the numeric id or owner/repo.",
                        self.repo
                    ),
                }
            }
        };
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &repository)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

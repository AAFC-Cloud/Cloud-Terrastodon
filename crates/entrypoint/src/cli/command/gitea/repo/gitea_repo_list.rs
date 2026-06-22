use clap::Args;
use clap::ValueEnum;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::fetch_all_gitea_organization_repositories;
use cloud_terrastodon_gitea::fetch_all_gitea_organizations;
use cloud_terrastodon_gitea::fetch_all_gitea_repositories;
use cloud_terrastodon_gitea::fetch_all_gitea_repositories_via_search;
use cloud_terrastodon_gitea::fetch_all_gitea_user_repositories;
use cloud_terrastodon_gitea::fetch_all_gitea_users;
use cloud_terrastodon_gitea::fetch_current_user_gitea_repositories;
use cloud_terrastodon_gitea::fetch_gitea_repositories_by_id_range;
use eyre::Result;
use std::io::Write;
use tracing::info;

#[derive(ValueEnum, Debug, Clone, Copy, Default)]
pub enum GiteaRepoListMethod {
    #[default]
    Combined,
    Search,
    Organizations,
    Users,
    CurrentUser,
    IdRange,
}

#[derive(Args, Debug, Clone)]
pub struct GiteaRepoListArgs {
    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[arg(long, default_value_t)]
    pub tenant: GiteaTenantArgument<'static>,

    /// Enumeration method to use.
    #[arg(long, value_enum, default_value_t)]
    pub method: GiteaRepoListMethod,

    /// Upper bound to use when `--method id-range` is selected.
    #[arg(long, default_value_t = 1000)]
    pub max_repo_id: u64,
}

impl GiteaRepoListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.tenant.resolve().await?;
        let repositories = self.fetch_repositories(&tenant).await?;
        info!(count = repositories.len(), ?self.method, "Fetched Gitea repositories");
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &repositories)?;
        handle.write_all(b"\n")?;
        Ok(())
    }

    pub async fn fetch_repositories(
        &self,
        tenant: &cloud_terrastodon_gitea::GiteaInstanceUrl,
    ) -> Result<Vec<cloud_terrastodon_gitea::GiteaRepo>> {
        match self.method {
            GiteaRepoListMethod::Combined => fetch_all_gitea_repositories(tenant).await,
            GiteaRepoListMethod::Search => fetch_all_gitea_repositories_via_search(tenant).await,
            GiteaRepoListMethod::Organizations => {
                let organizations = fetch_all_gitea_organizations(tenant).await?;
                let mut repositories = Vec::new();
                for organization in &organizations {
                    repositories.extend(
                        fetch_all_gitea_organization_repositories(tenant, &organization.username)
                            .await?,
                    );
                }
                Ok(cloud_terrastodon_gitea::dedupe_repositories(repositories))
            }
            GiteaRepoListMethod::Users => {
                let users = fetch_all_gitea_users(tenant).await?;
                let mut repositories = Vec::new();
                for user in &users {
                    repositories
                        .extend(fetch_all_gitea_user_repositories(tenant, &user.login).await?);
                }
                Ok(cloud_terrastodon_gitea::dedupe_repositories(repositories))
            }
            GiteaRepoListMethod::CurrentUser => fetch_current_user_gitea_repositories(tenant).await,
            GiteaRepoListMethod::IdRange => {
                fetch_gitea_repositories_by_id_range(tenant, 1, self.max_repo_id).await
            }
        }
    }
}

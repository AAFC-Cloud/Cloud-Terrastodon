use cloud_terrastodon_execution::CloudTerrastodonExecutor;
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

#[derive(facet::Facet, Debug, Clone, Copy, Default)]
#[repr(u8)]
pub enum GiteaRepoListMethod {
    #[default]
    Combined,
    Search,
    Organizations,
    Users,
    CurrentUser,
    IdRange,
}

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaRepoListArgs {
    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[facet(figue::named, default, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,

    /// Enumeration method to use.
    #[facet(figue::named, default)]
    pub method: GiteaRepoListMethod,

    /// Upper bound to use when `--method id-range` is selected.
    #[facet(figue::named, default = 1000)]
    pub max_repo_id: u64,
}

impl GiteaRepoListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenant = self.tenant.resolve(executor).await?;
        let repositories = self.fetch_repositories(&tenant, executor).await?;
        info!(count = repositories.len(), ?self.method, "Fetched Gitea repositories");
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &repositories)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }

    pub async fn fetch_repositories(
        &self,
        tenant: &cloud_terrastodon_gitea::GiteaInstanceUrl,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Vec<cloud_terrastodon_gitea::GiteaRepo>> {
        match self.method {
            GiteaRepoListMethod::Combined => fetch_all_gitea_repositories(tenant, executor).await,
            GiteaRepoListMethod::Search => {
                fetch_all_gitea_repositories_via_search(tenant, executor).await
            }
            GiteaRepoListMethod::Organizations => {
                let organizations = fetch_all_gitea_organizations(tenant, executor).await?;
                let mut repositories = Vec::new();
                for organization in &organizations {
                    repositories.extend(
                        fetch_all_gitea_organization_repositories(
                            tenant,
                            &organization.username,
                            executor,
                        )
                        .await?,
                    );
                }
                Ok(cloud_terrastodon_gitea::dedupe_repositories(repositories))
            }
            GiteaRepoListMethod::Users => {
                let users = fetch_all_gitea_users(tenant, executor).await?;
                let mut repositories = Vec::new();
                for user in &users {
                    repositories.extend(
                        fetch_all_gitea_user_repositories(tenant, &user.login, executor).await?,
                    );
                }
                Ok(cloud_terrastodon_gitea::dedupe_repositories(repositories))
            }
            GiteaRepoListMethod::CurrentUser => {
                fetch_current_user_gitea_repositories(tenant, executor).await
            }
            GiteaRepoListMethod::IdRange => {
                fetch_gitea_repositories_by_id_range(tenant, 1, self.max_repo_id, executor).await
            }
        }
    }
}

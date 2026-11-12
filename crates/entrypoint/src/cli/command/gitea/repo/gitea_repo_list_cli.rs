use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_gitea::GiteaOrganizationListRequest;
use cloud_terrastodon_gitea::GiteaOrganizationRepoListRequest;
use cloud_terrastodon_gitea::GiteaRepoListRequest;
use cloud_terrastodon_gitea::GiteaRepoScanByIdRequest;
use cloud_terrastodon_gitea::GiteaRepoSearchRequest;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::GiteaUserCurrentRepoListRequest;
use cloud_terrastodon_gitea::GiteaUserListRequest;
use cloud_terrastodon_gitea::GiteaUserRepoListRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use tracing::info;

#[derive(Facet, Debug, Clone, Copy, Default)]
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

#[derive(Facet, Debug, Clone)]
pub struct GiteaRepoListArgs {
    /// Tracked tenant URL or alias to query. Select an instance explicitly; account defaults do not select service scope.
    #[facet(figue::named, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
    /// Account ID or alias for this instance; omission selects its configured default.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,

    /// Enumeration method to use.
    #[facet(figue::named, default)]
    pub method: GiteaRepoListMethod,

    /// Upper bound to use when `--method id-range` is selected.
    #[facet(figue::named, default = 1000)]
    pub max_repo_id: u64,
}

impl GiteaRepoListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant = self.tenant.resolve(executor.cancellation()).await?;
        let account = self.account.as_ref().map(ToString::to_string);
        let account_id =
            executor.resolve_gitea_authentication(tenant.as_ref(), account.as_deref())?;
        let repositories = self
            .fetch_repositories(&tenant, account_id, executor)
            .await?;
        info!(count = repositories.len(), ?self.method, "Fetched Gitea repositories");
        Ok(CliOutput::facet(repositories))
    }

    pub async fn fetch_repositories(
        &self,
        tenant: &cloud_terrastodon_gitea::GiteaInstanceUrl,
        account_id: CloudTerrastodonAccountId,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Vec<cloud_terrastodon_gitea::GiteaRepo>> {
        match self.method {
            GiteaRepoListMethod::Combined => {
                GiteaRepoListRequest {
                    account_id,
                    tenant: Cow::Borrowed(tenant),
                }
                .execute(executor)
                .await
            }
            GiteaRepoListMethod::Search => {
                GiteaRepoSearchRequest {
                    account_id,
                    tenant: Cow::Borrowed(tenant),
                }
                .execute(executor)
                .await
            }
            GiteaRepoListMethod::Organizations => {
                let organizations = GiteaOrganizationListRequest {
                    account_id,
                    tenant: Cow::Borrowed(tenant),
                }
                .execute(executor)
                .await?;
                let mut repositories = Vec::new();
                for organization in &organizations {
                    repositories.extend(
                        GiteaOrganizationRepoListRequest {
                            account_id,
                            tenant: Cow::Borrowed(tenant),
                            organization_name: Cow::Borrowed(&organization.username),
                        }
                        .execute(executor)
                        .await?,
                    );
                }
                Ok(cloud_terrastodon_gitea::dedupe_repositories(repositories))
            }
            GiteaRepoListMethod::Users => {
                let users = GiteaUserListRequest {
                    account_id,
                    tenant: Cow::Borrowed(tenant),
                }
                .execute(executor)
                .await?;
                let mut repositories = Vec::new();
                for user in &users {
                    repositories.extend(
                        GiteaUserRepoListRequest {
                            account_id,
                            tenant: Cow::Borrowed(tenant),
                            username: Cow::Borrowed(&user.login),
                        }
                        .execute(executor)
                        .await?,
                    );
                }
                Ok(cloud_terrastodon_gitea::dedupe_repositories(repositories))
            }
            GiteaRepoListMethod::CurrentUser => {
                GiteaUserCurrentRepoListRequest {
                    account_id,
                    tenant: Cow::Borrowed(tenant),
                }
                .execute(executor)
                .await
            }
            GiteaRepoListMethod::IdRange => {
                GiteaRepoScanByIdRequest {
                    account_id,
                    tenant: Cow::Borrowed(tenant),
                    start_id: (1),
                    end_id: (self.max_repo_id),
                }
                .execute(executor)
                .await
            }
        }
    }
}

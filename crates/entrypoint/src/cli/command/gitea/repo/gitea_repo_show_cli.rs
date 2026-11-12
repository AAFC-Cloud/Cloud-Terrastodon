use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_gitea::GiteaRepoArgument;
use cloud_terrastodon_gitea::GiteaRepoGetByIdRequest;
use cloud_terrastodon_gitea::GiteaRepoGetRequest;
use cloud_terrastodon_gitea::GiteaRepoListRequest;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

#[derive(Facet, Debug, Clone)]
pub struct GiteaRepoShowArgs {
    /// Repository id, full name, or name.
    #[facet(figue::positional, proxy = String)]
    pub repo: GiteaRepoArgument<'static>,

    /// Tracked tenant URL or alias to query. Select an instance explicitly; account defaults do not select service scope.
    #[facet(figue::named, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
    /// Account ID or alias for this instance; omission selects its configured default.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
}

impl GiteaRepoShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant = self.tenant.resolve(executor.cancellation()).await?;
        let account = self.account.as_ref().map(ToString::to_string);
        let account_id =
            executor.resolve_gitea_authentication(tenant.as_ref(), account.as_deref())?;
        let repository = match &self.repo {
            GiteaRepoArgument::Id(repo_id) => {
                GiteaRepoGetByIdRequest {
                    account_id,
                    tenant: Cow::Borrowed(&tenant),
                    repo_id: (*repo_id.as_ref()),
                }
                .execute(executor)
                .await?
            }
            GiteaRepoArgument::FullName(full_name) => {
                GiteaRepoGetRequest {
                    account_id,
                    tenant: Cow::Borrowed(&tenant),
                    repo_full_name: Cow::Borrowed(full_name.as_ref()),
                }
                .execute(executor)
                .await?
            }
            GiteaRepoArgument::Name(_) => {
                let repositories = GiteaRepoListRequest {
                    account_id,
                    tenant: Cow::Borrowed(&tenant),
                }
                .execute(executor)
                .await?;
                let matches = repositories
                    .into_iter()
                    .filter(|repo| self.repo.matches(repo))
                    .collect::<Vec<_>>();
                match matches.as_slice() {
                    [repo] => repo.clone(),
                    [] => {
                        bail!("No Gitea repository found matching '{}'.", self.repo);
                    }
                    _ => {
                        bail!(
                            "Multiple Gitea repositories matched '{}'. Please specify the numeric id or owner/repo.",
                            self.repo
                        );
                    }
                }
            }
        };
        Ok(CliOutput::facet(repository))
    }
}

use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::GiteaUserArgument;
use cloud_terrastodon_gitea::GiteaUserGetRequest;
use cloud_terrastodon_gitea::GiteaUserListRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::borrow::Cow;

#[derive(Facet, Debug, Clone)]
pub struct GiteaUserShowArgs {
    /// User id or username.
    #[facet(figue::positional, proxy = String)]
    pub user: GiteaUserArgument<'static>,

    /// Tracked tenant URL or alias to query. Select an instance explicitly; account defaults do not select service scope.
    #[facet(figue::named, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
    /// Account ID or alias for this instance; omission selects its configured default.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
}

impl GiteaUserShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant = self.tenant.resolve(executor.cancellation()).await?;
        let account = self.account.as_ref().map(ToString::to_string);
        let account_id =
            executor.resolve_gitea_authentication(tenant.as_ref(), account.as_deref())?;
        let user = match &self.user {
            GiteaUserArgument::Username(username) => {
                GiteaUserGetRequest {
                    account_id,
                    tenant: Cow::Borrowed(&tenant),
                    username: Cow::Borrowed(username.as_ref()),
                }
                .execute(executor)
                .await?
            }
            GiteaUserArgument::Id(_) => {
                let users = GiteaUserListRequest {
                    account_id,
                    tenant: Cow::Borrowed(&tenant),
                }
                .execute(executor)
                .await?;
                let matches = users
                    .into_iter()
                    .filter(|user| self.user.matches(user))
                    .collect::<Vec<_>>();
                match matches.as_slice() {
                    [user] => user.clone(),
                    [] => {
                        bail!("No Gitea user found matching '{}'.", self.user);
                    }
                    _ => {
                        bail!(
                            "Multiple Gitea users matched '{}'. Please specify the numeric id.",
                            self.user
                        );
                    }
                }
            }
        };

        Ok(CliOutput::facet(user))
    }
}

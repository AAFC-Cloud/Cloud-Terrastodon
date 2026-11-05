use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::GiteaUserArgument;
use cloud_terrastodon_gitea::fetch_all_gitea_users;
use cloud_terrastodon_gitea::fetch_gitea_user;
use eyre::Result;
use eyre::bail;
use std::io::Write;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaUserShowArgs {
    /// User id or username.
    #[facet(figue::positional, proxy = String)]
    pub user: GiteaUserArgument<'static>,

    /// Tracked tenant URL or alias to query. Defaults to the active `tea` login.
    #[facet(figue::named, default, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
}

impl GiteaUserShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenant = self.tenant.resolve(executor).await?;
        let user = match &self.user {
            GiteaUserArgument::Username(username) => {
                fetch_gitea_user(&tenant, username.as_ref(), executor).await?
            }
            GiteaUserArgument::Id(_) => {
                let users = fetch_all_gitea_users(&tenant, executor).await?;
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

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &user)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

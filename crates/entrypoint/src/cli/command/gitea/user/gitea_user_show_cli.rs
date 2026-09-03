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
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.tenant.resolve().await?;
        let user = match &self.user {
            GiteaUserArgument::Username(username) => {
                fetch_gitea_user(&tenant, username.as_ref()).await?
            }
            GiteaUserArgument::Id(_) => {
                let users = fetch_all_gitea_users(&tenant).await?;
                let matches = users
                    .into_iter()
                    .filter(|user| self.user.matches(user))
                    .collect::<Vec<_>>();
                match matches.as_slice() {
                    [user] => user.clone(),
                    [] => bail!("No Gitea user found matching '{}'.", self.user),
                    _ => bail!(
                        "Multiple Gitea users matched '{}'. Please specify the numeric id.",
                        self.user
                    ),
                }
            }
        };

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &user)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

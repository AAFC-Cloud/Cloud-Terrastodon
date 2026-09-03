use crate::cli::gitea::org::GiteaOrgArgs;
use crate::cli::gitea::repo::GiteaRepoArgs;
use crate::cli::gitea::tenant::GiteaTenantArgs;
use crate::cli::gitea::user::GiteaUserArgs;
use eyre::Result;

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum GiteaCommand {
    /// Manage tracked Gitea tenants (instances).
    Tenant(GiteaTenantArgs),
    /// Organization-related commands.
    #[facet(figue::alias = "orgs")]
    Org(GiteaOrgArgs),
    /// User-related commands.
    #[facet(figue::alias = "users")]
    User(GiteaUserArgs),
    /// Repository-related commands.
    #[facet(figue::alias = "repos")]
    Repo(GiteaRepoArgs),
}

impl GiteaCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            GiteaCommand::Tenant(args) => args.invoke().await?,
            GiteaCommand::Org(args) => args.invoke().await?,
            GiteaCommand::User(args) => args.invoke().await?,
            GiteaCommand::Repo(args) => args.invoke().await?,
        }
        Ok(())
    }
}

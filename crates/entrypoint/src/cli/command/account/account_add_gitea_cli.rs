use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_gitea::GiteaInstanceUrl;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
/// Bind an explicit external tea profile to a named Gitea account.
#[derive(Debug, Facet)]
pub struct AccountAddGiteaArgs {
    #[facet(figue::named, proxy = String)]
    pub name: CloudTerrastodonAccountAlias,
    #[facet(figue::named)]
    pub instance: GiteaInstanceUrl,
    #[facet(figue::named)]
    pub login: String,
    /// Explicit tea configuration containing only this login; captured by account login.
    #[facet(figue::named)]
    pub configuration: PathBuf,
}
impl AccountAddGiteaArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        Ok(CliOutput::facet(
            CloudTerrastodonAccountStore::configured()
                .add(
                    self.name,
                    CloudTerrastodonAccountCredential::Gitea {
                        instance: self.instance.to_string().parse()?,
                        login: self.login,
                        configuration: std::path::absolute(self.configuration)?.try_into()?,
                    },
                    cancellation,
                )
                .await?,
        ))
    }
}

use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_bmc_helix_types::BmcHelixUsername;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Register a BMC Helix account for use across tracked servers.
#[derive(Debug, Facet)]
pub struct AccountAddHelixArgs {
    /// Local account alias.
    #[facet(figue::named, proxy = String)]
    pub name: CloudTerrastodonAccountAlias,
    /// Username used to sign in to BMC Helix.
    #[facet(figue::named)]
    pub username: BmcHelixUsername,
}

impl AccountAddHelixArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let profile = CloudTerrastodonAccountStore::configured()
            .add(
                self.name,
                CloudTerrastodonAccountCredential::BmcHelix {
                    server: None,
                    username: self.username,
                },
                cancellation,
            )
            .await?;
        Ok(CliOutput::facet(profile))
    }
}

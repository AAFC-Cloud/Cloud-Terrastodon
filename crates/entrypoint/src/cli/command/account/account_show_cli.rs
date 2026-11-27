use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::BmcHelixAccountSession;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountShowArgs {
    /// Account UUID or alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
    /// Show sign-in status on this BMC Helix server.
    #[facet(figue::named)]
    pub server: Option<BmcHelixServerName>,
}
impl AccountShowArgs {
    pub async fn invoke(self, _cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        if let Some(server) = &self.server {
            let server = BmcHelixServerStore::configured().get(Some(server))?;
            let account_id = store
                .resolve_bmc_helix(&server.url, Some(&self.account))?
                .id;
            let session = store.load()?.bmc_helix_session(account_id, &server.url);
            return Ok(CliOutput::facet(BmcHelixAccountSession {
                account_id,
                server: server.url,
                session,
            }));
        }
        let catalogue = store.load()?;
        let account = catalogue.resolve(Some(&self.account), None)?;
        Ok(CliOutput::facet(account.clone()))
    }
}

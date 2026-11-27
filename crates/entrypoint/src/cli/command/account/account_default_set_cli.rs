use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountDefaultSetArgs {
    /// Account UUID or alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
    /// Save the default account for this BMC Helix server.
    #[facet(figue::named)]
    pub server: Option<BmcHelixServerName>,
}
impl AccountDefaultSetArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        if let Some(server) = &self.server {
            let server = BmcHelixServerStore::configured().get(Some(server))?;
            let account_id = store
                .resolve_bmc_helix(&server.url, Some(&self.account))?
                .id;
            store
                .set_bmc_helix_default(&server.url, account_id, cancellation)
                .await?;
            return Ok(CliOutput::facet(account_id));
        }
        let catalogue = store.load()?;
        let account = catalogue.resolve(Some(&self.account), None)?;
        eyre::ensure!(
            !matches!(
                account.namespace,
                CloudTerrastodonAccountNamespace::BmcHelix { .. }
            ),
            "specify --server to set a BMC Helix server's default account"
        );
        store.set_default(account.id, cancellation).await?;
        Ok(CliOutput::facet(account.id))
    }
}

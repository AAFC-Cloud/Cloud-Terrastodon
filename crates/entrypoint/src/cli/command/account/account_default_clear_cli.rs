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
pub struct AccountDefaultClearArgs {
    /// Account UUID or alias. Omit when clearing a BMC Helix server's default.
    #[facet(figue::positional)]
    pub account: Option<CloudTerrastodonAccountArgument>,
    /// Clear this BMC Helix server's default account.
    #[facet(figue::named)]
    pub server: Option<BmcHelixServerName>,
}
impl AccountDefaultClearArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        if let Some(server) = &self.server {
            eyre::ensure!(
                self.account.is_none(),
                "omit the account when clearing the default for a BMC Helix server"
            );
            let server = BmcHelixServerStore::configured().get(Some(server))?;
            store
                .clear_bmc_helix_default(&server.url, cancellation)
                .await?;
            return Ok(CliOutput::facet(server));
        }
        let Some(account) = &self.account else {
            eyre::bail!("specify an account UUID or alias, or a BMC Helix --server");
        };
        let catalogue = store.load()?;
        let account = catalogue.resolve(Some(account), None)?;
        eyre::ensure!(
            !matches!(
                account.namespace,
                CloudTerrastodonAccountNamespace::BmcHelix { .. }
            ),
            "use --server to clear a BMC Helix server's default account"
        );
        store.clear_default(account.id, cancellation).await?;
        Ok(CliOutput::facet(account.id))
    }
}

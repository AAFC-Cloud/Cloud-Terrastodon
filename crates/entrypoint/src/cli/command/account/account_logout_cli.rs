use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountLogoutArgs {
    /// Account UUID or alias. Omit when using --all.
    #[facet(figue::positional)]
    pub account: Option<CloudTerrastodonAccountArgument>,
    /// Sign out every registered account and all of its credential sources.
    #[facet(figue::named, default)]
    pub all: bool,
    /// Retire this mechanism only; omission retires every mechanism on the account.
    #[facet(figue::named)]
    pub source: Option<CloudTerrastodonCredentialSource>,
    /// Sign out only this BMC Helix server.
    #[facet(figue::named)]
    pub server: Option<BmcHelixServerName>,
}
impl AccountLogoutArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        if self.all {
            ensure!(
                self.account.is_none() && self.source.is_none() && self.server.is_none(),
                "--all cannot be combined with an account selector, --source, or --server"
            );
            let accounts = CloudTerrastodonAccountStore::configured()
                .logout_all_with_executor(executor)
                .await?;
            return Ok(CliOutput::facet(accounts));
        }
        let account = self
            .account
            .ok_or_else(|| eyre::eyre!("specify an account UUID or alias, or pass --all"))?;
        let store = CloudTerrastodonAccountStore::configured();
        if let Some(server) = &self.server {
            ensure!(
                self.source
                    .is_none_or(|source| source == CloudTerrastodonCredentialSource::BmcHelix),
                "--server can only be combined with --source helix"
            );
            let server = BmcHelixServerStore::configured().get(Some(server))?;
            let account_id = store.resolve_bmc_helix(&server.url, Some(&account))?.id;
            store
                .logout_bmc_helix_with_executor(account_id, &server.url, executor)
                .await?;
            return Ok(CliOutput::facet(account_id));
        }
        let account_id = store.load()?.resolve(Some(&account), None)?.id;
        store
            .logout_with_executor(account_id, self.source, executor)
            .await?;
        Ok(CliOutput::facet(account_id))
    }
}

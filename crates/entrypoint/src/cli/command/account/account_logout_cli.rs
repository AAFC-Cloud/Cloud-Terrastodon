use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
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
}
impl AccountLogoutArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        if self.all {
            ensure!(
                self.account.is_none() && self.source.is_none(),
                "--all cannot be combined with an account selector or --source"
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
        let account_id = store.load()?.resolve(Some(&account), None)?.id;
        store
            .logout_with_executor(account_id, self.source, executor)
            .await?;
        Ok(CliOutput::facet(account_id))
    }
}

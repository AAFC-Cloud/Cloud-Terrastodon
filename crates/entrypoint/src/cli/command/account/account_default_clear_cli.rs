use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountDefaultClearArgs {
    /// Account UUID or alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
}
impl AccountDefaultClearArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        let account_id = store.load()?.resolve(Some(&self.account), None)?.id;
        store.clear_default(account_id, cancellation).await?;
        Ok(CliOutput::facet(account_id))
    }
}

use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountRemoveArgs {
    /// Account UUID or alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
}
impl AccountRemoveArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        let account_id = store.load()?.resolve(Some(&self.account), None)?.id;
        store.remove_with_executor(account_id, executor).await?;
        Ok(CliOutput::facet(account_id))
    }
}

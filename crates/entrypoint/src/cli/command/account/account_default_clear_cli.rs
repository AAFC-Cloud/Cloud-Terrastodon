use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountDefaultClearArgs {
    #[facet(figue::positional, proxy = String)]
    pub account_id: CloudTerrastodonAccountId,
}
impl AccountDefaultClearArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        store.clear_default(self.account_id, cancellation).await?;
        Ok(CliOutput::facet(self.account_id))
    }
}

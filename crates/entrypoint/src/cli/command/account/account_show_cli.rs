use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountShowArgs {
    #[facet(figue::positional, proxy = String)]
    pub account_id: CloudTerrastodonAccountId,
}
impl AccountShowArgs {
    pub async fn invoke(self, _cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        Ok(CliOutput::facet(
            store.load()?.by_id(self.account_id)?.clone(),
        ))
    }
}

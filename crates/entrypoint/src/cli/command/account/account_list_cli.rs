use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
/// List non-secret account profiles and configured defaults.
#[derive(Debug, Facet)]
pub struct AccountListArgs {}
impl AccountListArgs {
    pub async fn invoke(self, _cancellation: &CancellationScope) -> Result<CliOutput> {
        Ok(CliOutput::facet(
            CloudTerrastodonAccountStore::configured().load()?,
        ))
    }
}

use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountShowArgs {
    /// Account UUID or alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
}
impl AccountShowArgs {
    pub async fn invoke(self, _cancellation: &CancellationScope) -> Result<CliOutput> {
        let catalogue = CloudTerrastodonAccountStore::configured().load()?;
        let account = catalogue.resolve(Some(&self.account), None)?;
        Ok(CliOutput::facet(account.clone()))
    }
}

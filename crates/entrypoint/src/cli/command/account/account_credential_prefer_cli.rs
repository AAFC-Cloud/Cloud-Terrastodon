use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Choose the mechanism used when a command omits its source override.
#[derive(Debug, Facet)]
pub struct AccountCredentialPreferArgs {
    /// Account UUID or alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
    #[facet(figue::named)]
    pub source: CloudTerrastodonCredentialSource,
}
impl AccountCredentialPreferArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        let account_id = store.load()?.resolve(Some(&self.account), None)?.id;
        Ok(CliOutput::facet(
            store
                .set_preferred_source(account_id, self.source, cancellation)
                .await?,
        ))
    }
}

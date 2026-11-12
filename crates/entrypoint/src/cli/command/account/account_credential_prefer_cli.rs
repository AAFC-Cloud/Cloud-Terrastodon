use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Choose the mechanism used when a command omits its source override.
#[derive(Debug, Facet)]
pub struct AccountCredentialPreferArgs {
    #[facet(figue::positional, proxy = String)]
    pub account_id: CloudTerrastodonAccountId,
    #[facet(figue::named)]
    pub source: CloudTerrastodonCredentialSource,
}
impl AccountCredentialPreferArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        Ok(CliOutput::facet(
            CloudTerrastodonAccountStore::configured()
                .set_preferred_source(self.account_id, self.source, cancellation)
                .await?,
        ))
    }
}

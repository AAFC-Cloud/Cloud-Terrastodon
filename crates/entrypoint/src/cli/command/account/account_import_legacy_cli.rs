use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
/// Explicitly import an existing CT tenant login, browser credential, or legacy PAT.
/// This does not import an ambient Azure CLI login.
#[derive(Debug, Facet)]
pub struct AccountImportLegacyArgs {
    #[facet(figue::positional, proxy = String)]
    pub account_id: CloudTerrastodonAccountId,
    /// Credential mechanism to provision; omission uses the account preference.
    #[facet(figue::named)]
    pub source: Option<CloudTerrastodonCredentialSource>,
}
impl AccountImportLegacyArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        Ok(CliOutput::facet(
            CloudTerrastodonAccountStore::configured()
                .import_legacy(self.account_id, self.source, cancellation)
                .await?,
        ))
    }
}

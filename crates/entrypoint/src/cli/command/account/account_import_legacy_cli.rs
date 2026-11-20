use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
/// Explicitly import an existing CT tenant login, browser credential, or legacy PAT.
/// This does not import an ambient Azure CLI login.
#[derive(Debug, Facet)]
pub struct AccountImportLegacyArgs {
    /// Account UUID or alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,
    /// Credential mechanism to provision; omission uses the account preference.
    #[facet(figue::named)]
    pub source: Option<CloudTerrastodonCredentialSource>,
}
impl AccountImportLegacyArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        let account_id = store.load()?.resolve(Some(&self.account), None)?.id;
        Ok(CliOutput::facet(
            store
                .import_legacy(account_id, self.source, cancellation)
                .await?,
        ))
    }
}

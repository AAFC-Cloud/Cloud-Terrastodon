use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;

/// Change a registered account's alias.
#[derive(Debug, Facet)]
pub struct AccountRenameArgs {
    /// Account UUID or unambiguous alias.
    #[facet(figue::positional)]
    pub account: CloudTerrastodonAccountArgument,

    /// New alias, unique within the account's tenant or service.
    #[facet(figue::positional, proxy = String)]
    pub new_name: CloudTerrastodonAccountAlias,
}

impl AccountRenameArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let store = CloudTerrastodonAccountStore::configured();
        let account_id = store.load()?.resolve(Some(&self.account), None)?.id;
        let profile = store
            .rename(account_id, self.new_name, cancellation)
            .await?;
        Ok(CliOutput::facet(profile))
    }
}

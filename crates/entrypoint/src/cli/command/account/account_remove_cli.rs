use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountRemoveArgs {
    #[facet(figue::positional, proxy = String)]
    pub account_id: CloudTerrastodonAccountId,
}
impl AccountRemoveArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        CloudTerrastodonAccountStore::configured()
            .remove_with_executor(self.account_id, executor)
            .await?;
        Ok(CliOutput::facet(self.account_id))
    }
}

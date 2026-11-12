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
        executor
            .cancellation()
            .deferred("retiring and removing an account", async |scope| {
                let executor = executor.with_cancellation_scope(scope);
                executor.retire_account(self.account_id, None).await?;
                CloudTerrastodonAccountStore::configured()
                    .remove(self.account_id, executor.cancellation())
                    .await
            })
            .await?;
        Ok(CliOutput::facet(self.account_id))
    }
}

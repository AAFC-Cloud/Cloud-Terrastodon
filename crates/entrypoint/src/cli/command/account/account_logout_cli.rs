use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
#[derive(Debug, Facet)]
pub struct AccountLogoutArgs {
    #[facet(figue::positional, proxy = String)]
    pub account_id: CloudTerrastodonAccountId,
    /// Retire this mechanism only; omission retires every mechanism on the account.
    #[facet(figue::named)]
    pub source: Option<CloudTerrastodonCredentialSource>,
}
impl AccountLogoutArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        executor
            .cancellation()
            .deferred("retiring and removing account credentials", async |scope| {
                let executor = executor.with_cancellation_scope(scope);
                executor
                    .retire_account(self.account_id, self.source)
                    .await?;
                CloudTerrastodonAccountStore::configured()
                    .logout_source(self.account_id, self.source, executor.cancellation())
                    .await
            })
            .await?;
        Ok(CliOutput::facet(self.account_id))
    }
}

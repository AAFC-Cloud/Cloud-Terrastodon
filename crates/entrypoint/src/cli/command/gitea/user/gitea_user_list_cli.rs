use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::GiteaUserListRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use tracing::info;

#[derive(Facet, Debug, Clone)]
pub struct GiteaUserListArgs {
    /// Tracked tenant URL or alias to query. Select an instance explicitly; account defaults do not select service scope.
    #[facet(figue::named, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
    /// Account ID or alias for this instance; omission selects its configured default.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
}

impl GiteaUserListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant = self.tenant.resolve(executor.cancellation()).await?;
        let account = self.account.as_ref().map(ToString::to_string);
        let account_id =
            executor.resolve_gitea_authentication(tenant.as_ref(), account.as_deref())?;
        info!(%tenant, "Fetching Gitea users");
        let users = GiteaUserListRequest {
            account_id,
            tenant: Cow::Borrowed(&tenant),
        }
        .execute(executor)
        .await?;
        info!(count = users.len(), "Fetched Gitea users");
        Ok(CliOutput::facet(users))
    }
}

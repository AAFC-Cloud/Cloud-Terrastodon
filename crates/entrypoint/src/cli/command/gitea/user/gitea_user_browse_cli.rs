use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_cli_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_gitea::GiteaTenantArgument;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_gitea::GiteaUserListRequest;
use cloud_terrastodon_user_input::PickResultExt;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;

#[derive(Facet, Debug, Clone)]
pub struct GiteaUserBrowseArgs {
    /// Tracked tenant URL or alias to query. Select an instance explicitly; account defaults do not select service scope.
    #[facet(figue::named, proxy = String)]
    pub tenant: GiteaTenantArgument<'static>,
    /// Account ID or alias for this instance; omission selects its configured default.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
}

impl GiteaUserBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant = self.tenant.resolve(executor.cancellation()).await?;
        let account = self.account.as_ref().map(ToString::to_string);
        let account_id =
            executor.resolve_gitea_authentication(tenant.as_ref(), account.as_deref())?;
        let (chosen, maybe_error) = PickerTui::<_>::new()
            .pick_many_reloadable(executor.cancellation(), async |child, invalidate| {
                let executor = &executor.with_cancellation_scope(child);
                GiteaUserListRequest {
                    account_id,
                    tenant: Cow::Borrowed(&tenant),
                }
                .execute(executor)
                .with_invalidation(invalidate)
                .await
            })
            .await
            .into_chosen_and_maybe_error()?;
        Ok(CliOutput::facet(chosen).with_result(maybe_error))
    }
}

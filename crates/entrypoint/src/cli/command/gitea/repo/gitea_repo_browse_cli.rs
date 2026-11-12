use crate::cli::gitea::repo::gitea_repo_list_cli::GiteaRepoListArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_user_input::PickResultExt;
use eyre::Result;
use facet::Facet;

#[derive(Facet, Debug, Clone)]
pub struct GiteaRepoBrowseArgs {
    #[facet(flatten)]
    pub list: GiteaRepoListArgs,
}

impl GiteaRepoBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let tenant = self.list.tenant.resolve(executor.cancellation()).await?;
        let account = self.list.account.as_ref().map(ToString::to_string);
        let account_id =
            executor.resolve_gitea_authentication(tenant.as_ref(), account.as_deref())?;
        let repositories = self
            .list
            .fetch_repositories(&tenant, account_id, executor)
            .await?;
        let (chosen, maybe_error) = cloud_terrastodon_user_input::PickerTui::<_>::new()
            .pick_many(repositories, executor.cancellation())
            .await
            .into_chosen_and_maybe_error()?;
        Ok(CliOutput::facet(chosen).with_result(maybe_error))
    }
}

use crate::cli::gitea::repo::gitea_repo_list_cli::GiteaRepoListArgs;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use cloud_terrastodon_user_input::PickResultExt;
use eyre::Result;
use std::io::Write;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaRepoBrowseArgs {
    #[facet(flatten)]
    pub list: GiteaRepoListArgs,
}

impl GiteaRepoBrowseArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let tenant = self.list.tenant.resolve(executor).await?;
        let repositories = self.list.fetch_repositories(&tenant, executor).await?;
        let (chosen, maybe_error) = cloud_terrastodon_user_input::PickerTui::<_>::new()
            .pick_many(repositories)
            .await
            .into_chosen_and_maybe_error()?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &chosen)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        maybe_error
    }
}

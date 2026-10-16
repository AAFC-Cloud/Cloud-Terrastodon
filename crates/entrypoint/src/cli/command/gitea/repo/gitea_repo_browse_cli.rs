use crate::cli::gitea::repo::gitea_repo_list_cli::GiteaRepoListArgs;
use cloud_terrastodon_command::CancellationScope;
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
    pub async fn invoke(
        self,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let tenant = self.list.tenant.resolve(cancellation).await?;
        let repositories = self.list.fetch_repositories(&tenant, cancellation).await?;
        let (chosen, maybe_error) = cloud_terrastodon_user_input::PickerTui::<_>::new()
            .pick_many(repositories)
            .await
            .into_chosen_and_maybe_error()?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        maybe_error
    }
}

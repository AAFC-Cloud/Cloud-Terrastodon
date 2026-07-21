use crate::cli::gitea::repo::gitea_repo_list::GiteaRepoListArgs;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use eyre::Result;
use std::io::Write;

#[derive(facet::Facet, Debug, Clone)]
pub struct GiteaRepoBrowseArgs {
    #[facet(flatten)]
    pub list: GiteaRepoListArgs,
}

impl GiteaRepoBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.list.tenant.resolve().await?;
        let repositories = self.list.fetch_repositories(&tenant).await?;
        let chosen = cloud_terrastodon_user_input::PickerTui::<_>::new()
            .pick_many(repositories)
            .await?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

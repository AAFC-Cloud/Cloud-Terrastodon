use crate::cli::gitea::repo::gitea_repo_list::GiteaRepoListArgs;
use clap::Args;
use cloud_terrastodon_gitea::GiteaTenantArgumentExt;
use eyre::Result;
use std::io::Write;

#[derive(Args, Debug, Clone)]
pub struct GiteaRepoBrowseArgs {
    #[command(flatten)]
    pub list: GiteaRepoListArgs,
}

impl GiteaRepoBrowseArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant = self.list.tenant.resolve().await?;
        let repositories = self.list.fetch_repositories(&tenant).await?;
        let chosen = cloud_terrastodon_user_input::PickerTui::new().pick_many(repositories)?;
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &chosen)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}

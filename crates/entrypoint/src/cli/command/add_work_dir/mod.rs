use clap::Args;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_config::WorkDirsConfig;
use eyre::Context;
use eyre::Result;
use std::fs::canonicalize;
use std::path::PathBuf;

/// Add a directory to the list of configured work directories.
#[derive(Args, Debug, Clone)]
pub struct AddWorkDirArgs {
    /// The directory to register as a work directory.
    pub dir: PathBuf,
}

impl AddWorkDirArgs {
    pub async fn invoke(self) -> Result<()> {
        let mut dir = self.dir;
        if !dir.is_absolute() {
            dir = canonicalize(&dir)
                .context(format!("failed to make path absolute: {}", dir.display()))?;
        }

        let mut config = WorkDirsConfig::load().await?;
        config.work_dirs.insert(dir);
        config.save().await?;

        Ok(())
    }
}

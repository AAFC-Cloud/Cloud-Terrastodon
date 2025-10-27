use clap::Args;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use std::path::Path;
use std::path::PathBuf;

/// Copy run artifacts to a destination directory.
#[derive(Args, Debug, Clone)]
pub struct CopyResultsArgs {
    /// Destination directory for copying results.
    pub dest: PathBuf,
}

impl CopyResultsArgs {
    pub async fn invoke(self) -> Result<()> {
        // from https://stackoverflow.com/a/78769977/11141271
        #[async_recursion::async_recursion]
        async fn copy_dir_all<S, D>(src: S, dst: D) -> Result<(), std::io::Error>
        where
            S: AsRef<Path> + Send + Sync,
            D: AsRef<Path> + Send + Sync,
        {
            tokio::fs::create_dir_all(&dst).await?;
            let mut entries = tokio::fs::read_dir(src).await?;
            while let Some(entry) = entries.next_entry().await? {
                let ty = entry.file_type().await?;
                if ty.is_dir() {
                    copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name())).await?;
                } else {
                    tokio::fs::copy(entry.path(), dst.as_ref().join(entry.file_name())).await?;
                }
            }
            Ok(())
        }

        copy_dir_all(AppDir::Processed.as_path_buf(), self.dest).await?;
        Ok(())
    }
}

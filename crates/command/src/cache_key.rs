use crate::NoSpaces;
use chrono::Local;
use cloud_terrastodon_pathing::AppDir;
use eyre::Context;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::debug;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey {
    pub path: PathBuf,
    pub valid_for: Duration,
}

impl CacheKey {
    /// The path to a directory on disk where the cache is stored.
    pub fn path_on_disk(&self) -> PathBuf {
        AppDir::Commands.join(self.path.no_spaces())
    }

    /// Invalidate the cache by creating a sentinel file named "busted" to indicate that the cache entries should not be used.
    pub async fn invalidate(&self) -> Result<()> {
        let cache_dir = self.path_on_disk();
        if cache_dir.exists() {
            debug!(path=%cache_dir.display(),"Busting cache");

            // For each file named "context.txt" that is a descendant of the path,
            // we create a file named "busted" in the same directory.

            let mut dirs = vec![cache_dir];
            let now = Local::now();
            while let Some(dir) = dirs.pop() {
                let mut read_dir = tokio::fs::read_dir(&dir).await.wrap_err_with(|| {
                    format!("failed reading cache directory at {}", dir.display())
                })?;
                while let Some(entry) = read_dir.next_entry().await.wrap_err_with(|| {
                    format!("failed reading cache directory at {}", dir.display())
                })? {
                    let path = entry.path();
                    if path.is_dir() {
                        dirs.push(path);
                    } else if let Some(file_name) = path.file_name()
                        && file_name == "context.txt"
                    {
                        let busted_path = path.with_file_name("busted");
                        let mut file = OpenOptions::new()
                            .create(true)
                            .truncate(false)
                            .write(true)
                            .open(&busted_path)
                            .await
                            .wrap_err_with(|| {
                                format!(
                                    "failed creating busted cache indicator at {}",
                                    busted_path.display(),
                                )
                            })?;
                        file.write_all(now.to_rfc2822().as_bytes()).await?;
                    }
                }
            }
        }
        Ok(())
    }
}

pub trait HasCacheKey {
    fn cache_key(&self) -> CacheKey;
}

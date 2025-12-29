use crate::NoSpaces;
use chrono::Local;
use cloud_terrastodon_pathing::AppDir;
use eyre::Context;
use eyre::Result;
use std::borrow::Cow;
use std::future::Future;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::debug;

pub trait HasCacheKey {
    fn cache_key<'a>(&'a self) -> Cow<'a, PathBuf>;
    fn cache_path(&self) -> PathBuf {
        let key = self.cache_key();
        AppDir::Commands.join(key.no_spaces())
    }
}

pub trait InvalidatableCache {
    fn invalidate_cache(&self) -> impl Future<Output = Result<()>> + Send;
}
impl<T> InvalidatableCache for T where T: HasCacheKey {
    fn invalidate_cache(&self) -> impl Future<Output = Result<()>> + Send {
        let cache_dir = self.cache_path();
        async move {
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
                        } else if let Some(file_name) = path.file_name() {
                            if file_name == "context.txt" {
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
            }
            Ok(())
        }
    }
}
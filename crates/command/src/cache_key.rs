use crate::NoSpaces;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use std::borrow::Cow;
use std::future::Future;
use std::path::PathBuf;
use tracing::debug;

pub trait HasCacheKey {
    fn cache_key<'a>(&'a self) -> Cow<'a, PathBuf>;
    fn cache_path(&self) -> PathBuf {
        let key = self.cache_key();
        AppDir::Commands.join(key.no_spaces())
    }
    fn bust_cache(&self) -> impl Future<Output = Result<()>> + Send {
        let path = self.cache_path();
        async move {
            if path.exists() {
                debug!(path=%path.display(),"Busting cache");
                tokio::fs::remove_dir_all(path).await?;
            }
            Ok(())
        }
    }
}

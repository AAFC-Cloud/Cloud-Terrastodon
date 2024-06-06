use anyhow::bail;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::try_exists;

use tokio::fs::create_dir_all;
use tracing::info;

const IGNORE_ROOT: &str = "ignore";

pub enum IgnoreDir {
    Root,
    Commands,
    Imports,
    Processed,
}
impl IgnoreDir {
    pub fn as_path_buf(&self) -> PathBuf {
        match self {
            IgnoreDir::Root => PathBuf::from(IGNORE_ROOT),
            IgnoreDir::Commands => PathBuf::from_iter([IGNORE_ROOT, "commands"]),
            IgnoreDir::Imports => PathBuf::from_iter([IGNORE_ROOT, "imports"]),
            IgnoreDir::Processed => PathBuf::from_iter([IGNORE_ROOT, "processed"]),
        }
    }
    pub fn join(self, path: impl AsRef<Path>) -> PathBuf {
        let buf: PathBuf = self.into();
        buf.join(path)
    }
}

#[allow(async_fn_in_trait)]
pub trait Existy {
    async fn ensure_dir_exists(&self) -> anyhow::Result<()>;
}
impl<T: AsRef<Path>> Existy for T {
    async fn ensure_dir_exists(&self) -> anyhow::Result<()> {
        let path = self.as_ref();
        match try_exists(&path).await {
            Ok(true) => {
                if !path.is_dir() {
                    bail!("Path {} exists but isn't a dir!", path.display());
                }
                Ok(())
            }
            Ok(false) => {
                info!("Creating {}", path.display());
                create_dir_all(&path).await?;
                Ok(())
            }
            Err(e) => {
                bail!(
                    "Error encountered checking if {} exists: {:?}",
                    path.display(),
                    e
                )
            }
        }
    }
}

impl From<IgnoreDir> for PathBuf {
    fn from(dir: IgnoreDir) -> Self {
        dir.as_path_buf()
    }
}

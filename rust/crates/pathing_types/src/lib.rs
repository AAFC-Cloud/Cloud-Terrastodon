use anyhow::bail;
use std::path::Path;
use std::path::PathBuf;

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
    pub async fn ensure_exists(&self) -> anyhow::Result<()> {
        let path: PathBuf = self.as_path_buf();
        if !path.exists() {
            info!("Creating {:?}", path);
            create_dir_all(&path).await?;
        } else if !path.is_dir() {
            bail!("Path {} exists but isn't a dir!", path.to_string_lossy());
        }
        Ok(())
    }
}

impl From<IgnoreDir> for PathBuf {
    fn from(dir: IgnoreDir) -> Self {
        dir.into()
    }
}

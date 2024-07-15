use anyhow::bail;
use clap::ValueEnum;
use directories_next::ProjectDirs;
use once_cell::sync::Lazy;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::create_dir_all;
use tokio::fs::try_exists;
use tracing::debug;

static PROJECT_DIRS: Lazy<ProjectDirs> = Lazy::new(|| {
    let Some(project_dirs) = ProjectDirs::from_path(PathBuf::from("cloud_terrastodon")) else {
        panic!("Failed to acquire disk locations for project");
    };
    project_dirs
});
static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| {
    PROJECT_DIRS.cache_dir().to_path_buf()
});
static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| {
    PROJECT_DIRS.data_dir().to_path_buf()
});

#[derive(Debug, Clone, ValueEnum)]
pub enum IgnoreDir {
    Commands,
    Imports,
    Processed,
    Temp,
}
impl std::fmt::Display for IgnoreDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            IgnoreDir::Commands => "Commands",
            IgnoreDir::Imports => "Imports",
            IgnoreDir::Processed => "Processed",
            IgnoreDir::Temp => "Temp",
        })
    }
}
impl IgnoreDir {
    pub fn as_path_buf(&self) -> PathBuf {
        match self {
            IgnoreDir::Commands => CACHE_DIR.join("commands"),
            IgnoreDir::Imports => DATA_DIR.join("imports"),
            IgnoreDir::Processed => DATA_DIR.join("processed"),
            IgnoreDir::Temp => DATA_DIR.join("temp"),
        }
    }
    pub fn join(self, path: impl AsRef<Path>) -> PathBuf {
        let buf: PathBuf = self.into();
        buf.join(path)
    }
    pub fn variants() -> Vec<IgnoreDir> {
        vec![
            IgnoreDir::Commands,
            IgnoreDir::Imports,
            IgnoreDir::Processed,
            IgnoreDir::Temp,
        ]
    }
}

#[allow(async_fn_in_trait)]
pub trait Existy {
    async fn ensure_dir_exists(&self) -> anyhow::Result<()>;
    async fn ensure_parent_dir_exists(&self) -> anyhow::Result<()>;
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
                debug!("Creating {}", path.display());
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
    async fn ensure_parent_dir_exists(&self) -> anyhow::Result<()> {
        if let Some(parent) = self.as_ref().parent() {
            parent.ensure_dir_exists().await?;
            Ok(())
        } else {
            bail!("Could not acquire parent for {}", self.as_ref().display());
        }
    }
}

impl From<IgnoreDir> for PathBuf {
    fn from(dir: IgnoreDir) -> Self {
        dir.as_path_buf()
    }
}

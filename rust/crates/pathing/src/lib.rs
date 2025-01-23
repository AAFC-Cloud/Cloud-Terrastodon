use clap::ValueEnum;
use directories_next::ProjectDirs;
use eyre::bail;
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
static CACHE_DIR: Lazy<PathBuf> = Lazy::new(|| PROJECT_DIRS.cache_dir().to_path_buf());
static DATA_DIR: Lazy<PathBuf> = Lazy::new(|| PROJECT_DIRS.data_dir().to_path_buf());
static CONFIG_DIR: Lazy<PathBuf> = Lazy::new(|| PROJECT_DIRS.config_dir().to_path_buf());

#[derive(Debug, Clone, ValueEnum)]
pub enum AppDir {
    Commands,
    Imports,
    Processed,
    Temp,
    Config,
}
impl std::fmt::Display for AppDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AppDir::Commands => "Commands",
            AppDir::Imports => "Imports",
            AppDir::Processed => "Processed",
            AppDir::Temp => "Temp",
            AppDir::Config => "Config",
        })
    }
}
impl AppDir {
    pub fn as_path_buf(&self) -> PathBuf {
        match self {
            AppDir::Commands => CACHE_DIR.join("commands"),
            AppDir::Imports => DATA_DIR.join("imports"),
            AppDir::Processed => DATA_DIR.join("processed"),
            AppDir::Temp => DATA_DIR.join("temp"),
            AppDir::Config => CONFIG_DIR.clone(),
        }
    }
    pub fn join(self, path: impl AsRef<Path>) -> PathBuf {
        let buf: PathBuf = self.into();
        buf.join(path)
    }
    pub fn ok_to_clean() -> Vec<AppDir> {
        vec![
            AppDir::Commands,
            AppDir::Imports,
            AppDir::Processed,
            AppDir::Temp,
        ]
    }
    pub fn variants() -> Vec<AppDir> {
        vec![
            AppDir::Commands,
            AppDir::Imports,
            AppDir::Processed,
            AppDir::Temp,
            AppDir::Config,
        ]
    }
}

#[allow(async_fn_in_trait)]
pub trait Existy {
    async fn ensure_dir_exists(&self) -> eyre::Result<()>;
    async fn ensure_parent_dir_exists(&self) -> eyre::Result<()>;
}
impl<T: AsRef<Path>> Existy for T {
    async fn ensure_dir_exists(&self) -> eyre::Result<()> {
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
    async fn ensure_parent_dir_exists(&self) -> eyre::Result<()> {
        if let Some(parent) = self.as_ref().parent() {
            parent.ensure_dir_exists().await?;
            Ok(())
        } else {
            bail!("Could not acquire parent for {}", self.as_ref().display());
        }
    }
}

impl From<AppDir> for PathBuf {
    fn from(dir: AppDir) -> Self {
        dir.as_path_buf()
    }
}

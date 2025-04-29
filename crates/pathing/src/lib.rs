use clap::ValueEnum;
use directories_next::ProjectDirs;
use eyre::Context;
use eyre::bail;
use std::path::Path;
use std::path::PathBuf;
use std::sync::LazyLock;
use tokio::fs::create_dir_all;
use tokio::fs::try_exists;
use tracing::debug;
use tracing::field::debug;

static PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let Some(project_dirs) = ProjectDirs::from_path(PathBuf::from("cloud_terrastodon")) else {
        panic!("Failed to acquire disk locations for project");
    };
    project_dirs
});
static CACHE_DIR: LazyLock<PathBuf> = LazyLock::new(|| PROJECT_DIRS.cache_dir().to_path_buf());
static DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| PROJECT_DIRS.data_dir().to_path_buf());
static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| PROJECT_DIRS.config_dir().to_path_buf());

#[derive(Debug, Clone, ValueEnum)]
pub enum AppDir {
    Commands,
    Imports,
    Processed,
    Temp,
    Config,
    WorkItems,
}
impl std::fmt::Display for AppDir {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            AppDir::Commands => "Commands",
            AppDir::Imports => "Imports",
            AppDir::Processed => "Processed",
            AppDir::Temp => "Temp",
            AppDir::Config => "Config",
            AppDir::WorkItems => "Work Items",
        })
    }
}
impl AppDir {
    pub fn as_path_buf(&self) -> PathBuf {
        match self {
            AppDir::Commands => CACHE_DIR.join("commands"),
            AppDir::WorkItems => CACHE_DIR.join("work_items"),
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
        Self::variants()
            .iter()
            .filter(|x| !matches!(x, Self::Config))
            .cloned()
            .collect()
    }
    pub fn variants() -> &'static [AppDir] {
        AppDir::value_variants()
    }
}

#[async_trait::async_trait]
pub trait Existy {
    async fn ensure_dir_exists(&self) -> eyre::Result<()>;
    async fn ensure_parent_dir_exists(&self) -> eyre::Result<()>;
    async fn exists_async(&self) -> eyre::Result<bool>;
}
#[async_trait::async_trait]
impl<T: AsRef<Path>> Existy for T
where
    T: Sync,
{
    async fn ensure_dir_exists(&self) -> eyre::Result<()> {
        let path = self.as_ref();
        debug("Ensuring path exist: {path:?}");
        match try_exists(&path).await {
            Ok(true) => {
                if !path.is_dir() {
                    bail!("Path {} exists but isn't a dir!", path.display());
                }
                Ok(())
            }
            Ok(false) => {
                debug!("Creating {}", path.display());
                create_dir_all(&path)
                    .await
                    .wrap_err(format!("Ensuring dir exists: {}", path.display()))?;
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
    async fn exists_async(&self) -> eyre::Result<bool> {
        tokio::fs::try_exists(&self).await.wrap_err(format!(
            "Checking if path exists: {:?}",
            self.as_ref() as &Path
        ))
    }
}
#[async_trait::async_trait]
impl Existy for AppDir {
    async fn ensure_dir_exists(&self) -> eyre::Result<()> {
        self.as_path_buf().ensure_dir_exists().await
    }
    async fn ensure_parent_dir_exists(&self) -> eyre::Result<()> {
        self.as_path_buf().ensure_parent_dir_exists().await
    }
    async fn exists_async(&self) -> eyre::Result<bool> {
        self.as_path_buf().exists_async().await
    }
}

impl From<AppDir> for PathBuf {
    fn from(dir: AppDir) -> Self {
        dir.as_path_buf()
    }
}

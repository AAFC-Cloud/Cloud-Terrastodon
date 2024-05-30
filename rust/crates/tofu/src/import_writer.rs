use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use pathing_types::IgnoreDir;
use std::path::Path;
use std::path::PathBuf;
use tofu_types::prelude::AsTofuString;
use tokio::fs::create_dir_all;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub struct TofuImportWriter {
    path: PathBuf,
}
impl TofuImportWriter {
    pub fn new(path: impl AsRef<Path>) -> TofuImportWriter {
        TofuImportWriter {
            path: path.as_ref().to_path_buf(),
        }
    }
    pub async fn overwrite(&self, content: impl AsTofuString) -> Result<()> {
        let imports_dir: PathBuf = IgnoreDir::Imports.into();
        if !imports_dir.exists() {
            info!("Creating {:?}", imports_dir);
            create_dir_all(&imports_dir).await?;
        } else if !imports_dir.is_dir() {
            return Err(anyhow!("Path exists but isn't a dir!"))
                .context(imports_dir.to_string_lossy().into_owned());
        }

        let imports_path = imports_dir.join(&self.path);
        let mut imports_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&imports_path)
            .await?;
        info!("Writing {:?}", imports_path);
        imports_file
            .write_all(content.as_tofu_string().as_bytes())
            .await?;
        Ok(())
    }
}

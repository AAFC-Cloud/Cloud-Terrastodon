use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use tokio::io::AsyncWriteExt;
use tokio::io::stdout;

/// Retrieve the path to a well-known application directory.
#[derive(facet::Facet, Debug, Clone)]
pub struct GetPathArgs {
    /// The application directory to resolve.
    #[facet(figue::positional)]
    pub dir: AppDir,
}

impl GetPathArgs {
    pub async fn invoke(self) -> Result<()> {
        let mut out = stdout();
        out.write_all(self.dir.as_path_buf().as_os_str().as_encoded_bytes())
            .await?;
        out.flush().await?;

        Ok(())
    }
}

use anyhow::Context;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use pathing_types::Existy;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tofu_types::prelude::AsTofuString;
use tofu_types::prelude::TofuProviderBlock;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub struct TofuWriter {
    path: PathBuf,
}
impl TofuWriter {
    pub fn new(path: impl AsRef<Path>) -> TofuWriter {
        TofuWriter {
            path: path.as_ref().to_path_buf(),
        }
    }

    pub async fn format(&self) -> Result<()> {
        CommandBuilder::new(CommandKind::Tofu)
            .arg("fmt")
            .arg(self.path.as_os_str())
            .should_announce(true)
            .run_raw()
            .await?;
        Ok(())
    }

    pub async fn overwrite(&self, content: impl AsTofuString) -> Result<&Self> {
        self.path.ensure_parent_dir_exists().await?;
        let mut imports_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .await
            .context(format!("opening file {}", self.path.display()))?;
        info!("Writing {:?}", self.path);
        imports_file
            .write_all(content.as_tofu_string().as_bytes())
            .await
            .context("writing content")?;
        Ok(self)
    }
    pub async fn merge(&self, providers: Vec<TofuProviderBlock>) -> Result<&Self> {
        self.path.ensure_parent_dir_exists().await?;
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .append(true)
            .open(&self.path)
            .await
            .context(format!("opening file {}", self.path.display()))?;

        // Read existing content
        let mut existing_content = String::new();
        file.read_to_string(&mut existing_content)
            .await
            .context("reading content")?;

        // Determine existing provider blocks
        let existing_body = existing_content.parse::<Body>()?;
        let existing_providers: HashSet<TofuProviderBlock> = existing_body
            .into_blocks()
            .filter_map(|block| TofuProviderBlock::try_from(block).ok())
            .collect();

        // Add provider blocks not already present
        let mut append_body = Body::builder();
        for provider in providers {
            if existing_providers.contains(&provider) {
                continue;
            }
            let block: Block = provider.try_into()?;
            append_body = append_body.block(block);
        }
        let append_body = append_body.build();

        // Write content
        file.write_all(append_body.as_tofu_string().as_bytes())
            .await
            .context("appending content")?;
        Ok(self)
    }
}

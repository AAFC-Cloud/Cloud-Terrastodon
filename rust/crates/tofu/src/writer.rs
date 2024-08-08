use anyhow::Context;
use anyhow::Result;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use pathing::Existy;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tofu_types::prelude::AsTofuString;
use tofu_types::prelude::TofuImportBlock;
use tofu_types::prelude::TofuProviderBlock;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::prelude::TofuBlock;

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
    pub async fn merge(
        &self,
        to_merge: impl IntoIterator<Item = impl Into<TofuBlock>>,
    ) -> Result<&Self> {
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
        let existing_body = existing_content.parse::<Body>()?;

        // Create holders for deduplicating data
        let mut provider_blocks: HashSet<TofuProviderBlock> = Default::default();
        let mut import_blocks: HashSet<TofuImportBlock> = Default::default();
        let mut other_blocks: Vec<Block> = Default::default();

        // Track existing blocks
        for block in existing_body.into_blocks() {
            match TofuBlock::try_from(block)? {
                TofuBlock::Provider(block) => {
                    provider_blocks.insert(block);
                }
                TofuBlock::Import(block) => {
                    import_blocks.insert(block);
                }
                TofuBlock::Other(block) => {
                    other_blocks.push(block);
                }
            }
        }

        // Add blocks we want to merge
        for block in to_merge {
            match block.into() {
                TofuBlock::Provider(block) => {
                    provider_blocks.insert(block);
                }
                TofuBlock::Import(block) => {
                    import_blocks.insert(block);
                }
                TofuBlock::Other(block) => {
                    other_blocks.push(block);
                }
            }
        }

        // Build result body
        let mut result_body = Body::builder();
        for block in provider_blocks {
            result_body = result_body.block(block);
        }
        for block in import_blocks {
            let block: Block = block.try_into()?;
            result_body = result_body.block(block);
        }
        for block in other_blocks {
            result_body = result_body.block(block);
        }
        let result_body = result_body.build();

        // Write content
        file.write_all(result_body.as_tofu_string().as_bytes())
            .await
            .context("appending content")?;
        Ok(self)
    }
}

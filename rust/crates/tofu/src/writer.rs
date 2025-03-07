use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use cloud_terrastodon_core_pathing::Existy;
use cloud_terrastodon_core_tofu_types::prelude::AsTofuString;
use cloud_terrastodon_core_tofu_types::prelude::TofuImportBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuProviderBlock;
use cloud_terrastodon_core_tofu_types::prelude::TofuTerraformBlock;
use eyre::Context;
use eyre::Result;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use itertools::Itertools;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tracing::info;
use tracing::warn;

use crate::prelude::TofuBlock;
use crate::sorting::TofuBlockSortable;

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
        info!("Overwriting tf file {}", self.path.display());
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
            .truncate(false)
            .read(true)
            .write(true)
            .open(&self.path)
            .await
            .context(format!("opening file {}", self.path.display()))?;

        // Read existing content
        let mut existing_content = String::new();
        file.read_to_string(&mut existing_content)
            .await
            .context("reading content")?;
        let existing_body = existing_content.parse::<Body>().context(format!(
            "Failed to parse HCL from body: \n```\n{existing_content:?}\n```"
        ))?;

        // Create holders for deduplicating data
        let mut terraform_blocks: Vec<TofuTerraformBlock> = Default::default();
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
                TofuBlock::Terraform(block) => {
                    terraform_blocks.push(block);
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
                TofuBlock::Terraform(block) => {
                    terraform_blocks.push(block);
                }
            }
        }

        // Build result body
        let mut result_body = Body::builder();
        let mut terraform_block = TofuTerraformBlock::default();
        for block in terraform_blocks {
            if block.backend.is_some() {
                if terraform_block.backend.is_some() {
                    warn!("Multiple backend blocks detected, prioritizing latest")
                }
                terraform_block.backend = block.backend;
            }
            if let Some(required_providers) = block.required_providers {
                match &mut terraform_block.required_providers {
                    None => {
                        terraform_block.required_providers = Some(required_providers);
                    }
                    Some(existing) => {
                        // Sort by provider name
                        let providers = required_providers
                            .0
                            .into_iter()
                            .sorted_by(|(a, _), (b, _)| a.cmp(b));

                        for (provider, version) in providers {
                            if let Some(existing_provider_version) = existing.0.get(&provider) {
                                if *existing_provider_version != version {
                                    warn!(
                                        "Detected multiple required_provider entries for {provider}, discarding {:?} for {:?}",
                                        existing_provider_version, version
                                    );
                                }
                            }
                            existing.0.insert(provider, version);
                        }
                    }
                }
            }
            terraform_block.other.extend(block.other);
        }
        if !terraform_block.is_empty() {
            result_body = result_body.block(terraform_block);
        }

        let sorted_provider_blocks = provider_blocks.into_iter().sorted_by(|a, b| {
            a.provider_kind()
                .provider_prefix()
                .cmp(b.provider_kind().provider_prefix())
        });
        for block in sorted_provider_blocks {
            result_body = result_body.block(block);
        }

        let sorted_import_blocks = import_blocks
            .into_iter()
            .sorted_by(|a, b| a.to.expression_str().cmp(&b.to.expression_str()));
        for block in sorted_import_blocks {
            let block: Block = block.try_into()?;
            result_body = result_body.block(block);
        }

        let sorted_other_blocks = other_blocks.into_iter().sort_blocks();
        for block in sorted_other_blocks {
            result_body = result_body.block(block);
        }
        let result_body = result_body.build();

        // Truncate and write merged content
        file.set_len(0).await?;
        file.seek(std::io::SeekFrom::Start(0)).await?;
        file.write_all(result_body.as_tofu_string().as_bytes())
            .await
            .context("appending content")?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        // Create provider blocks
        let mut providers = HashSet::new();
        providers.insert(TofuProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        providers.insert(TofuProviderBlock::AzureRM {
            alias: Some("bruh".to_owned()),
            subscription_id: None,
        });

        // Write some content
        let path = tempfile::Builder::new().tempfile()?.into_temp_path();
        let writer = TofuWriter::new(path);

        // ensure deduplication
        writer.merge(providers.clone()).await?;
        writer.merge(providers.clone()).await?;
        // ensure old entries are kept
        writer
            .merge(providers.iter().take(1).cloned().collect_vec())
            .await?;
        writer
            .merge(providers.iter().skip(1).take(1).cloned().collect_vec())
            .await?;

        // Read back the content
        let mut file = OpenOptions::new().read(true).open(&writer.path).await?;
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        let body: Body = content.parse()?;

        // Assert that the merging successfully deduplicated
        let num_blocks = body.into_blocks().count();
        assert_eq!(num_blocks, providers.len());

        Ok(())
    }
}

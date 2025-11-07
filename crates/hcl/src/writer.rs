use crate::prelude::HclBlock;
use crate::sorting::HclBlockSortable;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_hcl_types::prelude::AsHclString;
use cloud_terrastodon_hcl_types::prelude::HclImportBlock;
use cloud_terrastodon_hcl_types::prelude::HclProviderBlock;
use cloud_terrastodon_hcl_types::prelude::TerraformBlock;
use cloud_terrastodon_pathing::Existy;
use eyre::Context;
use eyre::Result;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use itertools::Itertools;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::fs::OpenOptions;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tracing::debug;
use tracing::warn;

pub struct HclWriter {
    path: PathBuf,
    format_on_write: bool,
}
impl HclWriter {
    pub fn new(path: impl AsRef<Path>) -> HclWriter {
        HclWriter {
            path: path.as_ref().to_path_buf(),
            format_on_write: false,
        }
    }

    pub fn format_on_write(mut self) -> Self {
        // TODO: make this true by default with the method to turn off if desired
        self.format_on_write = true;
        self
    }

    pub async fn format_file(&self) -> Result<()> {
        debug!(path = %self.path.display(), "Formatting tf file");
        CommandBuilder::new(CommandKind::Terraform)
            .arg("fmt")
            .arg(self.path.as_os_str())
            .run_raw()
            .await?;
        Ok(())
    }

    async fn write(&self, file: &mut File, content: impl AsHclString + Sync) -> eyre::Result<()> {
        let content = if self.format_on_write {
            content.as_formatted_hcl_string().await?
        } else {
            content.as_hcl_string()
        };
        file.write_all(content.as_bytes())
            .await
            .context("writing content")?;
        Ok(())
    }

    pub async fn overwrite(&self, content: impl AsHclString + Sync) -> Result<&Self> {
        debug!("Overwriting tf file {}", self.path.display());
        self.path.ensure_parent_dir_exists().await?;
        let mut tf_file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&self.path)
            .await
            .context(format!("opening file {}", self.path.display()))?;
        debug!("Writing {:?}", self.path);
        self.write(&mut tf_file, content).await?;
        Ok(self)
    }
    pub async fn merge(
        &self,
        to_merge: impl IntoIterator<Item = impl Into<HclBlock>>,
    ) -> Result<&Self> {
        debug!(path = %self.path.display(), "Merging into tf file");
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
        let mut terraform_blocks: Vec<TerraformBlock> = Default::default();
        let mut provider_blocks: HashSet<HclProviderBlock> = Default::default();
        let mut import_blocks: HashSet<HclImportBlock> = Default::default();
        let mut other_blocks: Vec<Block> = Default::default();

        // Track existing blocks
        for block in existing_body.into_blocks() {
            match HclBlock::try_from(block)? {
                HclBlock::Provider(block) => {
                    provider_blocks.insert(block);
                }
                HclBlock::Import(block) => {
                    import_blocks.insert(block);
                }
                HclBlock::Other(block) => {
                    other_blocks.push(block);
                }
                HclBlock::Terraform(block) => {
                    terraform_blocks.push(block);
                }
                HclBlock::Resource(hcl_resource_block) => {
                    other_blocks.push(hcl_resource_block.into())
                }
                HclBlock::Data(hcl_data_block) => other_blocks.push(hcl_data_block.into()),
            }
        }

        // Add blocks we want to merge
        for block in to_merge {
            match block.into() {
                HclBlock::Provider(block) => {
                    provider_blocks.insert(block);
                }
                HclBlock::Import(block) => {
                    import_blocks.insert(block);
                }
                HclBlock::Other(block) => {
                    other_blocks.push(block);
                }
                HclBlock::Terraform(block) => {
                    terraform_blocks.push(block);
                }
                HclBlock::Resource(hcl_resource_block) => {
                    other_blocks.push(hcl_resource_block.into());
                }
                HclBlock::Data(hcl_data_block) => {
                    other_blocks.push(hcl_data_block.into());
                }
            }
        }

        // Build result body
        let mut result_body = Body::builder();
        let mut terraform_block = TerraformBlock::default();
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
                            if let Some(existing_provider_version) = existing.0.get(&provider)
                                && *existing_provider_version != version
                            {
                                warn!(
                                    "Detected multiple required_provider entries for {provider}, discarding {:?} for {:?}",
                                    existing_provider_version, version
                                );
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
        file.write_all(result_body.as_formatted_hcl_string().await?.as_bytes())
            .await
            .context("appending content")?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use itertools::Itertools;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        // Create provider blocks
        let mut providers = HashSet::new();
        providers.insert(HclProviderBlock::AzureRM {
            alias: None,
            subscription_id: None,
        });
        providers.insert(HclProviderBlock::AzureRM {
            alias: Some("bruh".to_owned()),
            subscription_id: None,
        });

        // Write some content
        let path = tempfile::Builder::new().tempfile()?.into_temp_path();
        let writer = HclWriter::new(path);

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

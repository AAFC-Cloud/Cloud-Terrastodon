use clap::Args;
use cloud_terrastodon_azure::prelude::Scope;
use cloud_terrastodon_azure::prelude::fetch_all_resources;
use cloud_terrastodon_hcl::prelude::AsHclString;
use cloud_terrastodon_hcl::prelude::HclBlock;
use cloud_terrastodon_hcl::prelude::HclImportBlock;
use cloud_terrastodon_hcl::prelude::HclProviderReference;
use cloud_terrastodon_hcl::prelude::HclResourceBlock;
use cloud_terrastodon_hcl::prelude::ResourceBlockReference;
use cloud_terrastodon_hcl::prelude::list_blocks_for_dir;
use eyre::Result;
use itertools::Itertools;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;
use tracing::warn;

/// Add Terraform import blocks for existing resource blocks.
#[derive(Args, Debug, Clone)]
pub struct TerraformSourceAddImportsArgs {
    #[arg(long, default_value = ".")]
    pub work_dir: PathBuf,
}

impl TerraformSourceAddImportsArgs {
    pub async fn invoke(self) -> Result<()> {
        info!("Fetching resources from Azure...");
        let resources = fetch_all_resources()
            .await?
            .into_iter()
            .into_group_map_by(|res| res.name.clone());

        info!("Analyzing Terraform source in {:?}", self.work_dir);
        let code = list_blocks_for_dir(&self.work_dir).await?;
        info!(count=?code.len(), "Discovered HCL blocks");

        info!("Reshaping data...");
        let resource_blocks: HashMap<ResourceBlockReference, &HclResourceBlock> = {
            let mut rtn = HashMap::default();
            for v in &code {
                if let HclBlock::Resource(ref resource_block) = v.hcl_block {
                    let key = resource_block.as_resource_block_reference();
                    if let Some(x) = rtn.insert(key, resource_block) {
                        warn!(
                            found=?x.as_resource_block_reference(),
                            "Duplicate resource blocks found, using latest"
                        )
                    }
                }
            }
            rtn
        };
        let import_blocks: HashMap<&ResourceBlockReference, &HclImportBlock> = {
            let mut rtn = HashMap::default();
            for v in &code {
                if let HclBlock::Import(ref import_block) = v.hcl_block {
                    let key = &import_block.to;
                    if let Some(x) = rtn.insert(key, import_block) {
                        warn!(
                            found=?x,
                            "Duplicate import blocks found, using latest"
                        )
                    }
                }
            }
            rtn
        };

        info!("Identifying missing import blocks...");
        for (resource_ref, resource_block) in resource_blocks {
            if import_blocks.contains_key(&resource_ref) {
                continue;
            }
            let Some(name) = resource_block.body().get_attribute("name") else {
                warn!(
                    ?resource_ref,
                    "Resource block missing 'name' attribute, cannot create import block"
                );
                continue;
            };
            let Some(name) = name.value.as_str() else {
                warn!(
                    ?resource_ref,
                    "Resource block 'name' attribute is not a string, cannot create import block"
                );
                continue;
            };
            let Some(candidate_resources) = resources.get(name) else {
                warn!(
                    ?resource_ref,
                    name=%name,
                    "No matching Azure resource found for Terraform resource block, cannot create import block"
                );
                continue;
            };

            for resource in candidate_resources {
                let import_block = HclImportBlock {
                    provider: HclProviderReference::Inherited, // todo: check subscription?
                    to: resource_ref.clone(),
                    id: resource.id.expanded_form(),
                };
                println!("{}", import_block.as_hcl_string());
            }
        }
        Ok(())
    }
}

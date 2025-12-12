use crate::reflow::HclReflower;
use hcl::edit::structure::Body;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ReflowTerraformBlocks;

#[async_trait::async_trait]
impl HclReflower for ReflowTerraformBlocks {
    async fn reflow(
        &mut self,
        hcl: HashMap<PathBuf, Body>,
    ) -> eyre::Result<HashMap<PathBuf, Body>> {
        let mut reflowed = HashMap::new();
        let mut terraform_blocks = Vec::new();
        for (path, mut body) in hcl {
            terraform_blocks.extend(body.remove_blocks("terraform"));
            if !body.is_empty() {
                reflowed.insert(path, body);
            }
        }
        let terraform_body = reflowed.entry(PathBuf::from("terraform.tf")).or_default();
        for block in terraform_blocks {
            terraform_body.push(block);
        }
        Ok(reflowed)
    }
}

use cloud_terrastodon_hcl_types::prelude::TerraformBlock;
use hcl::edit::visit_mut::VisitMut;
use tracing::error;
use tracing::warn;

#[derive(Default)]
pub struct TerraformBlockExtracterPatcher {
    pub terraform_block: TerraformBlock,
}
impl VisitMut for TerraformBlockExtracterPatcher {
    fn visit_body_mut(&mut self, body: &mut hcl::edit::structure::Body) {
        let terraform_blocks = body.remove_blocks("terraform");
        for block in terraform_blocks {
            if let Err(e) = TerraformBlock::assert_is_terraform_block(&block) {
                warn!(
                    "Found a terraform block that does not conform to the expected format: {:?}",
                    e
                );
                body.push(block);
                continue;
            }
            let other: TerraformBlock = match block.try_into() {
                Ok(x) => x,
                Err(e) => {
                    warn!("Failed to convert terraform block to desired format: {e:#?}");
                    continue;
                }
            };
            if let Err(e) = self.terraform_block.try_merge(other) {
                error!("Failed to merge terraform blocks: {:#?}", e);
                continue;
            }
        }
    }
}

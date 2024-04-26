use std::collections::HashMap;

use hcl::edit::structure::Block;
use hcl::edit::visit::visit_block;
use hcl::edit::visit::Visit;

pub type ResourceReference = String;
pub type ResourceId = String;
#[derive(Default)]
pub struct LookupHolder {
    pub resource_references_by_id: HashMap<ResourceId, ResourceReference>,
}
impl Visit for LookupHolder {
    fn visit_block(&mut self, block: &Block) {
        // Only process import blocks
        if block.ident.to_string() != "import" {
            visit_block(self, block);
            return;
        }

        // Get properties
        let Some(id) = block.body.get_attribute("id").map(|x| x.value.to_string()) else {
            return;
        };
        let Some(to) = block.body.get_attribute("to").map(|x| x.value.to_string()) else {
            return;
        };

        // Add to lookup table
        self.resource_references_by_id.insert(id, to);
    }
}

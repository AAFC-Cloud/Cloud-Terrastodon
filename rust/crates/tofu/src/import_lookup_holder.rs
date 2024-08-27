use cloud_terrastodon_core_azure::prelude::ScopeImpl;
use hcl::edit::structure::Block;
use hcl::edit::visit::visit_block;
use hcl::edit::visit::Visit;
use std::collections::HashMap;
use tracing::warn;

#[derive(Default)]
pub struct ImportLookupHolder {
    resource_references_by_id: HashMap<ScopeImpl, String>,
}
impl ImportLookupHolder {
    pub fn track(&mut self, id: ScopeImpl, to: String) {
        self.resource_references_by_id.insert(id, to);
    }
    pub fn get_import_to_attribute_from_id(&self, id: &ScopeImpl) -> Option<&String> {
        self.resource_references_by_id.get(id)
    }
}
impl Visit for ImportLookupHolder {
    fn visit_block(&mut self, block: &Block) {
        // Only process import blocks
        if block.ident.to_string() != "import" {
            visit_block(self, block);
            return;
        }

        // Get properties
        let Some(id) = block
            .body
            .get_attribute("id")
            .and_then(|x| x.value.as_str())
        else {
            return;
        };
        let Some(to) = block.body.get_attribute("to").map(|x| x.value.to_string()) else {
            return;
        };

        // Add to lookup table
        let Ok(scope) = id.parse() else {
            warn!("Failed to interpret id as scope: {id:?}");
            return;
        };
        self.track(scope, to);
    }
}

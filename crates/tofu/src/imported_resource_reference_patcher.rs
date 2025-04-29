use crate::import_lookup_holder::ImportLookupHolder;
use cloud_terrastodon_azure::prelude::ScopeImpl;
use hcl::edit::expr::Expression;
use hcl::edit::structure::AttributeMut;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::visit_block_mut;
use std::collections::HashSet;
use tracing::warn;

pub struct ImportedResourceReferencePatcher {
    pub lookups: ImportLookupHolder,
    pub missing_entries: HashSet<ScopeImpl>,
    pub allowed_to_patch_keys: HashSet<String>,
}
impl ImportedResourceReferencePatcher {
    pub fn new(lookup: ImportLookupHolder, allowed_to_patch_keys: HashSet<String>) -> Self {
        ImportedResourceReferencePatcher {
            lookups: lookup,
            missing_entries: Default::default(),
            allowed_to_patch_keys,
        }
    }
}
impl VisitMut for ImportedResourceReferencePatcher {
    fn visit_block_mut(&mut self, node: &mut hcl::edit::structure::Block) {
        // Do not transform hardcoded strings in import blocks lol
        if node.ident.as_str() != "import" {
            visit_block_mut(self, node)
        }
    }
    fn visit_attr_mut(&mut self, mut node: AttributeMut) {
        // Only process allowed keys
        if !self.allowed_to_patch_keys.contains(&node.key.to_string()) {
            visit_attr_mut(self, node);
            return;
        }

        // Only process string literals
        let Some(id) = node.value.as_str() else {
            visit_attr_mut(self, node);
            return;
        };

        // Lookup the key by the id
        let Ok(scope) = id.parse() else {
            warn!("Failed to interpret id as scope: {id:?}");
            return;
        };
        let reference = match self.lookups.get_import_to_attribute_from_id(&scope) {
            Some(x) => x,
            None => {
                self.missing_entries.insert(scope);
                return;
            }
        };

        // Parse the key into a reference expression
        let Ok(expr) = format!("{}.id", reference.trim()).parse::<Expression>() else {
            return;
        };

        // Update the value to use the reference
        *node.value_mut() = expr;
    }
}

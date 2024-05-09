use std::collections::HashSet;

use crate::import_lookup_holder::ImportLookupHolder;
use crate::import_lookup_holder::ResourceId;
use hcl::edit::expr::Expression;
use hcl::edit::structure::AttributeMut;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::VisitMut;

pub struct ImportedResourceReferencePatcher {
    pub lookups: ImportLookupHolder,
    pub missing_entries: HashSet<ResourceId>,
}
impl From<ImportLookupHolder> for ImportedResourceReferencePatcher {
    fn from(lookup: ImportLookupHolder) -> Self {
        ImportedResourceReferencePatcher {
            lookups: lookup,
            missing_entries: Default::default(),
        }
    }
}
impl VisitMut for ImportedResourceReferencePatcher {
    fn visit_attr_mut(&mut self, mut node: AttributeMut) {
        // Only process policy_definition_id attributes
        if node.key.to_string().trim() != "policy_definition_id" {
            visit_attr_mut(self, node);
            return;
        }

        // Only process string literals
        let Some(policy_definition_id) = node.value.as_str() else {
            visit_attr_mut(self, node);
            return;
        };

        // Lookup the policy definition key by the id
        let reference = match self
            .lookups
            .resource_references_by_id
            .get(policy_definition_id)
        {
            Some(x) => x,
            None => {
                self.missing_entries
                    .insert(policy_definition_id.to_string());
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

use std::str::FromStr;

use azure_types::scopes::ScopeImpl;
use hcl::edit::expr::Expression;
use hcl::edit::structure::AttributeMut;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::VisitMut;

use crate::lookup_holder::LookupHolder;
use crate::lookup_holder::ResourceId;

pub struct ReferencePatcher {
    pub lookups: LookupHolder,
    pub missing_entries: Vec<ResourceId>,
}
impl ReferencePatcher {
    pub fn add_data_for_missing(&self, body: &mut Body) {
        for missing in &self.missing_entries {
            let scope = match ScopeImpl::from_str(missing) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("Couldn't determine kind for {missing}: {e:?}");
                    continue;
                }
            };
            match scope {
                ScopeImpl::PolicyDefinition(_) => todo!(),
                ScopeImpl::PolicySetDefinition(_) => todo!(),
                x => todo!("{}", x)
            }
            // println!("Need data block for {missing}");
            // /providers/Microsoft.Authorization/policyDefinitions/
            // /providers/Microsoft.Authorization/policySetDefinitions/
        }
    }
}
impl From<LookupHolder> for ReferencePatcher {
    fn from(lookup: LookupHolder) -> Self {
        ReferencePatcher {
            lookups: lookup,
            missing_entries: Vec::new(),
        }
    }
}
impl VisitMut for ReferencePatcher {
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
                self.missing_entries.push(policy_definition_id.to_string());
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

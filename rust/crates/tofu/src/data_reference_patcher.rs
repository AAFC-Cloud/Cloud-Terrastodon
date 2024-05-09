use crate::data_lookup_holder::DataLookupHolder;
use hcl::edit::expr::Expression;
use hcl::edit::structure::AttributeMut;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::VisitMut;

pub struct DataReferencePatcher {
    pub lookup: DataLookupHolder,
}

impl From<DataLookupHolder> for DataReferencePatcher {
    fn from(lookup: DataLookupHolder) -> Self {
        DataReferencePatcher { lookup }
    }
}
impl VisitMut for DataReferencePatcher {
    fn visit_attr_mut(&mut self, mut node: AttributeMut) {
        // Only process string literals
        let Some(resource_id) = node.value.as_str() else {
            visit_attr_mut(self, node);
            return;
        };

        // Convert/validate strongly typed azure resource ID
        let Ok(resource_id) = resource_id.parse() else {
            return;
        };

        // Lookup the policy definition key by the id
        let reference = match self.lookup.data_references_by_id.get(&resource_id) {
            Some(x) => x,
            None => {
                return;
            }
        };

        // Parse the key into a reference expression
        let Ok(expr) = reference.id_expression_str().parse::<Expression>() else {
            return;
        };

        // Update the value to use the reference
        *node.value_mut() = expr;
    }
}

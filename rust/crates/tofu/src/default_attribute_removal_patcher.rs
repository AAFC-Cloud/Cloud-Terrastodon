use hcl::edit::visit_mut::visit_block_mut;
use hcl::edit::visit_mut::VisitMut;
use tracing::warn;

pub struct DefaultAttributeRemovalPatcher;
impl VisitMut for DefaultAttributeRemovalPatcher {
    fn visit_block_mut(&mut self, node: &mut hcl::edit::structure::Block) {
        match node.labels.first().map(|x| x.to_string()) {
            Some(x) if x == "azurerm_role_assignment" => {
                if node.body.has_attribute("role_definition_name")
                    && node.body.has_attribute("role_definition_id")
                {
                    if node.body.remove_attribute("role_definition_id").is_none() {
                        warn!("Tried to remove non-existant property!");
                    };
                }
            }
            _ => {},
        }
        visit_block_mut(self, node);
    }
}

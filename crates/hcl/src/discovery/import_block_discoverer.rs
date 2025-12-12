use cloud_terrastodon_azure::prelude::ScopeImpl;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Block;
use hcl::edit::visit::Visit;
use std::collections::HashMap;

#[derive(Default)]
pub struct ImportBlockDiscoverer {
    resource_references_by_id: HashMap<ScopeImpl, Expression>,
}
impl ImportBlockDiscoverer {
    pub fn track(&mut self, id: ScopeImpl, to: Expression) {
        self.resource_references_by_id.insert(id, to);
    }
    pub fn get_resource_for_id(&self, id: &ScopeImpl) -> Option<&Expression> {
        self.resource_references_by_id.get(id)
    }
}
impl Visit for ImportBlockDiscoverer {
    fn visit_block(&mut self, block: &Block) {
        // Must be a top-level import block
        if block.ident.to_string() != "import" {
            return;
        }

        // Must contain an `id` attribute
        let Some(id) = block
            .body
            .get_attribute("id")
            .and_then(|x| x.value.as_str())
            .map(ScopeImpl::from)
        else {
            return;
        };

        // Must contain a `to` attribute
        let Some(expr) = block.body.get_attribute("to").map(|x| x.value.clone()) else {
            return;
        };

        // Add to lookup table
        self.track(id, expr);
    }
}

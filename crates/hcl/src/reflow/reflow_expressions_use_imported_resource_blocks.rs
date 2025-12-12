use crate::discovery::ImportBlockDiscoverer;
use crate::reflow::HclReflower;
use cloud_terrastodon_azure::prelude::ScopeImpl;
use hcl::edit::expr::Expression;
use hcl::edit::structure::AttributeMut;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::visit::Visit;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::visit_block_mut;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Default)]
pub struct ReflowExpressionsUseImportedResourceBlocks {
    import_blocks: ImportBlockDiscoverer,
}
#[async_trait::async_trait]
impl HclReflower for ReflowExpressionsUseImportedResourceBlocks {
    async fn reflow(
        &mut self,
        hcl: HashMap<PathBuf, Body>,
    ) -> eyre::Result<HashMap<PathBuf, Body>> {
        let mut reflowed = HashMap::new();
        hcl.values()
            .for_each(|body| self.import_blocks.visit_body(body));
        for (path, mut body) in hcl {
            self.visit_body_mut(&mut body);
            reflowed.insert(path, body);
        }
        Ok(reflowed)
    }
}
impl VisitMut for ReflowExpressionsUseImportedResourceBlocks {
    fn visit_block_mut(&mut self, node: &mut Block) {
        // Must not transform import or terraform blocks
        if ["import", "terraform"].contains(&node.ident.as_str()) {
            return;
        }

        visit_block_mut(self, node)
    }
    fn visit_attr_mut(&mut self, mut node: AttributeMut) {
        // Must be string literal value
        let Some(id) = node.value.as_str().map(ScopeImpl::from) else {
            visit_attr_mut(self, node);
            return;
        };

        // Must have an import block for the ID
        let Some(reference) = self.import_blocks.get_resource_for_id(&id) else {
            return;
        };

        // Must become valid reference expression
        let Ok(expr) = format!("{}.id", reference.to_string().trim()).parse::<Expression>() else {
            return;
        };

        // Update the value to use the reference
        *node.value_mut() = expr;
    }
}

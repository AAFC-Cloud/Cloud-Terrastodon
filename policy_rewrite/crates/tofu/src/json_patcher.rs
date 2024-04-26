use anyhow::anyhow;
use anyhow::Result;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::visit_expr_mut;
use hcl::edit::visit_mut::VisitMut;
use serde_json::Value;

pub struct JsonPatcher;
impl VisitMut for JsonPatcher {
    fn visit_expr_mut(&mut self, node: &mut hcl::edit::expr::Expression) {
        let _ = || -> Result<()> {
            // Must be string
            let Some(node_content) = node.as_str() else {
                // Recurse otherwise
                visit_expr_mut(self, node);
                return Ok(());
            };
            // Must contain a quote
            if !node_content.contains('"') {
                return Err(anyhow!("not json"));
            }

            // Prettify json, failing if not json
            let json = serde_json::to_string_pretty(&serde_json::from_str::<Value>(node_content)?)?;

            // Convert to HCL
            let input = format!(r#"a = jsonencode({json})"#);
            let body: Body = input.parse()?;
            let json_encode_expr = body
                .into_attributes()
                .next()
                .ok_or(anyhow!("'a' not found"))?
                .value;

            // Update node
            *node = json_encode_expr;

            Ok(())
        }();
    }
}

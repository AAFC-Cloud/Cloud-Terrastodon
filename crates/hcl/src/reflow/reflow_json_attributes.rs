use crate::reflow::HclReflower;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::visit_mut::visit_expr_mut;
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct ReflowJsonAttributes;
#[async_trait::async_trait]
impl HclReflower for ReflowJsonAttributes {
    async fn reflow(
        &mut self,
        hcl: HashMap<PathBuf, Body>,
    ) -> eyre::Result<HashMap<PathBuf, Body>> {
        let mut reflowed = HashMap::new();
        for (path, mut body) in hcl {
            self.visit_body_mut(&mut body);
            reflowed.insert(path, body);
        }
        Ok(reflowed)
    }
}
impl VisitMut for ReflowJsonAttributes {
    fn visit_expr_mut(&mut self, node: &mut Expression) {
        // Must be string, continue parsing structures otherwise
        let Some(node_content) = node.as_str() else {
            return visit_expr_mut(self, node);
        };

        // Must contain a quote
        if !node_content.contains('"') {
            return;
        }

        // Must deserialize as json
        let Ok(value) = serde_json::from_str::<Value>(node_content) else {
            return;
        };

        // Must serialize as json
        let Ok(json) = serde_json::to_string_pretty(&value) else {
            return;
        };

        // Must be able to parse as HCL
        let Some(json_encode_expr) = format!(r#"a = jsonencode({json})"#)
            .parse::<Body>()
            .into_iter()
            .flat_map(|body| body.into_attributes())
            .next()
            .map(|attr| attr.value)
        else {
            return;
        };

        // Update node
        *node = json_encode_expr;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    #[test]
    fn it_works() -> eyre::Result<()> {
        // A json example
        let inner = r#"{"guh": true}"#;
        // Encode it as a string
        let encoded = serde_json::to_string(inner)?;
        // Embed it as if it were a string in HCL
        let input = format!(r#"a = {encoded}"#);
        // Parse the body
        let mut body = input.parse()?;
        // Convert embedded json strings
        let mut visitor = ReflowJsonAttributes;
        visitor.visit_body_mut(&mut body);
        // Should now use jsonencode
        assert_eq!(
            format!("{body}\n"),
            indoc! {r#"
            a = jsonencode({
              "guh": true
            })
        "#}
        );
        Ok(())
    }
}

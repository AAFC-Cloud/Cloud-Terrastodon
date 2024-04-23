use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use hcl::edit::structure::Body;
use hcl::edit::visit_mut::visit_expr_mut;
use hcl::edit::visit_mut::VisitMut;
use serde_json::Value;

struct ParametersToJsonVisitor;
impl VisitMut for ParametersToJsonVisitor {
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

pub fn parameters_to_json(body: &mut Body) -> Result<()> {
    let mut visitor = ParametersToJsonVisitor;
    visitor.visit_body_mut(body);
    Ok(())
}

pub fn process_to_files(code: &str, out_dir: &Path) -> Result<Vec<(PathBuf, String)>> {
    // Parse HCL
    let mut body = code.parse()?;

    // Employ jsonparse
    parameters_to_json(&mut body)?;

    // // Split generated into files
    // split_to_files(&body, out_dir)

    Ok(vec![(out_dir.join("generated.tf"), body.to_string())])
}

pub fn split_to_files(body: &Body, out_dir: &Path) -> Result<Vec<(PathBuf, String)>> {
    let mut rtn = Vec::new();
    for block in body.blocks().filter(|b| b.ident.as_str() == "resource") {
        let [kind, name, ..] = block.labels.as_slice() else {
            return Err(anyhow!("failed to destructure").context(format!("{:?}", block.labels)));
        };
        let out_file = out_dir.join(PathBuf::from_iter([
            kind.as_str(),
            format!("{}.tf", name.as_str()).as_str(),
        ]));

        let mut body = Body::new();
        body.push(block.clone());
        rtn.push((out_file, format!("{body}")))
    }

    Ok(rtn)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() -> Result<()> {
        let input = indoc::indoc! {r#"
            resource "azurerm_policy_definition" "my_definition" {
                display_name = "beans"
            }
        "#};

        let body = hcl::parse(input)?;
        let blocks = body
            .blocks()
            .map(|b| b.labels.iter().map(|l| l.as_str()).join(","))
            .join("\n");
        println!("got {:?}", blocks);
        Ok(())
    }
    #[test]
    fn attr() -> Result<()> {
        let input = r#"a = "{\"guh\": true}""#;
        let parsed: Body = input.parse()?;
        println!("got: {:?}", parsed);
        Ok(())
    }
    #[test]
    fn json1() -> Result<()> {
        let input = r#"a = jsonencode({"guh": true})"#;
        let parsed: Body = input.parse()?;
        println!("got: {:?}", parsed);
        Ok(())
    }
    #[test]
    fn json2() -> Result<()> {
        let inner = r#"{"guh": true}"#;
        let input = format!(r#"a = jsonencode({inner})"#);
        let body: Body = input.parse()?;
        let found = &body
            .get_attribute("a")
            .ok_or(anyhow!("'a' not found"))?
            .value;
        println!("got: {:?}", found);
        Ok(())
    }
    #[test]
    fn json3() -> Result<()> {
        // A json example
        let inner = r#"{"guh": true}"#;
        // Encode it as a string
        let encoded = serde_json::to_string(inner)?;
        // Embed it as if it were a string in HCL
        let input = format!(r#"a = {encoded}"#);
        // Parse the body
        let mut body = input.parse()?;
        // Convert embedded json strings
        parameters_to_json(&mut body)?;
        // Should now use jsonencode
        assert_eq!(format!("{body}"), r#"a = jsonencode({"guh":true})"#);
        Ok(())
    }
    #[test]
    fn json4() -> Result<()> {
        let input = r#"{"guh": true}"#;
        let value = serde_json::from_str::<Value>(input)?;
        let output = serde_json::to_string_pretty(&value)?;
        println!("{output}");
        Ok(())
    }
}

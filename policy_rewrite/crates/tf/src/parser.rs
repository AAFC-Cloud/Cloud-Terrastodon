use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use hcl::edit::visit_mut::visit_expr_mut;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::Decorate;
use itertools::Itertools;
use serde_json::Value;
use tokio::fs;

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

fn parameters_to_json(body: &mut Body) -> Result<()> {
    let mut visitor = ParametersToJsonVisitor;
    visitor.visit_body_mut(body);
    Ok(())
}

async fn as_single_body(source_dir: &Path) -> Result<Body> {
    let mut body = Body::new();

    // Read all files in source dir and append to body
    let mut found = fs::read_dir(source_dir).await?;
    while let Some(entry) = found.next_entry().await? {
        let path = entry.path();
        if !path.is_file() || path.extension() != Some(OsStr::new("tf")) {
            println!("Skipping {}", path.display());
            continue;
        }
        let contents = fs::read(&path).await?;
        let text = String::from_utf8(contents)?;
        let found_body: Body = text.parse()?;
        for structure in found_body.into_iter() {
            body.push(structure);
        }
    }
    Ok(body)
}

fn sort_body(src: Body) -> Result<Body> {
    let mut resource_blocks = HashMap::new();
    let mut import_blocks = Vec::new();
    let mut provider_blocks = Vec::new();
    let mut other_blocks = Vec::new();
    let mut attrs = Vec::new();
    for structure in src.into_iter() {
        match structure {
            Structure::Block(mut block) => {
                match block.ident.as_str() {
                    "resource" => {
                        let key = block.labels.iter().map(|l| l.to_string()).join(".");
                        let new_prefix = format!("# key={}\n{}",key, block.decor().prefix().map(|x| x.to_string()).unwrap_or_default());
                        block.decor_mut().set_prefix(new_prefix);
                        resource_blocks.insert(key, block);
                    }
                    "provider" => provider_blocks.push(block),
                    "import" => import_blocks.push(block),
                    _ => other_blocks.push(block),
                };
            }
            Structure::Attribute(attr) => {
                attrs.push(attr);
            }
        }
    }

    let mut dest = Body::new();
    // Add providers at the top
    provider_blocks
        .into_iter()
        .for_each(|block| dest.push(block));

    // Add unknowns
    other_blocks.into_iter().for_each(|block| dest.push(block));
    
    // Add import blocks followed by their resource blocks
    for import_block in import_blocks.into_iter() {
        let Some(key) = import_block.body.get_attribute("to") else {
            dest.push(import_block);
            continue;
        };
        let to = key.value.to_string();
        let key = to.trim();
        let found = resource_blocks.remove(key);
        dest.push(import_block);
        if let Some(resource_block) = found {
            dest.push(resource_block);
        } else {
            eprintln!("Couldn't find resource for import block targetting {key}");
        }
    }
    
    // Add remaining resource blocks
    resource_blocks
        .into_iter()
        .for_each(|(_k,v)| dest.push(v));

    Ok(dest)
}

pub async fn reflow_workspace(
    source_dir: &Path,
    dest_dir: &Path,
) -> Result<Vec<(PathBuf, String)>> {
    let mut body = as_single_body(source_dir).await?;

    // Employ jsonparse
    parameters_to_json(&mut body)?;

    // Sort
    let body = sort_body(body)?;

    // // Split generated into files
    // split_to_files(&body, out_dir)

    Ok(vec![(dest_dir.join("generated.tf"), body.to_string())])
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
    use hcl::edit::structure::Structure;
    use hcl::edit::Decorate;
    use indoc::indoc;
    use itertools::Itertools;

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
    #[test]
    fn linking() -> Result<()> {
        let x = indoc! {r#"
            provider "azurerm" {
                features {}
                skip_provider_registration = true
            }
            # __generated__ by OpenTofu
            # Please review these resources and move them into your main configuration files.

            # __generated__ by OpenTofu from "/providers/Microsoft.Management/managementGroups/MG1/providers/Microsoft.Authorization/policyDefinitions/LA-Microsoft.EventGrid-topics"
            resource "azurerm_policy_definition" "Deploy_Diagnostic_Settings_for_Event_Grid_Topic_to_Log_Analytics_Workspaces" {
                description         = null
                display_name        = "Deploy Diagnostic Settings for Event Grid Topic to Log Analytics Workspaces"
                management_group_id = "/providers/Microsoft.Management/managementGroups/MG1"
                mode = "Indexed"
                name = "LA-Microsoft.EventGrid-topics"
                policy_type = "Custom"
            } # hehe
        "#};
        let body: Body = x.parse()?;
        for structure in body.into_iter() {
            let decor = structure.decor();
            let kind = match structure {
                Structure::Block(ref b) => format!("block {}", b.ident),
                Structure::Attribute(ref a) => format!("attribute {}", a.key),
            };
            println!("Structure {}", kind);
            if let Some(prefix) = decor.prefix()
                && !prefix.is_empty()
            {
                println!("Found prefix {}", prefix.to_string());
            }
            if let Some(suffix) = decor.suffix()
                && !suffix.is_empty()
            {
                println!("Found suffix {}", suffix.to_string());
            }
        }
        Ok(())
    }
}

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::visit_expr_mut;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::Decorate;
use indoc::formatdoc;
use itertools::Itertools;
use serde_json::Value;
use tokio::fs;

struct ParametersToJsonPatcherVisitor;
impl VisitMut for ParametersToJsonPatcherVisitor {
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
    let mut visitor = ParametersToJsonPatcherVisitor;
    visitor.visit_body_mut(body);
    Ok(())
}

#[derive(Default, Clone)]
struct ReferencePatchingSortingVisitor {
    resource_blocks: HashMap<String, Block>,
    resource_keys_by_id: HashMap<String, String>,
    import_blocks: Vec<Block>,
    provider_blocks: Vec<Block>,
    other_blocks: Vec<Block>,
    attrs: Vec<Attribute>,
}
impl ReferencePatchingSortingVisitor {
    pub fn populate(&mut self, src: Body) -> Result<&mut Self> {
        // Populate holders
        for structure in src.into_iter() {
            match structure {
                Structure::Block(block) => {
                    match block.ident.as_str() {
                        "resource" => {
                            // Get the lookup key
                            let key = block.labels.iter().map(|l| l.to_string()).join(".");

                            // Add it to the lookup table
                            self.resource_blocks.insert(key, block);
                        }
                        "provider" => self.provider_blocks.push(block),
                        "import" => {
                            // Add it to the alias table
                            let id = block
                                .body
                                .get_attribute("id")
                                .ok_or(anyhow!("import block missing property").context("id"))?
                                .value
                                .as_str()
                                .ok_or(anyhow!("import block property not a string").context("id"))?
                                .to_string();
                            let key = block
                                .body
                                .get_attribute("to")
                                .ok_or(anyhow!("import block missing property").context("key"))?
                                .value
                                .to_string();
                            self.resource_keys_by_id.insert(id, key);

                            // Add it to the import block list
                            self.import_blocks.push(block);
                        }
                        _ => self.other_blocks.push(block),
                    };
                }
                Structure::Attribute(attr) => {
                    self.attrs.push(attr);
                }
            }
        }
        Ok(self)
    }
    pub fn sorted(mut self) -> Body {
        // Create output
        let mut output = Body::new();

        // Add providers to the top of the output
        self.provider_blocks
            .into_iter()
            .for_each(|block| output.push(block));

        // Add unknowns
        self.other_blocks
            .into_iter()
            .for_each(|block| output.push(block));

        // Sort import blocks by destination
        self.import_blocks
            .sort_by_key(|b| b.body.get_attribute("to").map(|a| a.value.to_string()));

        // Add import blocks followed by their resource blocks
        for mut import_block in self.import_blocks.into_iter() {
            // Import block must contain `to` key
            let Some(key) = import_block.body.get_attribute("to") else {
                output.push(import_block);
                continue;
            };

            // Get and trim key
            let to = key.value.to_string();
            let key = to.trim();

            // Find resource block
            let found = self.resource_blocks.remove(key);
            let Some(mut resource_block) = found else {
                output.push(import_block);
                eprintln!("Couldn't find resource for import block targetting {key}");
                continue;
            };

            // Determine label
            let label = resource_block
                .body
                .get_attribute("display_name")
                .or(resource_block.body.get_attribute("name"))
                .map(|x| x.value.to_string())
                .or(resource_block.labels.get(1).map(|x| x.to_string()))
                .unwrap_or_default();

            // Apply label
            import_block.decor_mut().set_prefix(formatdoc! {"
                #############
                ## {}
                #############
            ", label});

            // Clear block decorations
            resource_block.decor_mut().set_prefix("");

            // Push blocks
            output.push(import_block);
            output.push(resource_block);
        }

        // Add remaining resource blocks
        self.resource_blocks
            .into_iter()
            .for_each(|(_k, v)| output.push(v));

        // Return
        output
    }
}
impl VisitMut for ReferencePatchingSortingVisitor {
    fn visit_attr_mut(&mut self, mut node: hcl::edit::structure::AttributeMut) {
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
        let policy_definition_key = match self.resource_keys_by_id.get(policy_definition_id).ok_or(
            anyhow!("policy assignment policy definition id not present in lookup table")
                .context(format!("tried finding: {policy_definition_id}")),
        ) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("Skipping ID reference update because of problem: {e:?}");
                return;
            }
        };

        // Parse the key into a reference expression
        let Ok(expr) = format!("{}.id", policy_definition_key.trim()).parse::<Expression>() else {
            return;
        };

        // Update the value to use the reference
        *node.value_mut() = expr;
    }
}

fn update_references_and_sort(src: Body) -> Result<Body> {
    // Create visitor
    let mut visitor = ReferencePatchingSortingVisitor::default();

    // Ingest source body
    visitor.populate(src)?;

    // Build sorted body
    let mut output = visitor.clone().sorted();

    // Update references
    visitor.visit_body_mut(&mut output);

    // Return output
    Ok(output)
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

pub async fn reflow_workspace(
    source_dir: &Path,
    dest_dir: &Path,
) -> Result<Vec<(PathBuf, String)>> {
    // Gather all tf files into a single body
    let mut body = as_single_body(source_dir).await?;

    // Switch string literals to using jsonencode
    parameters_to_json(&mut body)?;

    // Update references and sort
    let body = update_references_and_sort(body)?;

    // Return single output file with formatted body as content
    Ok(vec![(dest_dir.join("generated.tf"), body.to_string())])
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

    #[test]
    fn id_replacement() -> Result<()> {
        let x = indoc! {r#"
            import {
                id = "abc"
                to = my.thing
            }
            resource "thing" "main" {
                bruh = "abc"
            }
        "#};
        let x = indoc! {r#"
            a = b.c
        "#};
        let body: Body = x.parse()?;
        let z = &body.get_attribute("a").unwrap().value;
        // body.get_blocks("resource")
        println!("{z:?}");
        Ok(())
    }
}

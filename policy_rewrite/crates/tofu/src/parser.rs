use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Result;
use hcl::edit::expr::Expression;
use hcl::edit::structure::Attribute;
use hcl::edit::structure::AttributeMut;
use hcl::edit::structure::Block;
use hcl::edit::structure::Body;
use hcl::edit::structure::Structure;
use hcl::edit::visit::visit_block;
use hcl::edit::visit::Visit;
use hcl::edit::visit_mut::visit_attr_mut;
use hcl::edit::visit_mut::visit_expr_mut;
use hcl::edit::visit_mut::VisitMut;
use hcl::edit::Decorate;
use indoc::formatdoc;
use itertools::Itertools;
use serde_json::Value;
use tokio::fs;

pub async fn reflow_workspace(
    source_dir: &Path,
    dest_dir: &Path,
) -> Result<Vec<(PathBuf, String)>> {
    // Gather all tf files into a single body
    let mut body = as_single_body(source_dir).await?;

    // Switch string literals to using jsonencode
    let mut json_patcher = JsonPatcher;
    json_patcher.visit_body_mut(&mut body);

    // Build lookup details from body
    let mut lookups = LookupHolder::default();
    lookups.visit_body(&body);

    // Update references from hardcoded IDs to resource attribute references
    let mut reference_patcher: ReferencePatcher = lookups.into();
    reference_patcher.visit_body_mut(&mut body);
    for missing in reference_patcher.missing_entries {
        println!("Need data block for {missing}");
        // /providers/Microsoft.Authorization/policyDefinitions/
        // /providers/Microsoft.Authorization/policySetDefinitions/
    }

    // Format the body
    let body: BodyFormatter = body.try_into()?;
    let body: Body = body.into();

    // Return single output file with formatted body as content
    Ok(vec![(dest_dir.join("generated.tf"), body.to_string())])
}

struct JsonPatcher;
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

#[derive(Default)]
struct LookupHolder {
    resource_references_by_id: HashMap<String, String>,
}
impl Visit for LookupHolder {
    fn visit_block(&mut self, block: &Block) {
        // Only process import blocks
        if block.ident.to_string() != "import" {
            visit_block(self, block);
            return;
        }

        // Get properties
        let Some(id) = block.body.get_attribute("id").map(|x| x.value.to_string()) else {
            return;
        };
        let Some(to) = block.body.get_attribute("to").map(|x| x.value.to_string()) else {
            return;
        };

        // Add to lookup table
        self.resource_references_by_id.insert(id, to);
    }
}

struct ReferencePatcher {
    lookups: LookupHolder,
    missing_entries: Vec<ResourceId>,
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

type ResourceReference = String;
type ResourceId = String;
#[derive(Default, Clone)]
struct BodyFormatter {
    resource_blocks: HashMap<ResourceReference, Block>,
    resource_keys_by_id: HashMap<ResourceId, ResourceReference>,
    import_blocks: Vec<Block>,
    provider_blocks: Vec<Block>,
    other_blocks: Vec<Block>,
    attrs: Vec<Attribute>,
}
impl TryFrom<Body> for BodyFormatter {
    type Error = anyhow::Error;

    fn try_from(src: Body) -> Result<Self> {
        let mut rtn = BodyFormatter::default();
        // Populate holders
        for structure in src.into_iter() {
            match structure {
                Structure::Block(block) => {
                    match block.ident.as_str() {
                        "resource" => {
                            // Get the lookup key
                            let key = block.labels.iter().map(|l| l.to_string()).join(".");

                            // Add it to the lookup table
                            rtn.resource_blocks.insert(key, block);
                        }
                        "provider" => rtn.provider_blocks.push(block),
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
                            rtn.resource_keys_by_id.insert(id, key);

                            // Add it to the import block list
                            rtn.import_blocks.push(block);
                        }
                        _ => rtn.other_blocks.push(block),
                    };
                }
                Structure::Attribute(attr) => {
                    rtn.attrs.push(attr);
                }
            }
        }
        Ok(rtn)
    }
}
impl From<BodyFormatter> for Body {
    fn from(mut value: BodyFormatter) -> Self {
        // Create output
        let mut output = Body::new();

        // Add providers to the top of the output
        value
            .provider_blocks
            .into_iter()
            .for_each(|block| output.push(block));

        // Add unknowns
        value
            .other_blocks
            .into_iter()
            .for_each(|block| output.push(block));

        // Sort import blocks by destination
        value
            .import_blocks
            .sort_by_key(|b| b.body.get_attribute("to").map(|a| a.value.to_string()));

        // Add import blocks followed by their resource blocks
        for mut import_block in value.import_blocks.into_iter() {
            // Import block must contain `to` key
            let Some(key) = import_block.body.get_attribute("to") else {
                output.push(import_block);
                continue;
            };

            // Get and trim key
            let to = key.value.to_string();
            let key = to.trim();

            // Find resource block
            let found = value.resource_blocks.remove(key);
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
        value
            .resource_blocks
            .into_iter()
            .for_each(|(_k, v)| output.push(v));

        // Return
        output
    }
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
        // let x = indoc! {r#"
        //     import {
        //         id = "abc"
        //         to = my.thing
        //     }
        //     resource "thing" "main" {
        //         bruh = "abc"
        //     }
        // "#};
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

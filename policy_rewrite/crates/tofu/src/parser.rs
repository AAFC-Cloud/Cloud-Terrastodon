use anyhow::Result;
use hcl::edit::structure::Body;
use hcl::edit::visit::Visit;
use hcl::edit::visit_mut::VisitMut;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

use crate::body_formatter::BodyFormatter;
use crate::json_patcher::JsonPatcher;
use crate::lookup_holder::LookupHolder;
use crate::reference_patcher::ReferencePatcher;

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
    use anyhow::anyhow;
    use hcl::edit::structure::Structure;
    use hcl::edit::Decorate;
    use hcl::Value;
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
        let mut visitor = JsonPatcher;
        visitor.visit_body_mut(&mut body);
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

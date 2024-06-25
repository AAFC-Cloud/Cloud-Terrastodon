use anyhow::Context;
use anyhow::Result;
use azure::prelude::fetch_all_users;
use hcl::edit::structure::Body;
use hcl::edit::visit::Visit;
use hcl::edit::visit_mut::VisitMut;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tracing::info;
use tracing::instrument;

use crate::body_formatter::PrettyBody;
use crate::data_block_creation::create_data_blocks_for_ids;
use crate::data_reference_patcher::DataReferencePatcher;
use crate::import_lookup_holder::ImportLookupHolder;
use crate::imported_resource_reference_patcher::ImportedResourceReferencePatcher;
use crate::json_patcher::JsonPatcher;
use crate::user_id_reference_patcher::UserIdReferencePatcher;

#[instrument(level = "trace")]
pub async fn reflow_workspace(
    source_dir: &Path,
    dest_dir: &Path,
) -> Result<Vec<(PathBuf, String)>> {
    let mut rtn = Vec::new();

    info!("Assembling body for parsing");
    let mut body = as_single_body(source_dir).await?;
    {
        info!("Updating json string literals to use jsonencode");
        let mut json_patcher = JsonPatcher;
        json_patcher.visit_body_mut(&mut body);
    }
    {
        info!("Gathering import blocks to discover IDs");
        let mut lookups = ImportLookupHolder::default();
        lookups.visit_body(&body);

        info!("Updating strings from hardcoded IDs to reference resource blocks instead");
        let mut import_reference_patcher = ImportedResourceReferencePatcher::new(
            lookups,
            ["policy_definition_id"]
                .into_iter()
                .map(|x| x.to_string())
                .collect(),
        );
        import_reference_patcher.visit_body_mut(&mut body);

        info!("Creating data blocks for hardcoded IDs without a matching resource block");
        let (data_blocks, data_references) =
            create_data_blocks_for_ids(&import_reference_patcher.missing_entries).await?;

        info!("Adding new data blocks to body");
        for block in data_blocks.into_blocks() {
            body.push(block);
        }

        info!("Updating string from hardcoded IDs to reference data blocks instead");
        let mut data_reference_patcher = DataReferencePatcher {
            lookup: data_references,
        };
        data_reference_patcher.visit_body_mut(&mut body);
    }
    {
        info!("Fetching users to perform user ID substitution");
        let users = fetch_all_users()
            .await?
            .into_iter()
            .map(|user| (user.id, user.user_principal_name))
            .collect();

        info!("Performing user ID substitution");
        let mut user_reference_patcher = UserIdReferencePatcher {
            user_principal_name_by_user_id: users,
            used: HashSet::default(),
        };
        user_reference_patcher.visit_body_mut(&mut body);

        info!("Building user lookup");
        if let Some(body) = user_reference_patcher.build_lookup_blocks()? {
            info!("Appending users.tf to output");
            rtn.push((dest_dir.join("users.tf"), body.to_string_pretty()?));
        } else {
            info!("No users referenced, lookup not needed");
        }
    }

    info!("Appending generated.tf to output");
    rtn.push((dest_dir.join("generated.tf"), body.to_string_pretty()?));

    Ok(rtn)
}

pub async fn as_single_body(source_dir: &Path) -> Result<Body> {
    let mut body = Body::new();

    // Read all files in source dir and append to body
    let mut found = fs::read_dir(source_dir).await?;
    while let Some(entry) = found.next_entry().await? {
        let path = entry.path();
        if !path.is_file() || path.extension() != Some(OsStr::new("tf")) {
            info!("Skipping {}", path.display());
            continue;
        }
        info!("Adding {} to body", path.display());
        let contents = fs::read(&path)
            .await
            .context(format!("reading {}", path.display()))?;
        let text =
            String::from_utf8(contents).context(format!("utf-8 parsing {}", path.display()))?;
        let found_body: Body = text
            .parse()
            .context(format!("body parsing {}", path.display()))?;
        for structure in found_body.into_iter() {
            body.push(structure);
        }
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf8_problem() {
        let text = r#"
            import {
                id = "omitted"
                to = azuread_group.écurité
            }
        "#;
        let _body: Body = text.parse().unwrap();
    }
    #[test]
    fn utf8_problem2() {
        let text = r#"
            locals {
                é = 4
            }
            output "ééé" {
            value = local.é
            }
        "#;
        let _body: Body = text.parse().unwrap();
    }
}

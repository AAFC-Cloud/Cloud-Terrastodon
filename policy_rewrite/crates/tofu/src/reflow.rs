use anyhow::Result;
use hcl::edit::structure::Body;
use hcl::edit::visit::Visit;
use hcl::edit::visit_mut::VisitMut;
use tracing::info;
use tracing::instrument;
use std::ffi::OsStr;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

use crate::body_formatter::BodyFormatter;
use crate::data_block_creation::create_data_blocks_for_ids;
use crate::data_reference_patcher::DataReferencePatcher;
use crate::import_lookup_holder::ImportLookupHolder;
use crate::imported_resource_reference_patcher::ImportedResourceReferencePatcher;
use crate::json_patcher::JsonPatcher;

#[instrument(level="debug")]
pub async fn reflow_workspace(
    source_dir: &Path,
    dest_dir: &Path,
) -> Result<Vec<(PathBuf, String)>> {
    // Gather all tf files into a single body
    info!("Assembling body for parsing...");
    let mut body = as_single_body(source_dir).await?;

    // Switch string literals to using jsonencode
    info!("Updating json string literals to use jsonencode...");
    let mut json_patcher = JsonPatcher;
    json_patcher.visit_body_mut(&mut body);

    // Build lookup details from body
    info!("Gathering import blocks...");
    let mut lookups = ImportLookupHolder::default();
    lookups.visit_body(&body);

    // Update references from hardcoded IDs to resource attribute references
    info!("Updating references...");
    let mut import_reference_patcher: ImportedResourceReferencePatcher = lookups.into();
    import_reference_patcher.visit_body_mut(&mut body);

    // Create data blocks
    info!("Creating data blocks for missing references...");
    let (data_blocks, data_references) =
        create_data_blocks_for_ids(&import_reference_patcher.missing_entries).await?;

    // Add data blocks to body
    for block in data_blocks.into_blocks() {
        body.push(block);
    }

    // Update references to data blocks
    let mut data_reference_patcher = DataReferencePatcher {
        lookup: data_references,
    };
    data_reference_patcher.visit_body_mut(&mut body);

    // Format the body
    info!("Formatting final body...");
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
            info!("Skipping {}", path.display());
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

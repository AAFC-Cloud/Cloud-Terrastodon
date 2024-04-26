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
    reference_patcher.add_data_for_missing(&mut body);

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
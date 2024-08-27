use std::path::PathBuf;

use anyhow::Context;
use cloud_terrastodon_core_tofu_types::prelude::CodeReference;
use hcl::edit::structure::Body;
use itertools::Itertools;
use tokio::fs;
use tracing::debug;

pub async fn list_blocks(path: PathBuf) -> anyhow::Result<Vec<CodeReference>> {
    let content = fs::read(&path)
        .await
        .context(format!("reading content from path {}", path.display()))?;
    debug!("Read {} bytes", content.len());
    let content = String::from_utf8(content)
        .context(format!("parsing {} content as utf-8", path.display()))?;
    debug!("Parsed into UTF-8 string of length {}", content.len());
    let body: Body = content
        .parse()
        .context(format!("parsing {} content as body", path.display()))?;
    Ok(body
        .into_blocks()
        .map(|block| CodeReference::from_block(&content, &block, &path))
        .collect_vec())
}

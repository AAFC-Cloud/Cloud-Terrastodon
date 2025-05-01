use cloud_terrastodon_hcl_types::prelude::CodeReference;
use eyre::Context;
use eyre::Result;
use hcl::edit::structure::Body;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;
use tracing::debug;
use tracing::info;

pub async fn list_blocks_for_file(path: PathBuf) -> Result<Vec<CodeReference>> {
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
    body.into_blocks()
        .map(|block| CodeReference::try_from_block(&content, block, &path))
        .collect()
}

pub async fn list_blocks_for_dir(path: impl AsRef<Path>) -> Result<Vec<CodeReference>> {
    // We don't use `as_single_body` because we need to track the files that each block comes from
    let mut files = fs::read_dir(path).await.context("reading dir")?;
    let mut rtn = Vec::new();
    let mut num_files = 0;
    while let Some(tf_file) = files.next_entry().await.context("reading entry")? {
        let kind = tf_file.file_type().await.context("getting file type")?;
        if !kind.is_file() {
            continue;
        }
        let path = tf_file.path();
        if path
            .extension()
            .filter(|ext| ext.to_string_lossy() == "tf")
            .is_none()
        {
            continue;
        }

        num_files += 1;
        info!("Gathering blocks from {}", path.display());

        let mut blocks = list_blocks_for_file(path).await.context("listing blocks")?;
        rtn.append(&mut blocks);
    }

    info!("Found {} blocks across {} files", rtn.len(), num_files);
    Ok(rtn)
}

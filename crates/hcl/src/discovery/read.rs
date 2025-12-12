use eyre::Context;
use eyre::Result;
use hcl::edit::structure::Body;
use std::ffi::OsStr;
use std::path::Path;
use tracing::debug;
use tracing::instrument;

#[instrument(level = "debug")]
pub async fn try_read_hcl_file(path: &Path) -> Result<Option<Body>> {
    if !path.is_file() || path.extension() != Some(OsStr::new("tf")) {
        debug!("Path is not a .tf file, skipping");
        return Ok(None);
    }

    debug!("Reading .tf file contents into byte array");
    let contents = tokio::fs::read(&path)
        .await
        .context(format!("reading {}", path.display()))?;

    debug!("Parsing .tf file contents as UTF-8 string");
    let text = String::from_utf8(contents).context(format!("utf-8 parsing {}", path.display()))?;

    debug!("Parsing .tf file contents as HCL body");
    let body: Body = text
        .parse()
        .context(format!("body parsing {}", path.display()))?;
    Ok(Some(body))
}

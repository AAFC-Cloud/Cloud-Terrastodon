use hcl::edit::structure::Body;
use std::collections::HashMap;
use std::path::PathBuf;

#[async_trait::async_trait]
pub trait HclReflower: Send {
    async fn reflow(&mut self, hcl: HashMap<PathBuf, Body>)
    -> eyre::Result<HashMap<PathBuf, Body>>;
}

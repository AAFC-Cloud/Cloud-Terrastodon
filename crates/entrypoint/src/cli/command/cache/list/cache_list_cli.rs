use clap::Args;
use cloud_terrastodon_command::discover_caches;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use itertools::Itertools;
use serde_json::json;

#[derive(Args, Debug, Clone)]
pub struct CacheListArgs {}

impl CacheListArgs {
    pub async fn invoke(self) -> Result<()> {
        // Discover caches under AppDir::Commands
        let root = AppDir::Commands.as_path_buf();
        let caches = discover_caches(&root).await?;

        // Convert to a serializable form (path string and rfc2822 string)
        let payload: Vec<serde_json::Value> = caches
            .into_iter()
            .sorted_by(|a,b| b.1.cmp(&a.1)) // sort by last_used desc
            .map(|(cache_key, dt)| json!({"path": cache_key.path_on_disk().to_string_lossy().into_owned(), "last_used": dt.to_rfc2822()}))
            .collect();

        println!("{}", serde_json::to_string_pretty(&payload)?);
        Ok(())
    }
}

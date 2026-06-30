use cloud_terrastodon_command::discover_caches;
use cloud_terrastodon_pathing::AppDir;
use eyre::Result;
use itertools::Itertools;

#[derive(facet::Facet, Debug, Clone)]
pub struct CacheListArgs {}

#[derive(Debug, facet::Facet)]
struct CacheListEntry {
    path: String,
    last_used: String,
}

impl CacheListArgs {
    pub async fn invoke(self) -> Result<()> {
        // Discover caches under AppDir::Commands
        let root = AppDir::Commands.as_path_buf();
        let caches = discover_caches(&root).await?;

        // Convert to a serializable form (path string and rfc2822 string)
        let payload: Vec<CacheListEntry> = caches
            .into_iter()
            .sorted_by(|a, b| b.1.cmp(&a.1)) // sort by last_used desc
            .map(|(cache_key, dt)| CacheListEntry {
                path: cache_key.path_on_disk().to_string_lossy().into_owned(),
                last_used: dt.to_rfc2822(),
            })
            .collect();

        println!("{}", cloud_terrastodon_command::to_string_pretty(&payload)?);
        Ok(())
    }
}

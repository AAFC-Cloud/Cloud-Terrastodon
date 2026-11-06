use cloud_terrastodon_cache::CacheInventorySelection;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;

/// List execution artifact identities and their selected observation timestamps.
#[derive(Facet, Debug, Clone)]
pub struct CacheListArgs {}

impl CacheListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let mut entries = executor.cache()?.inventory(executor.cancellation()).await?;
        entries.sort_by_key(|entry| {
            std::cmp::Reverse(match &entry.selection {
                CacheInventorySelection::Selected(freshness) => Some(freshness.observed_at),
                CacheInventorySelection::Absent | CacheInventorySelection::Invalid { .. } => None,
            })
        });
        println!(
            "{}",
            facet_json::to_string_pretty(&entries).map_err(|error| eyre::eyre!("{error:?}"))?
        );
        Ok(())
    }
}

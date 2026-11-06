use cloud_terrastodon_cache::CacheInventorySelection;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_user_input::Choice;
use cloud_terrastodon_user_input::PickerTui;
use eyre::Result;
use facet::Facet;
use std::collections::HashSet;

/// Invalidate selected logical cache scopes across execution and authentication variants.
#[derive(Facet, Debug, Clone)]
pub struct CacheCleanTuiArgs {}

impl CacheCleanTuiArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<()> {
        let cache = executor.cache()?;
        let choices: Vec<Choice<CacheScope>> = cache
            .inventory(executor.cancellation())
            .await?
            .into_iter()
            .map(|entry| Choice {
                key: format!(
                    "{:?}: {} ({})",
                    entry.identity.kind,
                    entry.identity.scope.components().join("/"),
                    match entry.selection {
                        CacheInventorySelection::Absent => "no selected artifact".to_owned(),
                        CacheInventorySelection::Selected(freshness) => {
                            format!("observed {}", freshness.observed_at.to_rfc2822())
                        }
                        CacheInventorySelection::Invalid { message } => {
                            format!("invalid metadata: {message}")
                        }
                    },
                ),
                value: entry.identity.scope,
            })
            .collect();

        let picked = PickerTui::<_>::new()
            .set_header("Select cache scopes to invalidate across all variants")
            .pick_many(choices)
            .await?;

        // Several physical variants can share one logical scope.
        let mut invalidated = HashSet::new();
        for scope in picked {
            if invalidated.insert(scope.clone()) {
                cache.invalidate(&scope, executor.cancellation()).await?;
                println!("Invalidated cache scope: {}", scope.components().join("/"));
            }
        }
        Ok(())
    }
}

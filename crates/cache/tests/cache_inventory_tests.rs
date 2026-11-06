#![cfg(test)]

use cloud_terrastodon_cache::ArtifactObservation;
use cloud_terrastodon_cache::ArtifactRetention;
use cloud_terrastodon_cache::ArtifactSet;
use cloud_terrastodon_cache::AuthPartition;
use cloud_terrastodon_cache::CacheInventorySelection;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheOperationOwner;
use cloud_terrastodon_cache::ExecutorKind;
use cloud_terrastodon_cache::NonEmptyString;
use cloud_terrastodon_cache::WorkCache;
use std::collections::BTreeMap;
use teamy_cancellation::CancellationToken;

struct SyntheticRead;

#[tokio::test]
async fn inventory_tracks_selection_without_loading_payloads_or_changing_freshness()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let cache = WorkCache::temporary()?;
            assert!(cache.inventory(&scope).await?.is_empty());
            let key = CacheKey::from_relative_path("synthetic/readable scope")?;
            let entry = cache.artifacts::<String>(
                ExecutorKind::Rest,
                key.clone(),
                CacheOperationOwner::of::<SyntheticRead>(),
                AuthPartition::new("synthetic-session")?,
                NonEmptyString::new("request")?,
                ArtifactRetention::Full,
            )?;
            let completed = entry
                .begin_attempt(BTreeMap::new(), &scope)
                .await?
                .unwrap()
                .finish(
                    ArtifactSet {
                        payload: b"synthetic response".to_vec(),
                        diagnostics: BTreeMap::new(),
                    },
                    ArtifactObservation::now(),
                    &scope,
                )
                .await?;
            assert!(entry.publish(&completed, &scope).await?);
            let first = cache.inventory(&scope).await?;
            assert_eq!(first.len(), 1);
            assert_eq!(first[0].identity.scope, key.scope);
            assert!(matches!(
                first[0].selection,
                CacheInventorySelection::Selected(_)
            ));
            // Inspection reports selection metadata even when a payload is unusable;
            // only execution's verified artifact loader decides whether it can reuse it.
            let attempts = entry.directory()?.join("attempts");
            for attempt in std::fs::read_dir(attempts)? {
                let payload = attempt?.path().join("payload.bin");
                if payload.is_file() {
                    std::fs::remove_file(payload)?;
                }
            }
            assert_eq!(
                cache.inventory(&scope).await?[0].selection,
                first[0].selection
            );
            cache.invalidate(&key.scope, &scope).await?;
            let invalidated = cache.inventory(&scope).await?;
            assert_eq!(invalidated.len(), 1);
            assert_eq!(invalidated[0].selection, CacheInventorySelection::Absent);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn damaged_selection_metadata_remains_visible_for_invalidation() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            for damage in ["selector", "manifest", "missing manifest"] {
                let cache = WorkCache::temporary()?;
                let key = CacheKey::from_relative_path("synthetic/damaged-selection")?;
                let entry = cache.artifacts::<String>(
                    ExecutorKind::Rest,
                    key.clone(),
                    CacheOperationOwner::of::<SyntheticRead>(),
                    AuthPartition::new("synthetic-session")?,
                    NonEmptyString::new("request")?,
                    ArtifactRetention::Full,
                )?;
                let completed = entry
                    .begin_attempt(BTreeMap::new(), &scope)
                    .await?
                    .unwrap()
                    .finish(
                        ArtifactSet {
                            payload: b"synthetic response".to_vec(),
                            diagnostics: BTreeMap::new(),
                        },
                        ArtifactObservation::now(),
                        &scope,
                    )
                    .await?;
                assert!(entry.publish(&completed, &scope).await?);
                match damage {
                    "selector" => {
                        std::fs::write(entry.directory()?.join("selection.json"), b"damaged")?
                    }
                    "manifest" => {
                        std::fs::write(completed.directory().join("manifest.json"), b"damaged")?
                    }
                    "missing manifest" => {
                        std::fs::remove_file(completed.directory().join("manifest.json"))?
                    }
                    _ => unreachable!(),
                }

                let inventory = cache.inventory(&scope).await?;
                assert_eq!(inventory.len(), 1);
                assert_eq!(inventory[0].identity.scope, key.scope);
                let CacheInventorySelection::Invalid { message } = &inventory[0].selection else {
                    panic!("{damage} should be reported as invalid metadata");
                };
                assert!(!message.is_empty());

                cache.invalidate(&key.scope, &scope).await?;
                assert!(!entry.directory()?.join("selection.json").exists());
                assert_eq!(
                    cache.inventory(&scope).await?[0].selection,
                    CacheInventorySelection::Absent
                );
            }
            Ok(())
        })
        .await
}

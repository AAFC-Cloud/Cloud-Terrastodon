use crate::CacheInventoryEntry;
use crate::WorkCache;
use crate::artifact_directory::ArtifactDirectory;
use crate::artifact_io_guard::ArtifactIoGuard;
use eyre::Report;
use eyre::Result;
use teamy_cancellation::CancellationScope;

impl WorkCache {
    /// Inspect addresses and selected metadata under this owner's root.
    ///
    /// Reads no payloads or credential inputs and does not extend freshness.
    /// The artifact gate serializes this snapshot with publication/invalidation.
    pub async fn inventory(
        &self,
        cancellation: &CancellationScope,
    ) -> Result<Vec<CacheInventoryEntry>> {
        let gate = cancellation
            .cancellable_scope(|_child| async {
                Ok::<_, Report>(ArtifactIoGuard::acquire(&self.state.artifact_gate).await)
            })
            .await?;
        let cache = self.clone();
        let cancellation = cancellation.clone();
        let (_gate, inventory) = gate
            .run_blocking(move || ArtifactDirectory::inventory(cache.root(), &cancellation))
            .await?;
        inventory
    }
}

#[cfg(test)]
mod tests {
    use crate::WorkCache;
    use crate::artifact_io_guard::ArtifactIoGuard;
    use std::future::Future;
    use std::future::IntoFuture;
    use std::future::poll_fn;
    use std::task::Poll;
    use std::time::Duration;
    use teamy_cancellation::CancellationToken;
    use tokio::time::timeout;

    #[tokio::test]
    async fn inventory_can_cancel_while_waiting_for_artifact_io() -> eyre::Result<()> {
        let cache = WorkCache::temporary()?;
        let gate = ArtifactIoGuard::acquire(&cache.state.artifact_gate).await;
        let token = CancellationToken::new();
        let inventory = token
            .with_async_scope(|scope| {
                let cache = &cache;
                async move { cache.inventory(&scope).await }
            })
            .into_future();
        tokio::pin!(inventory);
        poll_fn(|context| {
            assert!(inventory.as_mut().poll(context).is_pending());
            Poll::Ready(())
        })
        .await;
        token.request_cancel("stop inventory while another artifact operation owns the gate");
        assert!(timeout(Duration::from_secs(10), inventory).await?.is_err());
        assert!(cache.state.artifact_gate.try_lock().is_err());
        drop(gate);

        CancellationToken::new()
            .with_async_scope(|scope| {
                let cache = &cache;
                async move {
                    assert!(cache.inventory(&scope).await?.is_empty());
                    Ok(())
                }
            })
            .await
    }
}

use crate::CacheEntryIdentity;
use crate::CacheScope;
use crate::WorkCache;
use crate::artifact_directory::ArtifactDirectory;
use crate::artifact_io_guard::ArtifactIoGuard;
use crate::cache_invalidation_target::CacheInvalidationTarget;
use eyre::Context;
use eyre::Result;
use eyre::ensure;
use teamy_cancellation::CancellationScope;
use uuid::Uuid;

impl WorkCache {
    /// Invalidate a logical subtree across executor namespaces and variants.
    ///
    /// Existing caller snapshots remain readable. In-flight attempts keep their
    /// original generations and cannot republish after this transition. Durable
    /// failure leaves disk reuse quarantined until a covering invalidation succeeds.
    pub async fn invalidate(
        &self,
        scope: &CacheScope,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.invalidate_target(CacheInvalidationTarget::Scope(scope.clone()), cancellation)
            .await
    }

    /// Reject one canonical raw payload without invalidating independent pages.
    ///
    /// Call after the owning decoder rejects a disk result. This does not retry
    /// execution; callers retain the original decoder error and rejected artifacts.
    pub async fn reject(
        &self,
        identity: &CacheEntryIdentity,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        self.invalidate_target(
            CacheInvalidationTarget::Entry(identity.clone()),
            cancellation,
        )
        .await
    }

    async fn invalidate_target(
        &self,
        target: CacheInvalidationTarget,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        // User-owned result destructors run after this future releases its locks,
        // including when durable invalidation returns an error.
        let mut retired = Vec::new();
        let result = cancellation
            .deferred("invalidate reusable cache results", |_protected| async {
                let gate = ArtifactIoGuard::acquire(&self.state.artifact_gate).await;
                {
                    let mut state = self.lock_state()?;
                    // No error or dropped future after this point may reopen disk reuse.
                    let invalidation = Uuid::new_v4();
                    ensure!(
                        !state.invalidations.contains_key(&invalidation),
                        "cache invalidation identity collision"
                    );
                    state.invalidations.insert(invalidation, target.clone());
                    let identities = state
                        .values
                        .keys()
                        .filter(|identity| target.covers(identity))
                        .cloned()
                        .collect::<Vec<_>>();
                    retired.extend(
                        identities
                            .into_iter()
                            .filter_map(|identity| state.values.remove(&identity)),
                    );
                    state.generations.advance(&target)?;
                }
                let root = self.root().to_path_buf();
                let disk_target = target.clone();
                let (_gate, invalidated) = gate
                    .run_blocking(move || {
                        ArtifactDirectory::invalidate_selections(&root, &disk_target)
                    })
                    .await?;
                invalidated
                    .wrap_err("cache disk reuse remains quarantined after invalidation failure")?;
                let mut state = self.lock_state()?;
                state
                    .invalidations
                    .retain(|_, failed| !target.contains(failed));
                state
                    .publication_quarantine
                    .retain(|identity| !target.covers(identity));
                Ok(())
            })
            .await;
        drop(retired);
        result
    }
}

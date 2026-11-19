use crate::WorkCache;
use crate::cache_ownership_claims::CacheOwnershipClaims;
use crate::work_cache_root::WorkCacheRoot;
use crate::work_cache_state::WorkCacheState;
use eyre::Result;
use eyre::eyre;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::Weak;

/// Shares cache ownership among overlapping jobs using the same persistent root.
///
/// Jobs and their artifact workers retain the active owner through a
/// [`WorkCache`]. When all handles drop, the next job starts with fresh cache
/// state and rechecks the disk. Previously returned values can remain in use
/// as snapshots. Canonical operation and output claims survive between jobs.
#[derive(Debug)]
pub struct WorkCacheActivity {
    root: PathBuf,
    claims: Arc<CacheOwnershipClaims>,
    active: Mutex<Weak<WorkCacheState>>,
}

impl WorkCacheActivity {
    /// Bind a persistent path without performing I/O.
    ///
    /// All jobs within the activity must request their handles from this
    /// coordinator. An independently retained owner of the same root can
    /// block their progress until it drops. Temporary roots with owned cleanup
    /// use [`WorkCache::from_tempdir`].
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        WorkCacheRoot::validate_path(&root)?;
        Ok(Self {
            root,
            claims: Arc::default(),
            active: Mutex::default(),
        })
    }

    /// Obtain the current job owner or create one with fresh cache state.
    ///
    /// This does not acquire the OS lock. That happens on the first operation
    /// requiring it, through [`WorkCache::acquire`].
    pub fn work_cache(&self) -> Result<WorkCache> {
        let mut active = self
            .active
            .lock()
            .map_err(|_| eyre!("cache activity owner lock was poisoned"))?;
        if let Some(state) = active.upgrade() {
            return Ok(WorkCache { state });
        }
        let cache = WorkCache::with_claims(
            WorkCacheRoot::Path(self.root.clone()),
            Arc::clone(&self.claims),
        );
        *active = Arc::downgrade(&cache.state);
        Ok(cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CacheKey;
    use crate::CacheOperationOwner;
    use crate::CacheScope;
    use crate::ExecutorKind;
    use std::time::Duration;

    struct FirstOperation;
    struct SecondOperation;

    #[test]
    fn claims_survive_an_idle_interval_without_opening_the_root() -> Result<()> {
        let temporary = tempfile::tempdir()?;
        let root = temporary.path().join("not-created");
        let activity = WorkCacheActivity::new(&root)?;
        let key = CacheKey {
            scope: CacheScope::try_from_components(["synthetic", "read"])?,
            valid_for: Duration::from_secs(60),
        };
        let operation = CacheOperationOwner::of::<FirstOperation>();
        activity
            .work_cache()?
            .claim::<String>(ExecutorKind::Rest, &key, operation)?;
        assert!(activity.active.lock().unwrap().upgrade().is_none());

        let next = activity.work_cache()?;
        assert!(
            next.claim::<String>(
                ExecutorKind::Rest,
                &key,
                CacheOperationOwner::of::<SecondOperation>(),
            )
            .is_err(),
        );
        assert!(
            next.claim::<u64>(ExecutorKind::Rest, &key, operation)
                .is_err()
        );
        next.claim::<String>(ExecutorKind::Rest, &key, operation)?;
        assert!(!root.exists());
        Ok(())
    }
}

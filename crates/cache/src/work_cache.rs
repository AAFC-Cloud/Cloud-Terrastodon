use crate::ArtifactEntry;
use crate::ArtifactMetadata;
use crate::ArtifactRetention;
use crate::AuthPartition;
use crate::CacheEntryIdentity;
use crate::CacheKey;
use crate::CacheOperationOwner;
use crate::CachedWorkSpec;
use crate::ExecutorKind;
use crate::NonEmptyString;
use crate::WorkAttempt;
use crate::WorkOutput;
use crate::WorkReuse;
use crate::cache_ownership_claim::CacheOwnershipClaim;
use crate::cache_state::CacheState;
use crate::work_cache_state::WorkCacheState;
use crate::work_child_guard::WorkChildGuard;
use eyre::Result;
use eyre::ensure;
use eyre::eyre;
use std::any::TypeId;
use std::any::type_name;
use std::future::Future;
use std::marker::PhantomData;
use std::panic::Location;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::time::Duration;
use teamy_cancellation::CancellationScope;

/// An explicitly owned cache root and its in-process logical operation claims.
///
/// Clones share the same claims. Separately constructed handles are isolated,
/// even when given the same root. Construction and claiming perform no I/O,
/// resolve no defaults, and never consult the legacy command cache.
///
/// Raw artifacts and shared decoded results use this owner, including scoped
/// generation invalidation and transitive semantic dependency freshness.
#[derive(Clone, Debug)]
pub struct WorkCache {
    // One shared owner keeps the root and all accepted claims together.
    pub(crate) state: Arc<WorkCacheState>,
}

impl WorkCache {
    /// Bind canonical work after checking its operation/output ownership.
    ///
    /// Declare reuse eligibility before lookup and pass the callback's attempt
    /// token to all semantic child work. A reusable initializer cannot call a
    /// nonreusable child. Zero validity executes independently on every
    /// await and never joins or publishes a shared result. Initializers may borrow
    /// caller data; only retained outputs require `Send + Sync + 'static`.
    #[track_caller]
    #[allow(
        clippy::too_many_arguments,
        reason = "explicit identity, reuse policy, ancestry, and cancellation are independent contracts"
    )]
    pub fn cached_work<'a, T, F, Fut>(
        &self,
        identity: CacheEntryIdentity,
        operation_owner: CacheOperationOwner,
        reuse: WorkReuse,
        valid_for: Duration,
        parent: Option<&'a WorkAttempt>,
        cancellation: &'a CancellationScope,
        initialize: F,
    ) -> Result<CachedWorkSpec<'a, T, F>>
    where
        T: Send + Sync + 'static,
        F: FnOnce(Option<WorkAttempt>, CancellationScope) -> Fut + Send + 'a,
        Fut: Future<Output = Result<WorkOutput<T>>> + Send + 'a,
    {
        let reusable = reuse == WorkReuse::ReusableRead && !valid_for.is_zero();
        let child = parent
            .map(|parent| WorkChildGuard::admit(self, parent, reusable))
            .transpose()?;
        self.claim::<T>(
            identity.kind.clone(),
            &CacheKey {
                scope: identity.scope.clone(),
                valid_for,
            },
            operation_owner,
        )?;
        Ok(CachedWorkSpec {
            cache: self.clone(),
            identity,
            validity: valid_for,
            reusable,
            parent,
            child,
            cancellation,
            initialize,
            output: PhantomData,
        })
    }

    /// Reclaim decoded values and dead coordination entries. Existing snapshots
    /// and live initializer locks survive. This is memory eviction, not disk or
    /// generation invalidation; already-running work may publish afterward.
    pub fn release_memory(&self) -> Result<()> {
        let values = {
            let mut state = self.lock_state()?;
            state
                .coordination
                .retain(|_, entry| entry.strong_count() > 0);
            std::mem::take(&mut state.values)
        };
        drop(values);
        Ok(())
    }

    pub(crate) fn lock_state(&self) -> Result<MutexGuard<'_, CacheState>> {
        self.state
            .registry
            .lock()
            .map_err(|_| eyre!("cache state was poisoned while coordinating work"))
    }

    /// Claim a concrete raw-artifact identity before any disk access. The
    /// partition must be admitted by authentication policy and contain no secret;
    /// fingerprints must cover all semantic inputs without exposing credentials.
    #[track_caller]
    pub fn artifacts<T: 'static>(
        &self,
        kind: ExecutorKind,
        key: CacheKey,
        operation_owner: CacheOperationOwner,
        auth_partition: AuthPartition,
        fingerprint: NonEmptyString,
        retention: ArtifactRetention,
    ) -> Result<ArtifactEntry> {
        self.claim::<T>(kind.clone(), &key, operation_owner)?;
        Ok(ArtifactEntry {
            cache: self.clone(),
            identity: CacheEntryIdentity {
                kind,
                scope: key.scope,
                auth_partition,
                fingerprint,
            },
            metadata: ArtifactMetadata::new::<T>(operation_owner),
            validity: key.valid_for,
            retention,
            work: None,
        })
    }

    /// Establish an explicit root without creating, inspecting, or reading it.
    ///
    /// Roots must be absolute on the current platform. Unresolved `..`
    /// components are rejected as a library identity constraint, not a platform
    /// restriction. Symlinks are not resolved, and even the filesystem root is
    /// accepted. Storage validates its requirements when performing I/O.
    pub fn new(root: impl Into<PathBuf>) -> Result<Self> {
        let root = root.into();
        ensure!(root.is_absolute(), "cache root must be absolute: {root:?}");
        ensure!(
            !root
                .components()
                .any(|component| component == Component::ParentDir),
            "cache root must not contain unresolved parent traversal: {root:?}"
        );
        Ok(Self {
            state: Arc::new(WorkCacheState {
                root,
                registry: Mutex::new(CacheState::default()),
                artifact_gate: Arc::new(tokio::sync::Mutex::new(())),
            }),
        })
    }

    /// The explicit root, without any filesystem inspection or normalization.
    pub fn root(&self) -> &Path {
        &self.state.root
    }

    /// Reserve or verify the canonical owner and output type for a scope.
    ///
    /// The address is `(kind, key.scope)`: validity, request fingerprints, and
    /// authentication variants must not hide a conflicting operation contract.
    /// A conflict is an error, never a cache miss or permission to replace the
    /// earlier claim. Locations are diagnostic and do not determine ownership.
    ///
    /// The synchronous lock covers only this short in-memory check. It provides
    /// no single-in-flight execution guarantee and must not be held across work.
    /// Only type identity is retained: this check adds no `Send` or `Sync` bounds
    /// to a result. APIs that retain shared values must impose their own bounds.
    #[track_caller]
    pub fn claim<T: 'static>(
        &self,
        kind: ExecutorKind,
        key: &CacheKey,
        owner: CacheOperationOwner,
    ) -> Result<()> {
        let caller = Location::caller();
        let mut state = self.state.registry.lock().map_err(|_| {
            eyre!("cache ownership registry was poisoned by a panic while holding its lock")
        })?;
        let address = (kind, key.scope.clone());
        if let Some(existing) = state.claims.get(&address) {
            ensure!(
                existing.owner == owner,
                "cache operation owner conflict for {:?} scope {:?}: {} first claimed at {}; {} attempted at {}",
                address.0,
                address.1,
                existing.owner.name(),
                existing.first_claim,
                owner.name(),
                caller,
            );
            ensure!(
                existing.output_type_id == TypeId::of::<T>(),
                "cache output type conflict for {:?} scope {:?}, owner {}: {} first claimed at {}; {} attempted at {}",
                address.0,
                address.1,
                owner.name(),
                existing.output_type_name,
                existing.first_claim,
                type_name::<T>(),
                caller,
            );
        } else {
            state.claims.insert(
                address,
                CacheOwnershipClaim {
                    owner,
                    output_type_id: TypeId::of::<T>(),
                    output_type_name: type_name::<T>(),
                    first_claim: caller,
                },
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CacheScope;
    use std::cell::Cell;
    use std::sync::Barrier;
    use std::thread;
    use std::time::Duration;

    struct FirstOperation;
    struct SecondOperation;

    fn key() -> CacheKey {
        CacheKey {
            scope: CacheScope::try_from_components(["projects", "sample", "list"]).unwrap(),
            valid_for: Duration::from_secs(60),
        }
    }

    #[test]
    fn different_owners_conflict_even_with_the_same_output_and_different_validity() {
        let temporary = tempfile::tempdir().unwrap();
        let cache = WorkCache::new(temporary.path()).unwrap();
        let mut key = key();
        let first = CacheOperationOwner::of::<FirstOperation>();
        let second = CacheOperationOwner::of::<SecondOperation>();
        cache
            .claim::<String>(ExecutorKind::Rest, &key, first)
            .unwrap();

        key.valid_for = Duration::ZERO;
        let conflict = cache
            .claim::<String>(ExecutorKind::Rest, &key, second)
            .unwrap_err();
        assert!(conflict.to_string().contains(first.name()));
        assert!(conflict.to_string().contains(second.name()));
        // A rejected claim does not replace the accepted contract.
        cache
            .claim::<String>(ExecutorKind::Rest, &key, first)
            .unwrap();
    }

    #[test]
    fn an_operation_cannot_reinterpret_its_scope_as_a_different_output() {
        let temporary = tempfile::tempdir().unwrap();
        let cache = WorkCache::new(temporary.path()).unwrap();
        let owner = CacheOperationOwner::of::<FirstOperation>();
        cache
            .claim::<String>(ExecutorKind::Rest, &key(), owner)
            .unwrap();
        let conflict = cache
            .claim::<Vec<String>>(ExecutorKind::Rest, &key(), owner)
            .unwrap_err();
        assert!(conflict.to_string().contains(type_name::<String>()));
        assert!(conflict.to_string().contains(type_name::<Vec<String>>()));
        cache
            .claim::<String>(ExecutorKind::Rest, &key(), owner)
            .unwrap();
    }

    #[test]
    fn a_nonreusable_result_can_claim_its_type_without_sharing_the_value() {
        let temporary = tempfile::tempdir().unwrap();
        let cache = WorkCache::new(temporary.path()).unwrap();
        let key = CacheKey {
            valid_for: Duration::ZERO,
            ..key()
        };
        let owner = CacheOperationOwner::of::<FirstOperation>();
        cache
            .claim::<Cell<u64>>(ExecutorKind::Rest, &key, owner)
            .unwrap();
        assert!(cache.claim::<u64>(ExecutorKind::Rest, &key, owner).is_err());
    }

    #[test]
    fn namespaces_and_logical_scopes_have_independent_contracts() {
        let temporary = tempfile::tempdir().unwrap();
        let cache = WorkCache::new(temporary.path()).unwrap();
        let first = CacheOperationOwner::of::<FirstOperation>();
        let second = CacheOperationOwner::of::<SecondOperation>();
        cache
            .claim::<String>(ExecutorKind::Command, &key(), first)
            .unwrap();
        cache
            .claim::<u64>(ExecutorKind::Rest, &key(), second)
            .unwrap();
        cache
            .claim::<Vec<String>>(ExecutorKind::InProcess, &key(), first)
            .unwrap();
        cache
            .claim::<bool>(ExecutorKind::Other("Rest".to_owned()), &key(), first)
            .unwrap();
        let child = CacheKey {
            scope: key().scope.join("page/1").unwrap(),
            ..key()
        };
        cache
            .claim::<String>(ExecutorKind::Rest, &child, first)
            .unwrap();
        let other = CacheKey {
            scope: CacheScope::try_from_components(["projects", "other", "list"]).unwrap(),
            ..key()
        };
        cache
            .claim::<String>(ExecutorKind::Rest, &other, first)
            .unwrap();
        assert!(
            cache
                .claim::<String>(ExecutorKind::Other("Rest".to_owned()), &key(), first)
                .is_err()
        );
    }

    #[test]
    fn clones_share_claims_but_separately_constructed_handles_do_not() {
        let temporary = tempfile::tempdir().unwrap();
        let cache = WorkCache::new(temporary.path()).unwrap();
        let cloned = cache.clone();
        let isolated = WorkCache::new(temporary.path()).unwrap();
        let first = CacheOperationOwner::of::<FirstOperation>();
        let second = CacheOperationOwner::of::<SecondOperation>();
        cache
            .claim::<String>(ExecutorKind::Rest, &key(), first)
            .unwrap();
        assert!(
            cloned
                .claim::<String>(ExecutorKind::Rest, &key(), second)
                .is_err()
        );
        drop(cache);
        cloned
            .claim::<String>(ExecutorKind::Rest, &key(), first)
            .unwrap();
        isolated
            .claim::<String>(ExecutorKind::Rest, &key(), second)
            .unwrap();
    }

    #[test]
    fn concurrent_conflicting_claims_accept_exactly_one_owner() {
        let temporary = tempfile::tempdir().unwrap();
        let cache = WorkCache::new(temporary.path()).unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let owners = [
            CacheOperationOwner::of::<FirstOperation>(),
            CacheOperationOwner::of::<SecondOperation>(),
        ];
        let jobs = owners.map(|owner| {
            let cache = cache.clone();
            let barrier = barrier.clone();
            thread::spawn(move || {
                barrier.wait();
                cache
                    .claim::<String>(ExecutorKind::Rest, &key(), owner)
                    .is_ok()
            })
        });
        let accepted = jobs
            .into_iter()
            .map(|job| usize::from(job.join().unwrap()))
            .sum::<usize>();
        assert_eq!(accepted, 1);
    }

    #[test]
    fn construction_and_claiming_do_not_require_or_create_a_directory() {
        let temporary = tempfile::tempdir().unwrap();
        let missing = temporary.path().join("not-created");
        let cache = WorkCache::new(&missing).unwrap();
        cache
            .claim::<String>(
                ExecutorKind::Rest,
                &key(),
                CacheOperationOwner::of::<FirstOperation>(),
            )
            .unwrap();
        assert_eq!(cache.root(), missing);
        assert!(!missing.exists());

        // A synthetic file also succeeds: construction does not inspect root metadata.
        let existing_file = temporary.path().join("synthetic-file");
        std::fs::write(&existing_file, "owned fixture").unwrap();
        let cache = WorkCache::new(&existing_file).unwrap();
        cache
            .claim::<String>(
                ExecutorKind::Rest,
                &key(),
                CacheOperationOwner::of::<FirstOperation>(),
            )
            .unwrap();
        assert_eq!(
            std::fs::read_to_string(&existing_file).unwrap(),
            "owned fixture"
        );
    }

    #[test]
    fn root_validation_is_lexical_and_requires_an_absolute_nontraversing_path() {
        let temporary = tempfile::tempdir().unwrap();
        for relative in [Path::new(""), Path::new("cache"), Path::new("../cache")] {
            assert!(WorkCache::new(relative).is_err());
        }
        assert!(WorkCache::new(temporary.path().join("child").join("..")).is_err());
        let filesystem_root = temporary.path().ancestors().last().unwrap();
        assert!(WorkCache::new(filesystem_root).is_ok());
        #[cfg(windows)]
        for not_absolute in [r"C:cache", r"\cache"] {
            assert!(WorkCache::new(not_absolute).is_err());
        }
    }
}
